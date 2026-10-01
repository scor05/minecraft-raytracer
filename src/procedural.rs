use crate::cube::Cube;
use crate::material::Material;
use noise::{Fbm, NoiseFn, Perlin};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use raylib::prelude::Vector3;

// Para generar terrenos, se usan funciones de ruido, las cuales esencialmente
// retornan un mapa 2D de valores random (que en verdad no son random porque usan
// una semilla, solo se puede hacer pseudo-random por cosas muy precisas como
// temperaturas de procesadores, el tiempo, etc...)

// Los algorítmos para generar terreno que más se usan son OpenSimplex2 y Perlin (minecraft
// usa Perlin), pero hay otros como Cellular que retornan patrones más cercanos a células
// (https://auburn.github.io/FastNoiseLite/)

// genera islas en vez de solo planos
pub fn generate_island(
    center: Vector3,
    width: i32,
    depth: i32,
    height: i32,
    cube_size: f32,
    top_material: Material,
    layer_material: Material,
) -> Vec<Cube> {
    let seed = 67;
    let mut rng = StdRng::seed_from_u64(seed);
    let mut map = Vec::with_capacity((width * depth) as usize);
    let noise_fn = Fbm::<Perlin>::new(seed as u32);
    let center_x = (width - 1) as f32 / 2.0;
    let center_y = (depth - 1) as f32 / 2.0;
    let top_radius = width.min(depth) as f32 / 2.0;

    for z in 0..height {
        for x in 0..width {
            for y in 0..depth {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;

                if dx * dx + dy * dy <= (top_radius - z as f32) * (top_radius - z as f32) {
                    let material;
                    let rand = noise_fn.get([x as f64, y as f64]) as f32 * height as f32;
                    let pos = if z != 0 {
                        if rng.random_range(0..10) > 2 + z {
                            material = layer_material;
                            Vector3::new(
                                center.x + (x as f32 * cube_size),
                                center.y
                                    + ((rand.round() + z as f32) * cube_size)
                                        .clamp(cube_size, height as f32 * cube_size),
                                center.z + (y as f32 * cube_size),
                            )
                        } else {
                            continue;
                        }
                    } else {
                        material = top_material;
                        Vector3::new(
                            center.x + (x as f32 * cube_size),
                            center.y,
                            center.z + (y as f32 * cube_size),
                        )
                    };

                    map.push(Cube {
                        center: pos,
                        length: cube_size,
                        material,
                    });
                }
            }
        }
    }
    map
}

pub fn generate_inverse_island(
    center: Vector3,
    width: i32,
    depth: i32,
    height: i32,
    cube_size: f32,
    top_material: Material,
    layer_material: Material,
) -> Vec<Cube> {
    let seed = 67;
    let mut rng = StdRng::seed_from_u64(seed);
    let mut map = Vec::with_capacity((width * depth) as usize);
    let noise_fn = Fbm::<Perlin>::new(seed as u32);
    let center_x = (width - 1) as f32 / 2.0;
    let center_y = (depth - 1) as f32 / 2.0;
    let top_radius = width.min(depth) as f32 / 2.0;

    let mut z = height;
    while z >= 0 {
        for x in 0..width {
            for y in 0..depth {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;

                if dx * dx + dy * dy <= (top_radius - z as f32) * (top_radius - z as f32) {
                    let material;
                    let rand = noise_fn.get([x as f64, y as f64]) as f32 * height as f32;
                    let pos = if z != 0 {
                        if rng.random_range(0..10) < 8 - z {
                            material = layer_material;
                            Vector3::new(
                                center.x + (x as f32 * cube_size),
                                center.y
                                    + ((rand.round() - z as f32) * cube_size)
                                        .clamp(-height as f32 * cube_size, -cube_size),
                                center.z + (y as f32 * cube_size),
                            )
                        } else {
                            continue;
                        }
                    } else {
                        material = top_material;
                        Vector3::new(
                            center.x + (x as f32 * cube_size),
                            center.y,
                            center.z + (y as f32 * cube_size),
                        )
                    };

                    map.push(Cube {
                        center: pos,
                        length: cube_size,
                        material,
                    });
                }
            }
        }
        z -= 1;
    }
    map
}

pub fn replace_top_layer_with_river(
    blocks: &mut [Cube],
    center: Vector3,
    width: i32,
    depth: i32,
    cube_size: f32,
    water: Material,
) {
    for block in blocks {
        if (block.center.y - center.y).abs() > f32::EPSILON {
            continue;
        }

        let grid_x = ((block.center.x - center.x) / cube_size).round() as i32;
        let grid_z = ((block.center.z - center.z) / cube_size).round() as i32;
        if grid_x < 0 || grid_x >= width || grid_z < 0 || grid_z >= depth {
            continue;
        }

        // A subtle bend keeps the river from looking like a perfectly cut line.
        let river_z = depth - 4
            + match grid_x.rem_euclid(6) {
                0 | 1 => 0,
                2 | 5 => 1,
                _ => 2,
            };

        if grid_z == river_z {
            block.material = water;
            // Sink the water surface slightly below the surrounding grass.
            block.center.y += cube_size * 0.12;
        }
    }
}
