use nalgebra_glm::{Vec3, Mat4, look_at, perspective};
use minifb::{Key, Window, WindowOptions};
use std::time::Duration;
use std::f32::consts::PI;

mod framebuffer;
mod triangle;
mod vertex;
mod obj;
mod color;
mod fragment;
mod shaders;
mod camera;

use framebuffer::Framebuffer;
use vertex::Vertex;
use obj::Obj;
use camera::Camera;
use triangle::triangle;
use shaders::{vertex_shader, fragment_shader};
use fastnoise_lite::{FastNoiseLite, NoiseType, FractalType};

pub struct Uniforms {
    model_matrix: Mat4,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    viewport_matrix: Mat4,
    time: u32,
    noise: FastNoiseLite
}

fn create_noise() -> FastNoiseLite {
    create_cloud_noise()
}

fn create_cloud_noise() -> FastNoiseLite {
    let mut noise = FastNoiseLite::with_seed(1337);
    noise.set_noise_type(Some(NoiseType::OpenSimplex2));
    noise
}

fn create_model_matrix(translation: Vec3, scale: f32, rotation: Vec3) -> Mat4 {
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();

    let rotation_matrix_x = Mat4::new(
        1.0,  0.0,    0.0,   0.0,
        0.0,  cos_x, -sin_x, 0.0,
        0.0,  sin_x,  cos_x, 0.0,
        0.0,  0.0,    0.0,   1.0,
    );

    let rotation_matrix_y = Mat4::new(
        cos_y,  0.0,  sin_y, 0.0,
        0.0,    1.0,  0.0,   0.0,
        -sin_y, 0.0,  cos_y, 0.0,
        0.0,    0.0,  0.0,   1.0,
    );

    let rotation_matrix_z = Mat4::new(
        cos_z, -sin_z, 0.0, 0.0,
        sin_z,  cos_z, 0.0, 0.0,
        0.0,    0.0,  1.0, 0.0,
        0.0,    0.0,  0.0, 1.0,
    );

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;

    let transform_matrix = Mat4::new(
        scale, 0.0,   0.0,   translation.x,
        0.0,   scale, 0.0,   translation.y,
        0.0,   0.0,   scale, translation.z,
        0.0,   0.0,   0.0,   1.0,
    );

    transform_matrix * rotation_matrix
}

fn create_orbit_matrix(center: Vec3, radius: f32, speed: f32, time: f32) -> Mat4 {
    let angle = time * speed;
    let x = center.x + radius * angle.cos();
    let y = center.y; 
    let z = center.z + radius * angle.sin();
    Mat4::new_translation(&Vec3::new(x, y, z))
}

fn create_view_matrix(eye: Vec3, center: Vec3, up: Vec3) -> Mat4 {
    look_at(&eye, &center, &up)
}

fn create_perspective_matrix(window_width: f32, window_height: f32) -> Mat4 {
    let fov = 45.0 * PI / 180.0;
    let aspect_ratio = window_width / window_height;
    let near = 0.1;
    let far = 1000.0;

    perspective(fov, aspect_ratio, near, far)
}

fn create_viewport_matrix(width: f32, height: f32) -> Mat4 {
    Mat4::new(
        width / 2.0, 0.0, 0.0, width / 2.0,
        0.0, -height / 2.0, 0.0, height / 2.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    )
}

pub fn render(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex], sphere_index: usize) {
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, uniforms);
        transformed_vertices.push(transformed);
    }

    let mut triangles = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
        }
    }

    let mut fragments = Vec::new();
    for tri in &triangles {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2]));
    }

    for fragment in fragments {
        let x = fragment.position.x as usize;
        let y = fragment.position.y as usize;

        if x < framebuffer.width && y < framebuffer.height {
            let shaded_color = fragment_shader(&fragment, &uniforms, sphere_index);
            let color = shaded_color.to_hex();
            framebuffer.set_current_color(color);
            framebuffer.point(x, y, fragment.depth);
        }
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;
    let framebuffer_width = 800;
    let framebuffer_height = 600;
    let frame_delay = Duration::from_millis(16);

    let mut framebuffer = Framebuffer::new(framebuffer_width, framebuffer_height);
    let mut window = Window::new(
        "Animated Fragment Shader",
        window_width,
        window_height,
        WindowOptions::default(),
    ).unwrap();

    window.set_position(500, 500);
    window.update();

    framebuffer.set_background_color(0x333355);

    
    let mut camera = Camera::new(
        Vec3::new(0.0, 3.0, 5.0),  
        Vec3::new(0.0, 0.0, 0.0),  
        Vec3::new(0.0, 1.0, 0.0),  
    );
    let sphere_params = [
        (Vec3::new(0.0, 0.0, 0.0), 0.7, 0.0, 0.0),  
        (Vec3::new(-2.0, 0.0, 0.0), 0.5, 0.2, 0.0), 
        (Vec3::new(2.0, 0.0, 0.0), 0.5, 0.2, 1.0),  
        (Vec3::new(0.0, 2.0, 0.0), 0.5, 0.2, 2.0),  
        (Vec3::new(0.0, -2.0, 0.0), 0.5, 0.2, 3.0), 
        (Vec3::new(1.5, 1.5, 0.0), 0.5, 0.2, 4.0),  
        (Vec3::new(-1.5, -1.5, 0.0), 0.5, 0.2, 5.0), 
    ];

    let obj = Obj::load("assets/models/sphere.obj").expect("Failed to load obj");
    let vertex_arrays = obj.get_vertex_array();
    let mut time = 0;

    while window.is_open() {
        if window.is_key_down(Key::Escape) {
            break;
        }

        
        time += 1;

        
        if window.is_key_down(Key::Left) {
            camera.orbit(PI / 50.0, 0.0); 
        }
        if window.is_key_down(Key::Right) {
            camera.orbit(-PI / 50.0, 0.0); 
        }
        if window.is_key_down(Key::Up) {
            camera.orbit(0.0, -PI / 50.0); 
        }
        if window.is_key_down(Key::Down) {
            camera.orbit(0.0, PI / 50.0); 
        }
        if window.is_key_down(Key::W) {
            camera.zoom(0.1); 
        }
        if window.is_key_down(Key::S) {
            camera.zoom(-0.1); 
        }

        framebuffer.clear();

        let view_matrix = nalgebra_glm::look_at(
            &camera.eye,
            &camera.center,
            &camera.up,
        );
        let projection_matrix = create_perspective_matrix(window_width as f32, window_height as f32);
        let viewport_matrix = create_viewport_matrix(framebuffer_width as f32, framebuffer_height as f32);

        
        for (i, (position, scale, speed, phase)) in sphere_params.iter().enumerate() {
            let orbit_radius = position.magnitude();
            let orbit_angle = time as f32 * speed * 0.01 + phase;

            let orbit_position = Vec3::new(
                orbit_radius * orbit_angle.cos(),
                orbit_radius * orbit_angle.sin(),
                position.z,
            );

            let model_matrix = create_model_matrix(orbit_position, *scale, Vec3::zeros());

            let uniforms = Uniforms {
                model_matrix,
                view_matrix,
                projection_matrix,
                viewport_matrix,
                time,
                noise: create_noise(),
            };

            render(&mut framebuffer, &uniforms, &vertex_arrays, i);
        }

        window
            .update_with_buffer(&framebuffer.buffer, framebuffer_width, framebuffer_height)
            .unwrap();

        std::thread::sleep(frame_delay);
    }
}
