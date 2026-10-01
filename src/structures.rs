use crate::cube::Cube;
use crate::material::Material;
use crate::procedural::generate_inverse_island;
use raylib::prelude::Vector3;
use std::collections::HashMap;

type Materials = HashMap<&'static str, Material>;

fn block_at(
    origin: Vector3,
    x: i32,
    level: i32,
    z: i32,
    block_size: f32,
    material: Material,
) -> Cube {
    Cube {
        center: Vector3::new(
            origin.x + x as f32 * block_size,
            origin.y - level as f32 * block_size,
            origin.z + z as f32 * block_size,
        ),
        length: block_size,
        material,
    }
}

pub fn generate_castle(origin: Vector3, block_size: f32, materials: &Materials) -> Vec<Cube> {
    let stone = materials["stone_brick"];
    let glass = materials["glass"];
    let log = materials["log"];
    let carpet = materials["red_carpet"];
    let glowstone = materials["glowstone"];
    let chest = materials["chest"];
    let mut blocks = Vec::new();

    // Five levels: perimeter walls for the lower four and a solid roof on five.
    for level in 1..=5 {
        for x in 0..5 {
            for z in 0..5 {
                let perimeter = x == 0 || x == 4 || z == 0 || z == 4;
                let roof = level == 5;
                if !perimeter && !roof {
                    continue;
                }

                // The camera-facing wall is z == 0. Leave a two-block door.
                if z == 0 && x == 2 && level <= 2 {
                    continue;
                }

                let middle_window = level == 3
                    && ((z == 0 || z == 4) && x % 2 == 1 || (x == 0 || x == 4) && z % 2 == 1);
                let material = if middle_window { glass } else { stone };
                blocks.push(block_at(origin, x, level, z, block_size, material));
            }
        }
    }

    // Alternating crenellations around the roof. All four corners are included.
    for x in 0..5 {
        for z in 0..5 {
            let perimeter = x == 0 || x == 4 || z == 0 || z == 4;
            if perimeter && (x + z) % 2 == 0 {
                blocks.push(block_at(origin, x, 6, z, block_size, stone));
            }
        }
    }

    // Flag pole and a two-block red flag over the center of the roof.
    for level in 6..=8 {
        blocks.push(block_at(origin, 2, level, 2, block_size, log));
    }
    blocks.push(block_at(origin, 3, 8, 2, block_size, carpet));
    blocks.push(block_at(origin, 4, 8, 2, block_size, carpet));

    // Self-lit block inside the hollow room.
    blocks.push(block_at(origin, 2, 2, 2, block_size, glowstone));

    // Two chests outside and to the right of the front doorway.
    blocks.push(block_at(origin, 3, 1, -1, block_size, chest));
    blocks.push(block_at(origin, 4, 1, -1, block_size, chest));

    blocks
}

pub fn generate_tree(ground: Vector3, block_size: f32, materials: &Materials) -> Vec<Cube> {
    let log = materials["log"];
    let leaves = materials["leaves"];
    let mut blocks = Vec::new();

    for level in 1..=4 {
        blocks.push(block_at(ground, 0, level, 0, block_size, log));
    }

    // A 7x7 inverse island gives the canopy an approximate three-block radius.
    let canopy_origin = Vector3::new(
        ground.x - 3.0 * block_size,
        ground.y - 4.0 * block_size,
        ground.z - 3.0 * block_size,
    );
    let mut canopy = generate_inverse_island(canopy_origin, 7, 7, 3, block_size, leaves, leaves);

    // Let the trunk occupy the center of the canopy's lowest layer.
    canopy.retain(|cube| {
        (cube.center.x - ground.x).abs() > f32::EPSILON
            || (cube.center.z - ground.z).abs() > f32::EPSILON
            || (cube.center.y - canopy_origin.y).abs() > f32::EPSILON
    });
    blocks.extend(canopy);
    blocks
}
