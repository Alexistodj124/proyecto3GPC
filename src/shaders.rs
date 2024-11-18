
use nalgebra_glm::{Vec3, Vec4, Mat3, dot, mat4_to_mat3};
use crate::vertex::Vertex;
use crate::Uniforms;
use crate::fragment::Fragment;
use crate::color::Color;
use std::f32::consts::PI;
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::StdRng;

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    let position = Vec4::new(
        vertex.position.x,
        vertex.position.y,
        vertex.position.z,
        1.0
    );

    let transformed = uniforms.projection_matrix * uniforms.view_matrix * uniforms.model_matrix * position;

    let w = transformed.w;
    let transformed_position = Vec4::new(
        transformed.x / w,
        transformed.y / w,
        transformed.z / w,
        1.0
    );

    let screen_position = uniforms.viewport_matrix * transformed_position;

    let model_mat3 = mat4_to_mat3(&uniforms.model_matrix);
    let normal_matrix = model_mat3.transpose().try_inverse().unwrap_or(Mat3::identity());

    let transformed_normal = normal_matrix * vertex.normal;

    Vertex {
        position: vertex.position,
        normal: vertex.normal,
        tex_coords: vertex.tex_coords,
        color: vertex.color,
        transformed_position: Vec3::new(screen_position.x, screen_position.y, screen_position.z),
        transformed_normal: transformed_normal
    }
}

pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms, sphere_index: usize) -> Color {
  match sphere_index {
      0 => earth_shader(fragment, uniforms),
      1 => dalmata_shader(fragment, uniforms),
      2 => cloud_shader(fragment, uniforms),
      3 => cellular_shader(fragment, uniforms),
      4 => lava_shader(fragment, uniforms),
      5 => rocky_planet_shader(fragment, uniforms),
      6 => solar_shader(fragment, uniforms),  
      7 => gaseous_planet_shader(fragment, uniforms),
      _ => black_and_white(fragment, uniforms),  
  }
}



fn black_and_white(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let seed = uniforms.time as f32 * fragment.vertex_position.y * fragment.vertex_position.x;
  
    let mut rng = StdRng::seed_from_u64(seed.abs() as u64);
  
    let random_number = rng.gen_range(0..=100);
  
    let black_or_white = if random_number < 50 {
      Color::new(0, 0, 0)
    } else {
      Color::new(255, 255, 255)
    };
  
    black_or_white * fragment.intensity
}
  
fn dalmata_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 100.0;
    let ox = 0.0;
    let oy = 0.0;
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;
  
    let noise_value = uniforms.noise.get_noise_2d(
      (x + ox) * zoom,
      (y + oy) * zoom,
    );
  
    let spot_threshold = 0.5;
    let spot_color = Color::new(255, 255, 255); // White
    let base_color = Color::new(0, 0, 0); // Black
  
    let noise_color = if noise_value < spot_threshold {
      spot_color
    } else {
      base_color
    };
  
    noise_color * fragment.intensity
}
  
fn cloud_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 100.0;  // to move our values 
    let ox = 100.0; // offset x in the noise map
    let oy = 100.0;
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;
    let t = uniforms.time as f32 * 0.5;
  
    let noise_value = uniforms.noise.get_noise_2d(x * zoom + ox + t, y * zoom + oy);
  
    // Define cloud threshold and colors
    let cloud_threshold = 0.5; // Adjust this value to change cloud density
    let cloud_color = Color::new(255, 255, 255); // White for clouds
    let sky_color = Color::new(30, 97, 145); // Sky blue
  
    // Determine if the pixel is part of a cloud or sky
    let noise_color = if noise_value > cloud_threshold {
      cloud_color
    } else {
      sky_color
    };
  
    noise_color * fragment.intensity
}
  
fn cellular_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 30.0;  // Zoom factor to adjust the scale of the cell pattern
    let ox = 50.0;    // Offset x in the noise map
    let oy = 50.0;    // Offset y in the noise map
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;
  
    // Use a cellular noise function to create the plant cell pattern
    let cell_noise_value = uniforms.noise.get_noise_2d(x * zoom + ox, y * zoom + oy).abs();
  
    // Define different shades of green for the plant cells
    let cell_color_1 = Color::new(85, 107, 47);   // Dark olive green
    let cell_color_2 = Color::new(124, 252, 0);   // Light green
    let cell_color_3 = Color::new(34, 139, 34);   // Forest green
    let cell_color_4 = Color::new(173, 255, 47);  // Yellow green
  
    // Use the noise value to assign a different color to each cell
    let final_color = if cell_noise_value < 0.15 {
      cell_color_1
    } else if cell_noise_value < 0.7 {
      cell_color_2
    } else if cell_noise_value < 0.75 {
      cell_color_3
    } else {
      cell_color_4
    };
  
    // Adjust intensity to simulate lighting effects (optional)
    final_color * fragment.intensity
}
  
fn lava_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    // Base colors for the lava effect
    let bright_color = Color::new(255, 240, 0); // Bright orange (lava-like)
    let dark_color = Color::new(130, 20, 0);   // Darker red-orange
  
    // Get fragment position
    let position = Vec3::new(
      fragment.vertex_position.x,
      fragment.vertex_position.y,
      fragment.depth
    );
  
    // Base frequency and amplitude for the pulsating effect
    let base_frequency = 0.2;
    let pulsate_amplitude = 0.5;
    let t = uniforms.time as f32 * 0.01;
  
    // Pulsate on the z-axis to change spot size
    let pulsate = (t * base_frequency).sin() * pulsate_amplitude;
  
    // Apply noise to coordinates with subtle pulsating on z-axis
    let zoom = 1000.0; // Constant zoom factor
    let noise_value1 = uniforms.noise.get_noise_3d(
      position.x * zoom,
      position.y * zoom,
      (position.z + pulsate) * zoom
    );
    let noise_value2 = uniforms.noise.get_noise_3d(
      (position.x + 1000.0) * zoom,
      (position.y + 1000.0) * zoom,
      (position.z + 1000.0 + pulsate) * zoom
    );
    let noise_value = (noise_value1 + noise_value2) * 0.5;  // Averaging noise for smoother transitions
  
    // Use lerp for color blending based on noise value
    let color = dark_color.lerp(&bright_color, noise_value);
  
    color * fragment.intensity
}
  fn rocky_planet_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 50.0; 
    let ox = 10.0;  
    let oy = 20.0;
    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;

    let noise_value = uniforms.noise.get_noise_2d(x * zoom + ox, y * zoom + oy).abs();

    let mountain_color = Color::new(139, 69, 19); 
    let plain_color = Color::new(205, 133, 63);  
    let lowland_color = Color::new(222, 184, 135);   

    let final_color = if noise_value < 0.2 {
        lowland_color  
    } else if noise_value < 0.5 {
        plain_color  
    } else {
        mountain_color 
    };

    final_color * fragment.intensity
  }
  fn gaseous_planet_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
    let zoom = 0.3;  // Controla la densidad de las bandas de gas
    let speed = 0.1; // Velocidad del movimiento de gas
    let time = uniforms.time as f32 * speed;

    let x = fragment.vertex_position.x;
    let y = fragment.vertex_position.y;
    let z = fragment.depth;

    // Generar ruido para las franjas gaseosas con movimiento
    let noise_value = uniforms.noise.get_noise_3d(x * zoom, (y + time) * zoom, z * zoom).abs();

    // Definir colores suaves y etéreos para el planeta gaseoso
    let gas_color_1 = Color::new(135, 206, 250); // Azul cielo
    let gas_color_2 = Color::new(176, 224, 230); // Azul claro
    let gas_color_3 = Color::new(255, 228, 196); // Beige suave

    // Interpolación de colores para crear capas gaseosas
    let final_color = if noise_value < 0.4 {
        gas_color_1
    } else if noise_value < 0.7 {
        gas_color_2
    } else {
        gas_color_3
    };

    // Ajustar intensidad para efectos de iluminación sutiles
    final_color * fragment.intensity
}


fn solar_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let zoom = 20.0;
  let speed = 0.2; 
  let time = uniforms.time as f32 * speed;

  let x = fragment.vertex_position.x;
  let y = fragment.vertex_position.y;
  let z = fragment.depth;

  let noise_value1 = uniforms.noise.get_noise_3d(x * zoom + time, y * zoom, z * zoom).abs();
  let noise_value2 = uniforms.noise.get_noise_3d((x + 50.0) * zoom, (y + 50.0) * zoom, (z + time) * zoom).abs();
  let combined_noise = (noise_value1 + noise_value2) * 0.5;

  let core_color = Color::new(255, 140, 0);   
  let flare_color = Color::new(255, 69, 0);   
  let corona_color = Color::new(255, 215, 0);  

  let final_color = if combined_noise < 0.3 {
      corona_color  
  } else if combined_noise < 0.6 {
      core_color    
  } else {
      flare_color   
  };

  final_color * fragment.intensity
}

fn earth_shader(fragment: &Fragment, uniforms: &Uniforms) -> Color {
  let zoom = 20.0; 
  let speed = 0.10;
  let time = uniforms.time as f32 * speed;

  let x = fragment.vertex_position.x;
  let y = fragment.vertex_position.y;
  let z = fragment.depth;

  let noise_value1 = uniforms.noise.get_noise_3d(x * zoom + time, y * zoom, z * zoom).abs();
  let noise_value2 = uniforms.noise.get_noise_3d((x + 50.0) * zoom, (y + 50.0) * zoom, (z + time) * zoom).abs();
  let combined_noise = (noise_value1 + noise_value2) * 0.5; 

  let ocean_color = Color::new(0, 105, 148);   
  let land_color = Color::new(34, 139, 34); 
  let mountain_color = Color::new(139, 69, 19);   

  let final_color = if combined_noise < 0.3 {
      ocean_color   
  } else if combined_noise < 0.6 {
      land_color     
  } else {
      mountain_color 
  };

  final_color * fragment.intensity
}
