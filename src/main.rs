mod camera;
mod cube;
mod framebuffer;
mod light;
mod material;
mod procedural;
mod ray_intersect;
mod sphere;
mod structures;
mod textures;

use crate::camera::Camera;
use crate::framebuffer::Framebuffer;
use crate::light::Light;
use crate::material::load_materials;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::structures::{castle_lantern_center, generate_castle, generate_tree};
use crate::textures::{TextureId, TextureStore};
use procedural::*;
use rand::RngExt;
use raylib::prelude::*;
use std::f32::consts::PI;
use std::thread;
use std::time::Duration;

const FOV: f32 = PI / 2.0;
const FPS: u64 = 30;
const MS: u64 = 1000 / FPS;
const SHADOW_BIAS: f32 = 1e-4; // es qué tanto sumarle a los vectores que entran de la esfera
// viene de problemas de redondeo
const MAX_REFLECTIONS: u8 = 2;
const ISLAND_RADIUS: i32 = 16;
const BLOCK_SIZE: f32 = 3.0;
const ISLAND_HEIGHT: i32 = 5;
const RENDER_CHANCE: i32 = 98; // pobre compu no aguanta

// con esta función hacer que cada objeto al ser intersectado tire un rayo
// con el mismo ángulo con respecto a la normal para causar esas reflecciones.
// parar hasta max_reflections para prevenir loops infinitos (como espejo frente a espejo)
fn reflect(incident: &Vector3, normal: &Vector3) -> Vector3 {
    // I es la luz que viene, la normal es la normal del objeto con el que colisiona
    // fórmula que viene de la lógica de Phong: R = I - 2 * N (N · I)
    *incident - *normal * 2.0 * incident.dot(*normal)
}

// refracción depende de la densidad del material
// entre mayor n (IOR, index of refraction) más torcerá la luz (toma más energía atravesarlo)
fn refract(incident: &Vector3, normal: &Vector3, refractive_index: f32) -> Option<Vector3> {
    // Implementation of Snell's Law for refraction.
    // It calculates the direction of a ray as it passes from one medium to another.

    // `cosi` is the cosine of the angle between the incident ray and the normal.
    // We clamp it to the [-1, 1] range to avoid floating point errors.
    let mut cosi = incident.dot(*normal).max(-1.0).min(1.0);

    // `etai` is the refractive index of the medium the ray is currently in.
    // `etat` is the refractive index of the medium the ray is entering.
    // `n` is the normal vector, which may be flipped depending on the ray's direction.
    let mut etai = 1.0; // Assume we are in Air (or vacuum) initially
    let mut etat = refractive_index;
    let mut n = *normal;

    if cosi > 0.0 {
        // The ray is inside the medium (e.g., glass) and going out into the air.
        // We need to swap the refractive indices.
        std::mem::swap(&mut etai, &mut etat);
        // We also flip the normal so it points away from the medium.
        n = -n;
    } else {
        // The ray is outside the medium and going in.
        // We need a positive cosine for the calculation, so we negate it.
        cosi = -cosi;
    }

    // `eta` is the ratio of the refractive indices (n1 / n2).
    let eta = etai / etat;
    // `k` is a term derived from Snell's law that helps determine if total internal reflection occurs.
    let k = 1.0 - eta * eta * (1.0 - cosi * cosi);

    if k < 0.0 {
        // If k is negative, it means total internal reflection has occurred.
        // There is no refracted ray, so we return None.
        None
    } else {
        // If k is non-negative, we can calculate the direction of the refracted ray.
        Some(*incident * eta + n * (eta * cosi - k.sqrt()))
    }
}

// sumarle igual que a las sombras para que los rayos no se queden adentro de las esferas
fn offset_origin(intersect: &Intersect, direction: &Vector3) -> Vector3 {
    let offset = intersect.normal * SHADOW_BIAS;
    if direction.dot(intersect.normal) < 0.0 {
        intersect.point - offset
    } else {
        intersect.point + offset
    }
}

// esta función cubre los casos donde los objetos están puestos como uno atrás del otro
// en la misma línea de la luz, lo cual causa que al reboyar el rayo desde el objeto hasta la luz
// no va a llegar y va a topar con el primer objeto.
// retorna un f32 porque las sombras no reflejan la forma exacta de lo que las castea, sino que son
// difuminado. Retorna un valor de [0,1] para ver qué tan difuminado está
fn cast_shadow<T: RayIntersect>(
    intersect: &Intersect, // Para saber en dónde del segundo objeto topó
    light: &Light,         // hacia dónde iba el rayo
    objects: &[T],         // para checkear todos los objetos
) -> f32 {
    // para obtener la dirección de la luz desde el punto de intersecto inicial y normalizar.
    let light_dir = (light.pos - intersect.point).normalize(); // Para ir de AB se resta B - A,
    let light_distance = (light.pos - intersect.point).length(); // guardar la longitud

    // mover el origen un poco fuera de la superficie evita que la esfera se haga
    // sombra a sí misma por errores de precisión de punto flotante.
    let shadow_origin = offset_origin(&intersect, &light_dir);

    for o in objects {
        // sale desde el punto de intersecto con dirección hasta la dirección de la luz
        if let Some(shadow_intersect) = o.ray_intersect(&light_dir, &shadow_origin) {
            // solo bloquea la luz si el objeto está entre el punto y la luz.
            if shadow_intersect.distance < light_distance - light.source_radius.max(0.0) {
                return 1.0;
            }
        }
    }

    0.0
}

fn cast_ray<T: RayIntersect>(
    ray_origin: &Vector3,
    ray_direction: &Vector3,
    objects: &[T],
    lights: &[Light],
    textures: &TextureStore,
    skybox: TextureId,
    depth: u32, // depth de recursión
) -> Color {
    let mut closest_intersection: Option<Intersect> = None;

    if depth > MAX_REFLECTIONS as u32 {
        return textures.sample_skybox(skybox, ray_direction);
    }

    // pintar esferas más cercanas sobre las más lejanas
    for object in objects {
        if let Some(intersection) = object.ray_intersect(ray_direction, ray_origin) {
            let is_closer = closest_intersection
                .as_ref()
                .is_none_or(|closest| intersection.distance < closest.distance);
            if is_closer {
                closest_intersection = Some(intersection);
            }
        }
    }

    let Some(intersect) = closest_intersection else {
        return textures.sample_skybox(skybox, ray_direction);
    };

    let texture_color = textures.sample(
        intersect.material.textures.for_face(intersect.face),
        intersect.uv,
    );
    let material_color = Vector3::new(
        texture_color.r as f32 * intersect.material.diffuse.r as f32 / 255.0,
        texture_color.g as f32 * intersect.material.diffuse.g as f32 / 255.0,
        texture_color.b as f32 * intersect.material.diffuse.b as f32 / 255.0,
    );
    let view_dir = (*ray_origin - intersect.point).normalize();
    let mut diffuse = Vector3::zero();
    let mut specular = Vector3::zero();

    for light in lights {
        let light_dir = (light.pos - intersect.point).normalize();
        let shadow = cast_shadow(&intersect, light, objects);
        let received_intensity = (1.0 - shadow) * light.intensity.max(0.0);

        let diffuse_intensity = intersect.normal.dot(light_dir).max(0.0);
        diffuse +=
            material_color * diffuse_intensity * intersect.material.albedo[0] * received_intensity;

        let reflect_dir = reflect(&-light_dir, &intersect.normal);
        let specular_intensity = view_dir
            .dot(reflect_dir)
            .max(0.0)
            .powf(intersect.material.specular)
            * intersect.material.albedo[1]
            * received_intensity;
        specular += Vector3::new(1.0, 1.0, 1.0) * 255.0 * specular_intensity;
    }

    let mut reflect_color = Vector3::zero();
    let reflectivity = intersect.material.albedo[2];
    if reflectivity > 0.0 {
        let reflect_dir = reflect(&ray_direction, &intersect.normal);
        let reflect_origin = offset_origin(&intersect, &reflect_dir);
        reflect_color = color_to_vector(cast_ray(
            &reflect_origin,
            &reflect_dir,
            objects,
            lights,
            textures,
            skybox,
            depth + 1,
        ));
    }

    let transparency = intersect.material.albedo[3];
    let refract_color = if transparency > 0.0 {
        let refract_dir = if let Some(refract_dir) = refract(
            ray_direction,
            &intersect.normal,
            intersect.material.refraction_index,
        ) {
            refract_dir
        } else {
            reflect(ray_direction, &intersect.normal).normalize()
        };

        let refract_origin = offset_origin(&intersect, &refract_dir);
        color_to_vector(cast_ray(
            &refract_origin,
            &refract_dir,
            objects,
            lights,
            textures,
            skybox,
            depth + 1,
        ))
    } else {
        Vector3::zero()
    };

    let reflection = reflect_color * reflectivity;
    let refraction = refract_color * transparency;
    let emission = material_color * intersect.material.emission;

    let color = diffuse + specular + reflection + refraction + emission;

    vector_to_color(color)
}

fn color_to_vector(color: Color) -> Vector3 {
    Vector3::new(color.r as f32, color.g as f32, color.b as f32)
}

fn vector_to_color(vec: Vector3) -> Color {
    Color {
        r: vec.x as u8,
        g: vec.y as u8,
        b: vec.z as u8,
        a: 255,
    }
}

fn draw<T: RayIntersect + Sync>(
    fb: &mut Framebuffer,
    objects: &[T],
    camera: &Camera,
    lights: &[Light],
    textures: &TextureStore,
    skybox: TextureId,
    moving: &bool,
) {
    let width = fb.width as usize;
    let height = fb.height as usize;
    if width == 0 || height == 0 {
        return;
    }

    // anchura del plano de la imagen a distancia 1 (adyacente * tanx = op) con ady=1
    // FOV/2 para tener la mitad del plano
    let perspective_scale = (FOV / 2.0).tan();
    let aspect_ratio = width as f32 / height as f32;
    let moving = *moving;

    // detecta cores disponibles del cpu
    let worker_count = thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(1)
        .min(height);
    let rows_per_worker = height.div_ceil(worker_count);
    let mut pixels = vec![Color::BLACK; width * height];

    thread::scope(|scope| {
        for (chunk_index, rows) in pixels.chunks_mut(width * rows_per_worker).enumerate() {
            let starting_y = chunk_index * rows_per_worker;

            scope.spawn(move || {
                let mut rng = rand::rng();

                for (local_y, row) in rows.chunks_mut(width).enumerate() {
                    let y = starting_y + local_y;

                    for (x, pixel) in row.iter_mut().enumerate() {
                        // agregar un random chance de literalmente no renderizar nada
                        if moving && rng.random_range(0..100) < RENDER_CHANCE {
                            continue;
                        }

                        // pantalla -> camara
                        let screen_x = (2.0 * x as f32 / width as f32 - 1.0)
                            * aspect_ratio
                            * perspective_scale;
                        let screen_y = (2.0 * y as f32 / height as f32 - 1.0) * perspective_scale;

                        let camera_direction = Vector3::new(screen_x, screen_y, -1.0).normalize();
                        let world_direction = camera.basis_change(&camera_direction).normalize();

                        *pixel = cast_ray(
                            &camera.eye,
                            &world_direction,
                            objects,
                            lights,
                            textures,
                            skybox,
                            0,
                        );
                    }
                }
            });
        }
    });

    for (y, row) in pixels.chunks(width).enumerate() {
        for (x, color) in row.iter().copied().enumerate() {
            fb.set_pixel_color(x as u32, y as u32, color);
        }
    }
}

fn main() {
    let window_width = 400;
    let window_height = 300;
    let grass_island_center = Vector3::new(24.0, 0.0, 24.0);

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("3D Raytracer :D")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut framebuffer = Framebuffer::new(window_width as u32, window_height as u32);

    let material_library = load_materials("./assets/textures/")
        .unwrap_or_else(|error| panic!("Could not load materials: {error}"));

    let mut objects = generate_island(
        grass_island_center,
        ISLAND_RADIUS,
        ISLAND_RADIUS,
        ISLAND_HEIGHT,
        BLOCK_SIZE,
        material_library.materials["grass"],
        material_library.materials["dirt"],
    );

    replace_top_layer_with_river(
        &mut objects,
        grass_island_center,
        ISLAND_RADIUS,
        ISLAND_RADIUS,
        BLOCK_SIZE,
        material_library.materials["water"],
    );

    let castle_origin = grass_island_center + Vector3::new(6.0 * BLOCK_SIZE, 0.0, 5.0 * BLOCK_SIZE);
    objects.extend(generate_castle(
        castle_origin,
        BLOCK_SIZE,
        &material_library.materials,
    ));

    let tree_ground = grass_island_center + Vector3::new(2.0 * BLOCK_SIZE, 0.0, 7.0 * BLOCK_SIZE);
    objects.extend(generate_tree(
        tree_ground,
        BLOCK_SIZE,
        &material_library.materials,
    ));

    let island_midpoint = grass_island_center
        + Vector3::new(
            (ISLAND_RADIUS - 1) as f32 * BLOCK_SIZE * 0.5,
            0.0,
            (ISLAND_RADIUS - 1) as f32 * BLOCK_SIZE * 0.5,
        );
    let mut camera = Camera::new(
        island_midpoint + Vector3::new(0.0, -18.0, -60.0), // eye
        island_midpoint + Vector3::new(0.0, -9.0, 0.0),    // center
        Vector3::new(0.0, 1.0, 0.0),                       // up, perpendicular a center
    );

    let lantern_center = castle_lantern_center(castle_origin, BLOCK_SIZE);
    let far_left_back_light = grass_island_center
        + Vector3::new(
            (ISLAND_RADIUS + 2) as f32 * BLOCK_SIZE,
            -4.0 * BLOCK_SIZE,
            (ISLAND_RADIUS + 2) as f32 * BLOCK_SIZE,
        );
    let close_left_light =
        grass_island_center - Vector3::new(5.0 * BLOCK_SIZE, 2.0 * BLOCK_SIZE, 5.0 * BLOCK_SIZE);
    let lights = [
        Light {
            pos: lantern_center,
            intensity: 2.0,
            source_radius: 1.5,
        },
        Light {
            pos: far_left_back_light,
            intensity: 1.4,
            source_radius: 0.0,
        },
        Light {
            pos: close_left_light,
            intensity: 1.7,
            source_radius: 0.0,
        },
    ];

    let rotation_speed = PI / 50.0;
    let movement_speed = 1.25;
    let zoom_speed = 1.0;

    while !window.window_should_close() {
        framebuffer.clear();

        // para mover es yaw, pitch (x, y)
        if window.is_key_down(KeyboardKey::KEY_LEFT) {
            camera.rotate(-rotation_speed, 0.0);
        }

        if window.is_key_down(KeyboardKey::KEY_RIGHT) {
            camera.rotate(rotation_speed, 0.0);
        }

        if window.is_key_down(KeyboardKey::KEY_UP) {
            camera.rotate(0.0, -rotation_speed);
        }

        if window.is_key_down(KeyboardKey::KEY_DOWN) {
            camera.rotate(0.0, rotation_speed);
        }

        if window.is_key_down(KeyboardKey::KEY_W) {
            camera.move_forward(movement_speed);
        }
        if window.is_key_down(KeyboardKey::KEY_S) {
            camera.move_forward(-movement_speed);
        }
        if window.is_key_down(KeyboardKey::KEY_A) {
            camera.move_right(-movement_speed);
        }
        if window.is_key_down(KeyboardKey::KEY_D) {
            camera.move_right(movement_speed);
        }
        if window.is_key_down(KeyboardKey::KEY_SPACE) {
            camera.move_up(-movement_speed);
        }
        if window.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) {
            camera.move_up(movement_speed);
        }

        if window.is_key_down(KeyboardKey::KEY_Q) {
            camera.zoom(zoom_speed);
        }
        if window.is_key_down(KeyboardKey::KEY_E) {
            camera.zoom(-zoom_speed);
        }

        let moving = window.is_key_down(KeyboardKey::KEY_LEFT)
            || window.is_key_down(KeyboardKey::KEY_RIGHT)
            || window.is_key_down(KeyboardKey::KEY_UP)
            || window.is_key_down(KeyboardKey::KEY_DOWN)
            || window.is_key_down(KeyboardKey::KEY_W)
            || window.is_key_down(KeyboardKey::KEY_S)
            || window.is_key_down(KeyboardKey::KEY_A)
            || window.is_key_down(KeyboardKey::KEY_D)
            || window.is_key_down(KeyboardKey::KEY_SPACE)
            || window.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
            || window.is_key_down(KeyboardKey::KEY_Q)
            || window.is_key_down(KeyboardKey::KEY_E);

        draw(
            &mut framebuffer,
            &objects,
            &camera,
            &lights,
            &material_library.textures,
            material_library.skybox,
            &moving,
        );

        thread::sleep(Duration::from_millis(MS));

        framebuffer.swap_buffers(&mut window, &raylib_thread);
    }
}
