mod camera;
mod cube;
mod framebuffer;
mod light;
mod material;
mod procedural;
mod ray_intersect;
mod sphere;

use crate::camera::Camera;
use crate::framebuffer::Framebuffer;
use crate::light::Light;
use crate::material::Material;
use crate::ray_intersect::{Intersect, RayIntersect};
use procedural::*;
use rand::RngExt;
use raylib::prelude::*;
use std::f32::consts::PI;
use std::thread;
use std::time::Duration;

const FOV: f32 = PI / 2.0;
const FPS: u64 = 500;
const MS: u64 = 1000 / FPS;
const SHADOW_BIAS: f32 = 1e-4; // es qué tanto sumarle a los vectores que entran de la esfera
// viene de problemas de redondeo
const MAX_REFLECTIONS: u8 = 2;
const TERRAIN_SIZE: i32 = 5;

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
            if shadow_intersect.distance < light_distance {
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
    light: &Light,
    depth: u32, // depth de recursión
) -> Color {
    let background_color = Color::new(20, 20, 20, 255);
    let mut closest_intersection: Option<Intersect> = None;

    if depth > MAX_REFLECTIONS as u32 {
        return background_color;
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
        return background_color;
    };

    let light_dir = (light.pos - intersect.point).normalize();

    let shadow_intensity = cast_shadow(&intersect, light, objects);
    let light_intensity = 1.0 - shadow_intensity;

    // diffuse
    let diffuse_intensity = intersect.normal.dot(light_dir).max(0.0);
    let material_color = Vector3::new(
        intersect.material.diffuse.r as f32,
        intersect.material.diffuse.g as f32,
        intersect.material.diffuse.b as f32,
    );
    let diffuse =
        material_color * diffuse_intensity * intersect.material.albedo[0] * light_intensity;

    // refleccion
    let reflect_dir = reflect(&-light_dir, &intersect.normal);
    let view_dir = (*ray_origin - intersect.point).normalize();

    // Igual de teoría de Phong: S = V · R, con V s
    let specular_intensity = view_dir
        .dot(reflect_dir)
        .max(0.0)
        .powf(intersect.material.specular)
        * intersect.material.albedo[1];

    // la luz se considera como blanca (1,1,1) en vector normalizado
    // se multiplica por 255 para que esté en rango de alfa de RGB de u8 (0-255)

    let mut reflect_color = Vector3::zero();
    let reflectivity = intersect.material.albedo[2];
    if reflectivity > 0.0 {
        let reflect_dir = reflect(&ray_direction, &intersect.normal);
        let reflect_origin = offset_origin(&intersect, &reflect_dir);
        reflect_color = color_to_vector(cast_ray(
            &reflect_origin,
            &reflect_dir,
            objects,
            light,
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
            // Total internal reflection: no refracted ray is possible.
            reflect(ray_direction, &intersect.normal).normalize()
        };

        let refract_origin = offset_origin(&intersect, &refract_dir);
        color_to_vector(cast_ray(
            &refract_origin,
            &refract_dir,
            objects,
            light,
            depth + 1,
        ))
    } else {
        Vector3::zero()
    };

    let specular = Vector3::new(1.0, 1.0, 1.0) * 255.0 * specular_intensity * light_intensity; // luz * intensidad
    let reflection = reflect_color * reflectivity;
    let refraction = refract_color * transparency;

    let color = diffuse + specular + reflection + refraction;

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
    light: &Light,
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
                        if moving && rng.random_range(0..100) < 95 {
                            continue;
                        }

                        // pantalla -> camara
                        let screen_x = (2.0 * x as f32 / width as f32 - 1.0)
                            * aspect_ratio
                            * perspective_scale;
                        let screen_y = (2.0 * y as f32 / height as f32 - 1.0) * perspective_scale;

                        let camera_direction = Vector3::new(screen_x, screen_y, -1.0).normalize();
                        let world_direction = camera.basis_change(&camera_direction).normalize();

                        *pixel = cast_ray(&camera.eye, &world_direction, objects, light, 0);
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
    let window_width = 720;
    let window_height = 720;

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("3D Raytracer :D")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut framebuffer = Framebuffer::new(window_width as u32, window_height as u32);

    let rubber = Material {
        diffuse: Color::new(255, 0, 0, 255),
        albedo: [0.40, 0.60, 0.02, 0.0],
        specular: 10.0,
        refraction_index: 1.52,
    };

    let steel = Material {
        diffuse: Color::new(180, 180, 180, 255),
        albedo: [0.99, 0.01, 0.90, 0.0],
        specular: 100.0,
        refraction_index: 2.50,
    };

    let colored_material =
        |r, g, b, specular, reflectivity, transparency, refraction_index| Material {
            diffuse: Color::new(r, g, b, 255),
            albedo: [0.75, 0.25, reflectivity, transparency],
            specular,
            refraction_index,
        };

    // Approximate visible-light IORs for representative real materials.
    // White is treated as moissanite, giving it the highest IOR in the scene.
    let white = colored_material(255, 255, 255, 240.0, 0.90, 0.80, 2.65);
    let orange = colored_material(255, 125, 35, 24.0, 0.08, 0.08, 1.50);
    let gold = colored_material(255, 205, 45, 48.0, 0.80, 0.0, 1.55);
    let emerald = colored_material(35, 205, 105, 36.0, 0.10, 0.55, 1.58);
    let cyan = colored_material(35, 205, 225, 64.0, 0.55, 0.75, 1.33);
    let blue = colored_material(45, 95, 235, 72.0, 0.35, 0.35, 1.50);
    let violet = colored_material(135, 65, 225, 56.0, 0.75, 0.30, 1.54);
    let magenta = colored_material(235, 55, 175, 40.0, 0.12, 0.15, 1.49);
    let pink = colored_material(255, 125, 165, 28.0, 0.45, 0.25, 1.46);
    let lime = colored_material(155, 225, 55, 20.0, 0.0, 0.10, 1.52);

    /*
    let objects = [
        Sphere {
            center: Vector3::new(-3.2, -2.0, 6.5),
            radius: 1.0,
            material: rubber,
        },
        Sphere {
            center: Vector3::new(-0.5, -2.4, 7.5),
            radius: 1.1,
            material: white,
        },
        Sphere {
            center: Vector3::new(2.5, -2.0, 7.0),
            radius: 1.2,
            material: gold,
        },
        Sphere {
            center: Vector3::new(-0.8, 0.5, 6.2),
            radius: 1.0,
            material: magenta,
        },
        Sphere {
            center: Vector3::new(2.0, 0.5, 8.2),
            radius: 1.4,
            material: emerald,
        },
    ];
    */

    let objects = generate_terrain(TERRAIN_SIZE, TERRAIN_SIZE);

    let mut camera = Camera::new(
        Vector3::new(0.0, 0.0, 0.0), // eye
        Vector3::new(0.0, 0.0, 5.0), // hacia dónde inicia viendo, center
        Vector3::new(0.0, 1.0, 0.0), // up, perpendicular a center
    );

    let light = Light {
        // Frente a las esferas y ligeramente descentrada para que ambas tengan
        // una cara iluminada y una zona de sombra visible.
        pos: Vector3::new(5.0, -5.0, 5.0),
    };

    let rotation_speed = PI / 100.0;
    let movement_speed = 0.5;
    let zoom_speed = 0.3;

    while !window.window_should_close() {
        framebuffer.clear();

        // para mover es yaw, pitch (x, y)
        if window.is_key_down(KeyboardKey::KEY_LEFT) {
            camera.orbit(rotation_speed, 0.0);
        }

        if window.is_key_down(KeyboardKey::KEY_RIGHT) {
            camera.orbit(-rotation_speed, 0.0);
        }

        if window.is_key_down(KeyboardKey::KEY_UP) {
            camera.orbit(0.0, rotation_speed);
        }

        if window.is_key_down(KeyboardKey::KEY_DOWN) {
            camera.orbit(0.0, -rotation_speed);
        }

        // Translate the camera without changing its viewing direction.
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

        // Zoom changes the distance between the eye and focal point.
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

        draw(&mut framebuffer, &objects, &camera, &light, &moving);

        thread::sleep(Duration::from_millis(MS));

        framebuffer.swap_buffers(&mut window, &raylib_thread);
    }
}
