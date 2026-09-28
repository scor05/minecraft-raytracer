use raylib::prelude::Color;

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub diffuse: Color,
    // Pesos de color difuso, brillo especular, reflexión y transparencia.
    pub albedo: [f32; 4],
    pub specular: f32, // define qué tan grande es el circulito de reflexión, va de 1-256
    // Índice de refracción (IOR) del material.
    pub refraction_index: f32,
}
