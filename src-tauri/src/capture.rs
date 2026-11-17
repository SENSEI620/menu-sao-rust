//! Foto da tela onde o mouse está, para o fundo do menu.
//!
//! O fundo é sempre desfocado, então o Rust já entrega a foto PRONTA: reduzida a 1/4, levemente
//! desfocada, mais escura e com cores mais vivas. Assim a tela não precisa aplicar nenhum filtro
//! pesado sobre uma imagem do tamanho do monitor, e a foto trafega com ~20 KB em vez de centenas.

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use image::codecs::jpeg::JpegEncoder;
use image::imageops::{blur, resize, FilterType};
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

const BRIGHTNESS: f32 = 0.74;
const SATURATION: f32 = 1.25;

/// 1/4 do tamanho + desfoque leve + brilho/saturação, em JPEG como data URL.
pub fn encode(rgba: RgbaImage) -> Result<Shot, String> {
    let (w, h) = (rgba.width(), rgba.height());
    let small = resize(&rgba, (w / 4).max(1), (h / 4).max(1), FilterType::Triangle);
    let soft = blur(&small, 1.6);
    let mut rgb = DynamicImage::ImageRgba8(soft).to_rgb8();
    for px in rgb.pixels_mut() {
        let [r, g, b] = px.0.map(f32::from);
        let l = 0.299 * r + 0.587 * g + 0.114 * b;
        let adj = |c: f32| ((l + (c - l) * SATURATION) * BRIGHTNESS).clamp(0.0, 255.0) as u8;
        px.0 = [adj(r), adj(g), adj(b)];
    }
    let mut buf = Vec::with_capacity(32 * 1024);
    JpegEncoder::new_with_quality(&mut buf, 82).encode_image(&rgb).map_err(|e| e.to_string())?;
    Ok(Shot { data_url: format!("data:image/jpeg;base64,{}", STANDARD.encode(&buf)), width: w, height: h })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_url_is_a_quarter_size_and_small() {
        let img = RgbaImage::from_pixel(1920, 1080, image::Rgba([30, 80, 140, 255]));
        let shot = encode(img).unwrap();
        assert!(shot.data_url.starts_with("data:image/jpeg;base64,"));
        assert_eq!((shot.width, shot.height), (1920, 1080));
        assert!(shot.data_url.len() < 30_000, "foto grande demais: {}", shot.data_url.len());
        let bytes = STANDARD.decode(shot.data_url.trim_start_matches("data:image/jpeg;base64,")).unwrap();
        let back = image::load_from_memory(&bytes).unwrap();
        assert_eq!((back.width(), back.height()), (480, 270));
    }

    #[test]
    fn darkens_and_keeps_colors_vivid() {
        let img = RgbaImage::from_pixel(64, 64, image::Rgba([200, 100, 50, 255]));
        let shot = encode(img).unwrap();
        let bytes = STANDARD.decode(shot.data_url.trim_start_matches("data:image/jpeg;base64,")).unwrap();
        let px = image::load_from_memory(&bytes).unwrap().to_rgb8().get_pixel(8, 8).0;
        assert!(px[0] < 200 && px[0] > px[1] && px[1] > px[2], "{px:?}");
    }
}
