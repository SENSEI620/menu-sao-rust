//! Foto da tela onde o mouse está, para o fundo desfocado do menu.
//! (Janelas transparentes não conseguem desfocar a área de trabalho, então o
//! programa tira uma foto antes de abrir e o menu desfoca essa foto.)
//!
//! A foto é reduzida pela metade: o fundo fica desfocado de qualquer jeito,
//! e assim ela chega à tela bem mais rápido.

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use image::codecs::jpeg::JpegEncoder;
use image::imageops::{resize, FilterType};
use image::{DynamicImage, RgbaImage};

pub struct Shot {
    pub data_url: String,
    pub width: u32,
    pub height: u32,
}

pub fn monitor_at(x: i32, y: i32) -> Result<Shot, String> {
    let monitor = xcap::Monitor::from_point(x, y).map_err(|e| e.to_string())?;
    let img = monitor.capture_image().map_err(|e| e.to_string())?;
    let (w, h) = (img.width(), img.height());
    let rgba = RgbaImage::from_raw(w, h, img.into_raw()).ok_or("imagem inválida")?;
    encode(rgba)
}

/// Reduz pela metade e codifica em JPEG (qualidade 70) como data URL.
pub fn encode(rgba: RgbaImage) -> Result<Shot, String> {
    let (w, h) = (rgba.width(), rgba.height());
    let small = resize(&rgba, (w / 2).max(1), (h / 2).max(1), FilterType::Triangle);
    let rgb = DynamicImage::ImageRgba8(small).to_rgb8();
    let mut buf = Vec::with_capacity(256 * 1024);
    JpegEncoder::new_with_quality(&mut buf, 70)
        .encode_image(&rgb)
        .map_err(|e| e.to_string())?;
    Ok(Shot { data_url: format!("data:image/jpeg;base64,{}", STANDARD.encode(&buf)), width: w, height: h })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_a_data_url_at_half_size() {
        let img = RgbaImage::from_pixel(640, 360, image::Rgba([30, 80, 140, 255]));
        let shot = encode(img).unwrap();
        assert!(shot.data_url.starts_with("data:image/jpeg;base64,"));
        assert_eq!((shot.width, shot.height), (640, 360));
        let b64 = shot.data_url.trim_start_matches("data:image/jpeg;base64,");
        let bytes = STANDARD.decode(b64).unwrap();
        let back = image::load_from_memory(&bytes).unwrap();
        assert_eq!((back.width(), back.height()), (320, 180));
    }
}
