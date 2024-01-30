use crate::color::Color;

#[derive(Debug, PartialEq)]
pub enum Texture<T> {
    Constant(T),
    Checkerboard { a: T, b: T, scale: f64 },
}

impl<T: Copy> Texture<T> {
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
        }
    }

    pub fn constant(t: T) -> Self {
        Self::Constant(t)
    }

    pub fn checkerboard(a: T, b: T, scale: f64) -> Self {
        Self::Checkerboard { a, b, scale }
    }
}

impl Texture<Color> {
    pub fn is_black(&self) -> bool {
        match self {
            Texture::Constant(c) => c.is_black(),
            Texture::Checkerboard { a, b, .. } => a.is_black() && b.is_black(),
        }
    }
}

impl Texture<f64> {
    pub fn is_zero(&self) -> bool {
        match self {
            &Texture::Constant(c) => c == 0.0,
            &Texture::Checkerboard { a, b, .. } => a == 0.0 && b == 0.0,
        }
    }
}
