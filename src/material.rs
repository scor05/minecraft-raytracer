use crate::ray_intersect::SurfaceFace;
use crate::textures::{TextureId, TextureStore};
use raylib::prelude::Color;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub struct FaceTextures {
    pub top: TextureId,
    pub bottom: TextureId,
    pub front: TextureId,
    pub back: TextureId,
    pub left: TextureId,
    pub right: TextureId,
}

impl FaceTextures {
    pub fn all(texture: TextureId) -> Self {
        Self {
            top: texture,
            bottom: texture,
            front: texture,
            back: texture,
            left: texture,
            right: texture,
        }
    }

    pub fn top_side_bottom(top: TextureId, side: TextureId, bottom: TextureId) -> Self {
        Self {
            top: bottom,
            bottom: top,
            front: side,
            back: side,
            left: side,
            right: side,
        }
    }

    pub fn front_and_sides(front: TextureId, side: TextureId) -> Self {
        Self {
            top: side,
            bottom: side,
            front,
            back: side,
            left: side,
            right: side,
        }
    }

    pub fn for_face(self, face: SurfaceFace) -> TextureId {
        match face {
            SurfaceFace::Top => self.top,
            SurfaceFace::Bottom => self.bottom,
            SurfaceFace::Front => self.front,
            SurfaceFace::Back => self.back,
            SurfaceFace::Left => self.left,
            SurfaceFace::Right => self.right,
            SurfaceFace::Curved => self.front,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Material {
    pub diffuse: Color,
    pub textures: FaceTextures,
    // Pesos de color difuso, brillo especular, reflexión y transparencia.
    pub albedo: [f32; 4],
    pub specular: f32,
    pub refraction_index: f32,
}

pub struct MaterialLibrary {
    pub materials: HashMap<&'static str, Material>,
    pub textures: TextureStore,
}

fn material(
    textures: FaceTextures,
    albedo: [f32; 4],
    specular: f32,
    refraction_index: f32,
) -> Material {
    Material {
        diffuse: Color::WHITE,
        textures,
        albedo,
        specular,
        refraction_index,
    }
}

pub fn load_materials(path: impl AsRef<Path>) -> Result<MaterialLibrary, String> {
    let path = path.as_ref();
    let mut textures = TextureStore::new();

    let dirt = textures.load(path.join("dirt.png"))?;
    let grass_top = textures.load(path.join("grass_top.png"))?;
    let grass_side = textures.load(path.join("grass_side.png"))?;
    let log_top = textures.load(path.join("log_top.png"))?;
    let log_side = textures.load(path.join("log_side.png"))?;
    let barrel_top = textures.load(path.join("barrel_top.png"))?;
    let barrel_side = textures.load(path.join("barrel_side.png"))?;
    let chest_front = textures.load(path.join("chest.png"))?;
    let chest_side = textures.load(path.join("chest_side.png"))?;
    let lantern_top = textures.load(path.join("lantern_topbottom.png"))?;
    let lantern_side = textures.load(path.join("lantern_sides.png"))?;

    let mut materials = HashMap::new();
    materials.insert(
        "grass",
        material(
            FaceTextures::top_side_bottom(grass_top, grass_side, dirt),
            [0.85, 0.15, 0.0, 0.0],
            16.0,
            1.52,
        ),
    );
    materials.insert(
        "dirt",
        material(FaceTextures::all(dirt), [0.9, 0.1, 0.0, 0.0], 8.0, 1.5),
    );
    materials.insert(
        "log",
        material(
            FaceTextures::top_side_bottom(log_top, log_side, log_top),
            [0.9, 0.1, 0.0, 0.0],
            12.0,
            1.53,
        ),
    );
    materials.insert(
        "barrel",
        material(
            FaceTextures::top_side_bottom(barrel_top, barrel_side, barrel_top),
            [0.85, 0.15, 0.03, 0.0],
            24.0,
            1.5,
        ),
    );
    materials.insert(
        "chest",
        material(
            FaceTextures::front_and_sides(chest_front, chest_side),
            [0.85, 0.15, 0.03, 0.0],
            24.0,
            1.5,
        ),
    );
    materials.insert(
        "lantern",
        material(
            FaceTextures::top_side_bottom(lantern_top, lantern_side, lantern_top),
            [0.8, 0.2, 0.08, 0.0],
            32.0,
            1.5,
        ),
    );

    let uniform_materials = [
        (
            "cobblestone",
            "cobblestone.png",
            [0.9, 0.1, 0.0, 0.0],
            8.0,
            1.52,
        ),
        (
            "crying_obsidian",
            "crying_obsidian.png",
            [0.55, 0.2, 0.35, 0.0],
            96.0,
            1.55,
        ),
        ("glass", "glass.png", [0.15, 0.1, 0.1, 0.85], 128.0, 1.5),
        (
            "glowstone",
            "glowstone.png",
            [0.9, 0.1, 0.0, 0.0],
            16.0,
            1.5,
        ),
        ("gold", "gold.png", [0.45, 0.25, 0.65, 0.0], 128.0, 1.47),
        (
            "mossy_cobble",
            "mossy_cobble.png",
            [0.9, 0.1, 0.0, 0.0],
            8.0,
            1.52,
        ),
        (
            "netherrack",
            "netherrack.png",
            [0.9, 0.1, 0.0, 0.0],
            12.0,
            1.5,
        ),
        ("oak_log", "oak_log.png", [0.9, 0.1, 0.0, 0.0], 12.0, 1.53),
        (
            "obsidian",
            "obsidian.png",
            [0.6, 0.15, 0.3, 0.0],
            96.0,
            1.55,
        ),
        (
            "red_carpet",
            "red_carpet.png",
            [0.9, 0.1, 0.0, 0.0],
            8.0,
            1.5,
        ),
        ("stone", "stone.png", [0.9, 0.1, 0.0, 0.0], 12.0, 1.52),
        (
            "stone_brick",
            "stone_brick.png",
            [0.9, 0.1, 0.0, 0.0],
            12.0,
            1.52,
        ),
    ];

    for (name, filename, albedo, specular, refraction_index) in uniform_materials {
        let texture = textures.load(path.join(filename))?;
        materials.insert(
            name,
            material(
                FaceTextures::all(texture),
                albedo,
                specular,
                refraction_index,
            ),
        );
    }

    Ok(MaterialLibrary {
        materials,
        textures,
    })
}
