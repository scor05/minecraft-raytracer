use raylib::prelude::{Color, Image, Vector2};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureId(usize);

struct CpuTexture {
    width: usize,
    height: usize,
    pixels: Vec<Color>,
}

pub struct TextureStore {
    textures: Vec<CpuTexture>,
    loaded_paths: HashMap<PathBuf, TextureId>,
}

impl TextureStore {
    pub fn new() -> Self {
        Self {
            textures: Vec::new(),
            loaded_paths: HashMap::new(),
        }
    }

    pub fn load(&mut self, path: impl AsRef<Path>) -> Result<TextureId, String> {
        let path = path.as_ref();
        if let Some(texture_id) = self.loaded_paths.get(path) {
            return Ok(*texture_id);
        }

        let path_text = path
            .to_str()
            .ok_or_else(|| format!("Texture path is not valid UTF-8: {}", path.display()))?;
        let image = Image::load_image(path_text)
            .map_err(|error| format!("Failed to load {}: {error}", path.display()))?;

        let width = image.width as usize;
        let height = image.height as usize;
        if width == 0 || height == 0 {
            return Err(format!("Texture has no pixels: {}", path.display()));
        }

        let pixels = image.get_image_data().iter().copied().collect();
        let texture_id = TextureId(self.textures.len());
        self.textures.push(CpuTexture {
            width,
            height,
            pixels,
        });
        self.loaded_paths.insert(path.to_path_buf(), texture_id);
        Ok(texture_id)
    }

    pub fn sample(&self, texture_id: TextureId, uv: Vector2) -> Color {
        let Some(texture) = self.textures.get(texture_id.0) else {
            return Color::MAGENTA;
        };

        let u = uv.x.clamp(0.0, 1.0);
        let v = uv.y.clamp(0.0, 1.0);
        let x = (u * (texture.width - 1) as f32).round() as usize;
        let y = (v * (texture.height - 1) as f32).round() as usize;

        texture.pixels[y * texture.width + x]
    }
}
