use crate::material::Material;
use crate::ray_intersect::{Intersect, RayIntersect};
use raylib::prelude::Vector3;

#[derive(Debug, Clone, Copy)]
pub struct Cube {
    pub center: Vector3,
    pub length: f32,
    pub material: Material,
}

impl RayIntersect for Cube {
    fn ray_intersect(&self, ray_direction: &Vector3, ray_origin: &Vector3) -> Option<Intersect> {
        if self.length <= 0.0 {
            return None;
        }

        let half_length = self.length * 0.5;
        let min = self.center - Vector3::new(half_length, half_length, half_length);
        let max = self.center + Vector3::new(half_length, half_length, half_length);

        let mut near_distance = f32::NEG_INFINITY;
        let mut far_distance = f32::INFINITY;
        let mut near_normal = Vector3::zero();
        let mut far_normal = Vector3::zero();

        if !intersect_axis(
            ray_origin.x,
            ray_direction.x,
            min.x,
            max.x,
            Vector3::new(-1.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            &mut near_distance,
            &mut far_distance,
            &mut near_normal,
            &mut far_normal,
        ) || !intersect_axis(
            ray_origin.y,
            ray_direction.y,
            min.y,
            max.y,
            Vector3::new(0.0, -1.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            &mut near_distance,
            &mut far_distance,
            &mut near_normal,
            &mut far_normal,
        ) || !intersect_axis(
            ray_origin.z,
            ray_direction.z,
            min.z,
            max.z,
            Vector3::new(0.0, 0.0, -1.0),
            Vector3::new(0.0, 0.0, 1.0),
            &mut near_distance,
            &mut far_distance,
            &mut near_normal,
            &mut far_normal,
        ) {
            return None;
        }

        let (distance, normal) = if near_distance >= 0.0 {
            (near_distance, near_normal)
        } else if far_distance >= 0.0 {
            (far_distance, far_normal)
        } else {
            return None;
        };

        Some(Intersect {
            material: self.material,
            distance,
            normal,
            point: *ray_origin + *ray_direction * distance,
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn intersect_axis(
    origin: f32,
    direction: f32,
    min: f32,
    max: f32,
    min_normal: Vector3,
    max_normal: Vector3,
    near_distance: &mut f32,
    far_distance: &mut f32,
    near_normal: &mut Vector3,
    far_normal: &mut Vector3,
) -> bool {
    if direction.abs() < f32::EPSILON {
        return origin >= min && origin <= max;
    }

    let min_distance = (min - origin) / direction;
    let max_distance = (max - origin) / direction;
    let (axis_near, axis_far, axis_near_normal, axis_far_normal) = if min_distance <= max_distance {
        (min_distance, max_distance, min_normal, max_normal)
    } else {
        (max_distance, min_distance, max_normal, min_normal)
    };

    if axis_near > *near_distance {
        *near_distance = axis_near;
        *near_normal = axis_near_normal;
    }
    if axis_far < *far_distance {
        *far_distance = axis_far;
        *far_normal = axis_far_normal;
    }

    *near_distance <= *far_distance
}
