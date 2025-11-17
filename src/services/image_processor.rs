use crate::error::ImageResizeError;
use image::{DynamicImage, ImageFormat, imageops::FilterType};
use std::io::Cursor;

pub struct ImageProcessor {
    max_width: u32,
    max_height: u32,
    max_file_size: u64,
}

impl ImageProcessor {
    pub fn new(max_width: u32, max_height: u32, max_file_size: u64) -> Self {
        Self {
            max_width,
            max_height,
            max_file_size,
        }
    }

    pub fn process_image(
        &self,
        image_bytes: Vec<u8>,
        original_size: u64,
    ) -> Result<ProcessResult, ImageResizeError> {
        // If original is already under size limit, return as-is
        if original_size < self.max_file_size {
            tracing::info!(
                original_size = original_size,
                max_size = self.max_file_size,
                "Image already under size limit, using original"
            );
            return Ok(ProcessResult {
                data: image_bytes,
                width: 0,
                height: 0,
                original_width: 0,
                original_height: 0,
                was_resized: false,
            });
        }

        // Decode image
        let img =
            image::load_from_memory(&image_bytes).map_err(|_| ImageResizeError::InvalidImage)?;

        let (original_width, original_height) = (img.width(), img.height());

        tracing::info!(
            original_width = original_width,
            original_height = original_height,
            original_size = original_size,
            "Image exceeds size limit, starting resize process"
        );

        // Attempt resize with progressively smaller dimensions
        let mut current_width = self.max_width.min(original_width);
        let mut current_height = self.max_height.min(original_height);
        let mut quality = 85u8;
        let mut attempts = 0u32;
        const MAX_ATTEMPTS: u32 = 10;

        while attempts < MAX_ATTEMPTS {
            attempts += 1;

            // Calculate dimensions preserving aspect ratio
            let (target_width, target_height) = calculate_dimensions(
                original_width,
                original_height,
                current_width,
                current_height,
            );

            // Resize image
            let resized = img.resize(target_width, target_height, FilterType::Triangle);

            // Encode with current quality
            let encoded = encode_image(&resized, quality)?;

            tracing::debug!(
                attempt = attempts,
                width = target_width,
                height = target_height,
                quality = quality,
                size = encoded.len(),
                "Resize attempt"
            );

            if (encoded.len() as u64) < self.max_file_size {
                tracing::info!(
                    attempts = attempts,
                    final_width = target_width,
                    final_height = target_height,
                    final_size = encoded.len(),
                    quality = quality,
                    "Successfully resized image under size limit"
                );

                return Ok(ProcessResult {
                    data: encoded,
                    width: target_width,
                    height: target_height,
                    original_width,
                    original_height,
                    was_resized: true,
                });
            }

            // Reduce dimensions or quality for next attempt
            if quality > 60 {
                quality -= 5;
            } else {
                current_width = (current_width as f32 * 0.9) as u32;
                current_height = (current_height as f32 * 0.9) as u32;

                if current_width < 100 || current_height < 100 {
                    return Err(ImageResizeError::SizeExceeded {
                        attempts,
                        final_size: encoded.len() as u64,
                    });
                }
            }
        }

        Err(ImageResizeError::SizeExceeded {
            attempts,
            final_size: original_size,
        })
    }
}

pub struct ProcessResult {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub original_width: u32,
    pub original_height: u32,
    pub was_resized: bool,
}

fn calculate_dimensions(
    original_width: u32,
    original_height: u32,
    max_width: u32,
    max_height: u32,
) -> (u32, u32) {
    let width_ratio = max_width as f32 / original_width as f32;
    let height_ratio = max_height as f32 / original_height as f32;
    let ratio = width_ratio.min(height_ratio);

    if ratio >= 1.0 {
        (original_width, original_height)
    } else {
        (
            (original_width as f32 * ratio) as u32,
            (original_height as f32 * ratio) as u32,
        )
    }
}

fn encode_image(img: &DynamicImage, _quality: u8) -> Result<Vec<u8>, ImageResizeError> {
    let mut buffer = Cursor::new(Vec::new());

    img.write_to(&mut buffer, ImageFormat::Jpeg)
        .map_err(|e| ImageResizeError::ImageError(format!("Failed to encode image: {}", e)))?;

    Ok(buffer.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_dimensions_preserves_aspect_ratio() {
        let (w, h) = calculate_dimensions(1920, 1080, 400, 400);
        assert_eq!(w, 400);
        assert_eq!(h, 225); // 400 * (1080/1920) = 225

        let (w, h) = calculate_dimensions(1080, 1920, 400, 400);
        assert_eq!(w, 225);
        assert_eq!(h, 400);
    }

    #[test]
    fn test_calculate_dimensions_no_upscale() {
        let (w, h) = calculate_dimensions(100, 100, 400, 400);
        assert_eq!(w, 100);
        assert_eq!(h, 100);
    }
}
