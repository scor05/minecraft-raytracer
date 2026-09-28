use crate::material::Material;
use crate::sphere::Sphere;
use noise::{Fbm, NoiseFn, OpenSimplex};
use rand::rngs::StdRng;
use rand::{RngExt, SeedableRng};
use raylib::prelude::{Color, Vector3};

// Para generar terrenos, se usan funciones de ruido, las cuales esencialmente
// retornan un mapa 2D de valores random (que en verdad no son random porque usan
// una semilla, solo se puede hacer pseudo-random por cosas muy precisas como
// temperaturas de procesadores, el tiempo, etc...)

// Los algorítmos para generar terreno que más se usan son OpenSimplex2 y Perlin (minecraft
// usa Perlin), pero hay otros como Cellular que retornan patrones más cercanos a células
// (https://auburn.github.io/FastNoiseLite/)
pub fn generate_terrain(width: i32, depth: i32) -> Vec<Sphere> {
    let seed = 50 as u64;
    let spacing = 3.0;
    let mut rng = StdRng::seed_from_u64(seed);
    let mut map = Vec::with_capacity((width * depth) as usize);
    let noise_fn = Fbm::<OpenSimplex>::new(seed as u32);
    let material = Material {
        diffuse: Color::new(
            rng.random_range(35..=70),
            rng.random_range(120..=190),
            rng.random_range(35..=80),
            255,
        ),
        albedo: [0.85, 0.15, 0.05, 0.0],
        specular: 16.0,
        refraction_index: 1.52,
    };

    for x in 0..width {
        for y in 0..depth {
            let pos = Vector3::new(
                (x - width / 2) as f32 * spacing,
                noise_fn.get([x as f64, y as f64]) as f32 * 4.0,
                6.0 + y as f32 * spacing,
            );

            map.push(Sphere {
                center: pos,
                radius: 2.0,
                material,
            });
        }
    }
    map
}
