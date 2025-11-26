// Texture assets decoded at runtime

use image::GenericImageView;

/// Raw RGBA texture data that can be uploaded to the GPU.
pub struct TextureAsset {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

/// Load the Companion Cube's base color texture from the embedded JPEG.
pub fn companion_cube_base_color_texture() -> Result<TextureAsset, String> {
    decode_to_rgba(include_bytes!(
        "gbl/cube/textures/DefaultMaterial_baseColor.jpeg"
    ))
}

fn decode_to_rgba(bytes: &[u8]) -> Result<TextureAsset, String> {
    let image = image::load_from_memory(bytes)
        .map_err(|e| format!("Failed to decode texture: {e}"))?;
    let (width, height) = image.dimensions();
    let rgba = image.to_rgba8();

    Ok(TextureAsset {
        width,
        height,
        pixels: rgba.into_raw(),
    })
}

