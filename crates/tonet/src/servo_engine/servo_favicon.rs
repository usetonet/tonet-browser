//! Encode Servo [`servo::Image`] favicons as PNG for egui [`egui::Context::include_bytes`].

use image::{DynamicImage, ImageBuffer, RgbaImage};
use servo::{Image, PixelFormat};

/// Encode a raster [`Image`] as PNG bytes, or `None` if format/length is unsupported.
pub(crate) fn encode_image_as_png(img: &Image) -> Option<Vec<u8>> {
    let w = img.width;
    let h = img.height;
    let data = img.data();
    let rgba: RgbaImage = match img.format {
        PixelFormat::RGBA8 => {
            let expected = (w.checked_mul(h)?.checked_mul(4)?) as usize;
            if data.len() != expected {
                return None;
            }
            ImageBuffer::from_raw(w, h, data.to_vec())?
        }
        PixelFormat::BGRA8 => {
            let expected = (w.checked_mul(h)?.checked_mul(4)?) as usize;
            if data.len() != expected {
                return None;
            }
            let mut v = Vec::with_capacity(data.len());
            for c in data.chunks_exact(4) {
                v.extend_from_slice(&[c[2], c[1], c[0], c[3]]);
            }
            ImageBuffer::from_raw(w, h, v)?
        }
        PixelFormat::RGB8 => {
            let expected = (w.checked_mul(h)?.checked_mul(3)?) as usize;
            if data.len() != expected {
                return None;
            }
            let mut v = Vec::with_capacity((w as usize).checked_mul(h as usize)?.checked_mul(4)?);
            for c in data.chunks_exact(3) {
                v.extend_from_slice(&[c[0], c[1], c[2], 255]);
            }
            ImageBuffer::from_raw(w, h, v)?
        }
        PixelFormat::K8 | PixelFormat::KA8 => {
            return None;
        }
    };
    let mut out = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut out);
    DynamicImage::ImageRgba8(rgba)
        .write_to(&mut cursor, image::ImageFormat::Png)
        .ok()?;
    (!out.is_empty()).then_some(out)
}
