use raylib::prelude::*;

pub struct Light {
    pub pos: Vector3,
    pub intensity: f32,
    // tamaño de la luz para que pueda salir del objeto
    pub source_radius: f32,
}
