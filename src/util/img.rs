//! ## compress image  
//! Inspired by [image_compressor](https://github.com/altair823/image_compressor/blob/main/src/compressor.rs), MIT license.

use axum::body::Bytes;
use image::{imageops::FilterType, ImageFormat};
use img_parts::{DynImage, ImageEXIF};
use mozjpeg::{ColorSpace, Compress, ScanMode};

pub(crate) fn process_img(data: Bytes, format: ImageFormat) -> Option<Vec<u8>> {
  match format {
    ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::WebP => {
      if let Ok(Some(mut img)) = DynImage::from_bytes(data) {
        img.set_exif(None);
        let img_noexif = img.encoder().bytes();
        let dyn_img = image::load_from_memory_with_format(&img_noexif, format)
          .unwrap_or_default();
        let factor = Factor::get(img_noexif.len());

        // resize
        let width = (dyn_img.width() as f32 * factor.size_ratio) as u32;
        let height = (dyn_img.width() as f32 * factor.size_ratio) as u32;
        let resized_img = dyn_img.resize(width, height, FilterType::Lanczos3);

        // compress
        let mut comp = Compress::new(ColorSpace::JCS_RGB);
        comp.set_scan_optimization_mode(ScanMode::Auto);
        comp.set_quality(factor.quality);

        let target_width = resized_img.width() as usize;
        let target_height = resized_img.height() as usize;
        comp.set_size(target_width, target_height);

        comp.set_mem_dest();
        comp.set_optimize_scans(true);
        comp.start_compress();

        let mut line: usize = 0;
        let resized_img_data = resized_img.into_rgb8().into_vec();
        loop {
          if line > target_height - 1 {
            break;
          }
          let idx = line * target_width * 3..(line + 1) * target_width * 3;
          comp.write_scanlines(&resized_img_data[idx]);
          line += 1;
        }
        comp.finish_compress();

        if let Ok(comp) = comp.data_to_vec() {
          Some(comp)
        } else {
          None
        }
      } else {
        None
      }
    }
    _ => Some(data.to_vec()),
  }
}

#[derive(Copy, Clone)]
struct Factor {
  /// Quality of the new compressed image.
  /// Values range from 0 to 100 in float.
  quality: f32,

  /// Ratio for resize the new compressed image.
  /// Values range from 0 to 1 in float.
  size_ratio: f32,
}

impl Factor {
  /// Create a new `Factor` instance.
  /// The `quality` range from 0 to 100 in float,
  /// and `size_ratio` range from 0 to 1 in float.
  ///
  /// # Panics
  ///
  /// - If the quality value is 0 or less.
  /// - If the quality value exceeds 100.
  /// - If the size ratio value is 0 or less.
  /// - If the size ratio value exceeds 1.
  fn new(quality: f32, size_ratio: f32) -> Self {
    if (quality > 0. && quality <= 100.) && (size_ratio > 0. && size_ratio <= 1.) {
      Self {
        quality,
        size_ratio,
      }
    } else {
      panic!("Wrong Factor argument!");
    }
  }

  fn get(file_size: usize) -> Factor {
    match file_size {
      file_size if file_size > 5000000 => Factor::new(60., 0.7),
      file_size if file_size > 1000000 => Factor::new(65., 0.75),
      file_size if file_size > 500000 => Factor::new(70., 0.8),
      file_size if file_size > 300000 => Factor::new(75., 0.85),
      file_size if file_size > 100000 => Factor::new(80., 0.9),
      _ => Factor::new(85., 1.0),
    }
  }
}
