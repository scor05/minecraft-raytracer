use crate::material::Material;
use crate::ray_intersect::{Intersect, RayIntersect, SurfaceFace};
use raylib::prelude::{Vector2, Vector3};

#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    pub center: Vector3,
    pub radius: f32,
    pub material: Material,
}

impl RayIntersect for Sphere {
    fn ray_intersect(&self, ray_direction: &Vector3, ray_origin: &Vector3) -> Option<Intersect> {
        let origin_to_center = *ray_origin - self.center;

        let a = ray_direction.dot(*ray_direction);
        if a == 0.0 {
            return None;
        }

        let b = 2.0 * origin_to_center.dot(*ray_direction);
        let c = origin_to_center.dot(origin_to_center) - self.radius * self.radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            return None;
        }

        let discriminant_root = discriminant.sqrt();
        let denominator = 2.0 * a;
        let near_distance = (-b - discriminant_root) / denominator;
        let far_distance = (-b + discriminant_root) / denominator;

        let distance = if near_distance >= 0.0 {
            near_distance
        } else if far_distance >= 0.0 {
            far_distance
        } else {
            return None;
        };

        let point = *ray_origin + *ray_direction * distance;
        let normal = (point - self.center).normalize();

        Some(Intersect {
            material: self.material,
            distance,
            normal,
            point,
            face: SurfaceFace::Curved,
            uv: Vector2::zero(),
        })
    }
}
