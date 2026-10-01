use crate::material::Material;
use raylib::prelude::{Vector2, Vector3};

#[derive(Debug, Clone, Copy)]
pub enum SurfaceFace {
    Top,
    Bottom,
    Front,
    Back,
    Left,
    Right,
    Curved,
}

#[derive(Debug, Clone, Copy)]
pub struct Intersect {
    pub material: Material,
    pub distance: f32,
    pub normal: Vector3,
    pub point: Vector3,
    pub face: SurfaceFace,
    pub uv: Vector2,
}

pub trait RayIntersect {
    fn ray_intersect(&self, ray_direction: &Vector3, ray_origin: &Vector3) -> Option<Intersect>;
}
