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

pub fn castle_lantern_center(origin: Vector3, block_size: f32) -> Vector3 {
    let lantern_size = 1.5;
    Vector3::new(
        origin.x + 2.0 * block_size,
        origin.y - 4.5 * block_size + block_size * 0.5 + lantern_size * 0.5 + block_size,
        origin.z + 2.0 * block_size,
    )
}

pub fn generate_castle(origin: Vector3, block_size: f32, materials: &Materials) -> Vec<Cube> {
    let stone = materials["stone_brick"];
    let glass = materials["glass"];
    let log = materials["log"];
    let carpet = materials["red_carpet"];
    let lantern = materials["lantern"];
    let gold = materials["gold"];
    let barrel = materials["barrel"];
    let chest = materials["chest"];
    let mut blocks = Vec::new();

    for level in 1..=5 {
        for x in 0..5 {
            for z in 0..5 {
                let perimeter = x == 0 || x == 4 || z == 0 || z == 4;
                let roof = level == 5;
                if !perimeter && !roof {
                    continue;
                }

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

    for x in 0..5 {
        for z in 0..5 {
            let perimeter = x == 0 || x == 4 || z == 0 || z == 4;
            if perimeter && (x + z) % 2 == 0 {
                blocks.push(block_at(origin, x, 6, z, block_size, stone));
            }
        }
    }

    for level in 6..=9 {
        blocks.push(block_at(origin, 2, level, 2, block_size, log));
    }
    blocks.push(block_at(origin, 3, 8, 2, block_size, carpet));
    blocks.push(block_at(origin, 4, 8, 2, block_size, carpet));
    blocks.push(block_at(origin, 3, 9, 2, block_size, carpet));
    blocks.push(block_at(origin, 4, 9, 2, block_size, carpet));

    blocks.push(Cube {
        center: castle_lantern_center(origin, block_size)
            - Vector3::new(block_size, -block_size * 0.25, block_size),
        length: 1.5,
        material: lantern,
    });

    blocks.push(Cube {
        center: castle_lantern_center(origin, block_size)
            + Vector3::new(block_size, block_size * 0.25, block_size),
        length: 1.5,
        material: lantern,
    });

    blocks.push(block_at(origin, 2, 1, 2, block_size, gold));

    blocks.push(block_at(origin, 3, 1, -1, block_size, chest));
    blocks.push(block_at(origin, 4, 1, -1, block_size, chest));

    blocks.push(block_at(origin, 1, 1, -1, block_size, barrel));

    for z in 0..=4 {
        blocks.push(block_at(origin, 5, 1, z, block_size, barrel));
    }
    for z in [1, 3] {
        blocks.push(block_at(origin, 5, 2, z, block_size, barrel));
    }

    blocks
}

pub fn generate_tree(ground: Vector3, block_size: f32, materials: &Materials) -> Vec<Cube> {
    let log = materials["log"];
    let leaves = materials["leaves"];
    let mut blocks = Vec::new();

    for level in 1..=4 {
        blocks.push(block_at(ground, 0, level, 0, block_size, log));
    }

    let canopy_origin = Vector3::new(
        ground.x - 3.0 * block_size,
        ground.y - 4.0 * block_size,
        ground.z - 3.0 * block_size,
    );
    let mut canopy = generate_inverse_island(canopy_origin, 7, 7, 3, block_size, leaves, leaves);

    canopy.retain(|cube| {
        (cube.center.x - ground.x).abs() > f32::EPSILON
            || (cube.center.z - ground.z).abs() > f32::EPSILON
            || (cube.center.y - canopy_origin.y).abs() > f32::EPSILON
    });
    blocks.extend(canopy);
    blocks
}

pub fn generate_bridge(
    island_origin: Vector3,
    island_width: i32,
    length: i32,
    width: i32,
    block_size: f32,
    material: Material,
) -> Vec<Cube> {
    let first_x = (island_width - width) / 2;
    let mut blocks = Vec::with_capacity((length * width) as usize);

    for z in 1..=length {
        for x in 0..width {
            blocks.push(Cube {
                center: Vector3::new(
                    island_origin.x + (first_x + x) as f32 * block_size,
                    island_origin.y,
                    island_origin.z - z as f32 * block_size,
                ),
                length: block_size,
                material,
            });
        }
    }

    blocks
}

pub fn generate_ruined_portal(
    island_origin: Vector3,
    block_size: f32,
    materials: &Materials,
) -> Vec<Cube> {
    let obsidian = materials["obsidian"];
    let crying_obsidian = materials["crying_obsidian"];
    let gold = materials["gold"];
    let stone = materials["stone"];
    let stone_brick = materials["stone_brick"];
    let frame_origin = island_origin + Vector3::new(5.0 * block_size, 0.0, 8.0 * block_size);
    let mut blocks = Vec::new();

    for x in 0..=4 {
        let material = if x == 1 { crying_obsidian } else { obsidian };
        blocks.push(block_at(frame_origin, x, 1, 0, block_size, material));
    }

    for level in 2..=6 {
        let material = if level == 3 {
            crying_obsidian
        } else {
            obsidian
        };
        blocks.push(block_at(frame_origin, 0, level, 0, block_size, material));
    }

    for level in 2..=4 {
        let material = if level == 4 {
            crying_obsidian
        } else {
            obsidian
        };
        blocks.push(block_at(frame_origin, 4, level, 0, block_size, material));
    }
    for x in 1..=2 {
        let material = if x == 2 { crying_obsidian } else { obsidian };
        blocks.push(block_at(frame_origin, x, 6, 0, block_size, material));
    }

    for x in 0..=2 {
        blocks.push(block_at(frame_origin, x, 7, 0, block_size, gold));
    }

    for x in -1..=5 {
        for z in -3..=-1 {
            if (x == -1 && z == -3) || (x == 3 && z == -2) || (x == 5 && z == -1) {
                continue;
            }
            blocks.push(block_at(frame_origin, x, 1, z, block_size, stone));
        }
    }

    for x in -1..=5 {
        if x != 2 {
            blocks.push(block_at(frame_origin, x, 1, -4, block_size, stone_brick));
        }
    }
    for z in -3..=1 {
        blocks.push(block_at(frame_origin, -2, 1, z, block_size, stone_brick));
        if z != -1 {
            blocks.push(block_at(frame_origin, 6, 1, z, block_size, stone_brick));
        }
    }
    for x in -1..=5 {
        if x != 3 {
            blocks.push(block_at(frame_origin, x, 1, 1, block_size, stone_brick));
        }
    }

    blocks
}
