use std::fmt::Debug;

use image::{DynamicImage, Pixel, Rgb, RgbImage};

use crate::color::Color;

#[derive(Clone, PartialEq)]
pub enum Texture<T> {
    Constant(T),
    Checkerboard { a: T, b: T, scale: f64 },
    Image(RgbImage),
}

pub trait FromPixel {
    fn from_pixel(pixel: &Rgb<u8>) -> Self;
}

impl<T: Copy + FromPixel> Texture<T> {
    pub fn eval(&self, &(u, v): &(f64, f64)) -> T {
        match self {
            Self::Constant(t) => *t,
            Self::Checkerboard { a, b, scale } => {
                let u = (u * scale * 2.0) as usize;
                let v = (v * scale * 2.0) as usize;
                if (u & 1) ^ (v & 1) == 0 {
                    *a
                } else {
                    *b
                }
            }
            Self::Image(image) => {
                let mut u = u.fract();
                if u < 0.0 {
                    u += 1.0;
                }

                let mut v = v.fract();
                if v < 0.0 {
                    v += 1.0;
                }

                let x = ((image.width() - 1) as f64 * u) as u32;
                let y = ((image.height() - 1) as f64 * v) as u32;
                T::from_pixel(image.get_pixel(x, y))
            }
        }
    }

    pub fn constant(t: T) -> Self {
        Self::Constant(t)
    }

    pub fn checkerboard(a: T, b: T, scale: f64) -> Self {
        Self::Checkerboard { a, b, scale }
    }

    pub fn image(image: DynamicImage) -> Self {
        Self::Image(image.to_rgb8())
    }
}

// Implemented manually to avoid dumping the entire contents of the image when
// debugging
impl<T: Debug> Debug for Texture<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Constant(c) => f.debug_tuple("Constant").field(c).finish(),
            Self::Checkerboard { a, b, scale } => f
                .debug_struct("Checkerboard")
                .field("a", a)
                .field("b", b)
                .field("scale", scale)
                .finish(),
            Self::Image(image) => f
                .debug_tuple("Image")
                .field(&image.width())
                .field(&image.height())
                .finish(),
        }
    }
}

impl Texture<Color> {
    pub fn is_black(&self) -> bool {
        match self {
            Texture::Constant(c) => c.is_black(),
            Texture::Checkerboard { a, b, .. } => a.is_black() && b.is_black(),
            Texture::Image(_) => false,
        }
    }
}

impl Texture<f64> {
    pub fn is_zero(&self) -> bool {
        match self {
            &Texture::Constant(c) => c == 0.0,
            &Texture::Checkerboard { a, b, .. } => a == 0.0 && b == 0.0,
            &Texture::Image(_) => false,
        }
    }
}

impl FromPixel for f64 {
    fn from_pixel(pixel: &Rgb<u8>) -> f64 {
        pixel.to_luma()[0] as f64 / 255.0
    }
}

impl FromPixel for Color {
    fn from_pixel(value: &Rgb<u8>) -> Self {
        Color::from_rgb(value[0], value[1], value[2])
    }
}
