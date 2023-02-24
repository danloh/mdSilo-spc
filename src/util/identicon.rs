//! ## Default avatar generator  
//! forked from: [identicon](https://github.com/dgraham/identicon) ,
//! [LICENSE: MIT](https://github.com/dgraham/identicon/blob/master/LICENSE)

use image::{ImageBuffer, Rgb, RgbImage};
use std::slice::Iter;

struct HSL {
  hue: f32,
  sat: f32,
  lum: f32,
}

impl HSL {
  pub fn new(hue: f32, sat: f32, lum: f32) -> HSL {
    HSL { hue, sat, lum }
  }

  // http://www.w3.org/TR/css3-color/#hsl-color
  pub fn rgb(&self) -> Rgb<u8> {
    let hue = self.hue / 360.0;
    let sat = self.sat / 100.0;
    let lum = self.lum / 100.0;

    let b = if lum <= 0.5 {
      lum * (sat + 1.0)
    } else {
      lum + sat - lum * sat
    };
    let a = lum * 2.0 - b;

    let r = HSL::hue_to_rgb(a, b, hue + 1.0 / 3.0);
    let g = HSL::hue_to_rgb(a, b, hue);
    let b = HSL::hue_to_rgb(a, b, hue - 1.0 / 3.0);

    Rgb([
      (r * 255.0).round() as u8,
      (g * 255.0).round() as u8,
      (b * 255.0).round() as u8,
    ])
  }

  fn hue_to_rgb(a: f32, b: f32, hue: f32) -> f32 {
    let h = if hue < 0.0 {
      hue + 1.0
    } else if hue > 1.0 {
      hue - 1.0
    } else {
      hue
    };

    if h < 1.0 / 6.0 {
      return a + (b - a) * 6.0 * h;
    }

    if h < 1.0 / 2.0 {
      return b;
    }

    if h < 2.0 / 3.0 {
      return a + (b - a) * (2.0 / 3.0 - h) * 6.0;
    }

    a
  }
}

struct Nibbler<'a> {
  byte: Option<u8>,
  bytes: Iter<'a, u8>,
}

impl<'a> Nibbler<'a> {
  pub fn new(bytes: &[u8]) -> Nibbler {
    Nibbler {
      bytes: bytes.iter(),
      byte: None,
    }
  }
}

impl<'a> Iterator for Nibbler<'a> {
  type Item = u8;

  fn next(&mut self) -> Option<u8> {
    match self.byte {
      Some(value) => {
        self.byte = None;
        Some(value)
      }
      None => match self.bytes.next() {
        Some(value) => {
          let hi = *value & 0xf0;
          let lo = *value & 0x0f;
          self.byte = Some(lo);
          Some(hi >> 4)
        }
        None => None,
      },
    }
  }
}

pub struct Identicon<'a> {
  source: &'a [u8],
  size: u32,
}

impl<'a> Identicon<'a> {
  pub fn new(source: &[u8], size: u32) -> Identicon {
    Identicon { source, size }
  }

  // https://processing.org/reference/map_.html
  fn map(value: u32, vmin: u32, vmax: u32, dmin: u32, dmax: u32) -> f32 {
    ((value - vmin) * (dmax - dmin)) as f32 / ((vmax - vmin) + dmin) as f32
  }

  fn foreground(&self) -> Rgb<u8> {
    // Use last 28 bits to determine HSL values.
    let h1 = (self.source[12] as u16 & 0x0f) << 8;
    let h2 = self.source[13] as u16;

    let h = (h1 | h2) as u32;
    let s = self.source[14] as u32;
    let l = self.source[15] as u32;

    let hue = Identicon::map(h, 0, 4095, 0, 360);
    let sat = Identicon::map(s, 0, 255, 0, 20);
    let lum = Identicon::map(l, 0, 255, 0, 20);

    HSL::new(hue, 65.0 - sat, 75.0 - lum).rgb()
  }

  fn rect(image: &mut RgbImage, x0: u32, y0: u32, x1: u32, y1: u32, color: Rgb<u8>) {
    for x in x0..x1 {
      for y in y0..y1 {
        image.put_pixel(x, y, color);
      }
    }
  }

  fn pixels(&self) -> [bool; 25] {
    let mut nibbles = Nibbler::new(self.source).map(|x| x % 2 == 0);
    let mut pixels = [false; 25];
    for col in (0..3).rev() {
      for row in 0..5 {
        let ix = col + (row * 5);
        let mirror_col = 4 - col;
        let mirror_ix = mirror_col + (row * 5);
        let paint = nibbles.next().unwrap_or(false);
        pixels[ix] = paint;
        pixels[mirror_ix] = paint;
      }
    }
    pixels
  }

  pub fn image(&self) -> RgbImage {
    let pixel_size = 70;
    let sprite_size = 5;
    let margin = pixel_size / 2;
    let background = Rgb([240, 240, 240]);
    let foreground = self.foreground();

    let mut image: RgbImage =
      ImageBuffer::from_pixel(self.size, self.size, background);

    for (row, pix) in self.pixels().chunks(sprite_size).enumerate() {
      for (col, painted) in pix.iter().enumerate() {
        if *painted {
          let x = col * pixel_size;
          let y = row * pixel_size;
          Identicon::rect(
            &mut image,
            (x + margin) as u32,
            (y + margin) as u32,
            (x + pixel_size + margin) as u32,
            (y + pixel_size + margin) as u32,
            foreground,
          );
        }
      }
    }

    image
  }
}

#[cfg(test)]
mod tests {
  use super::Nibbler;
  use super::HSL;
  use image::Rgb;

  #[test]
  fn it_converts_black() {
    let black = Rgb([0, 0, 0]);
    let rgb = HSL::new(0.0, 0.0, 0.0).rgb();
    assert_eq!(black, rgb);
  }

  #[test]
  fn it_converts_white() {
    let white = Rgb([255, 255, 255]);
    let rgb = HSL::new(0.0, 0.0, 100.0).rgb();
    assert_eq!(white, rgb);
  }

  #[test]
  fn it_converts_red() {
    let red = Rgb([255, 0, 0]);
    let rgb = HSL::new(0.0, 100.0, 50.0).rgb();
    assert_eq!(red, rgb);
  }

  #[test]
  fn it_converts_green() {
    let green = Rgb([0, 255, 0]);
    let rgb = HSL::new(120.0, 100.0, 50.0).rgb();
    assert_eq!(green, rgb);
  }

  #[test]
  fn it_converts_blue() {
    let blue = Rgb([0, 0, 255]);
    let rgb = HSL::new(240.0, 100.0, 50.0).rgb();
    assert_eq!(blue, rgb);
  }

  #[test]
  fn it_iterates_nibbles() {
    let bytes = vec![0x2a];
    let nibbles = Nibbler::new(&bytes);
    let result: Vec<u8> = nibbles.collect();
    assert_eq!(vec![0x02, 0x0a], result);
  }
}
