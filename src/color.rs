use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub},
};

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Color {
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
    };

    pub fn from_rgb(r: u8, g: u8, b: u8) -> Color {
        Color {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
        }
    }
    pub fn powf(self, pow: f64) -> Color {
        Color {
            r: self.r.powf(pow),
            g: self.g.powf(pow),
            b: self.b.powf(pow),
        }
    }
    pub fn to_rgb(self) -> (u8, u8, u8) {
        (
            (self.r.clamp(0.0, 1.0) * 255.0) as u8,
            (self.g.clamp(0.0, 1.0) * 255.0) as u8,
            (self.b.clamp(0.0, 1.0) * 255.0) as u8,
        )
    }
    pub fn is_black(self) -> bool {
        self.r == 0.0 && self.g == 0.0 && self.b == 0.0
    }
}

impl From<Color> for (f32, f32, f32) {
    fn from(color: Color) -> (f32, f32, f32) {
        (color.r as f32, color.g as f32, color.b as f32)
    }
}

impl PartialEq for Color {
    fn eq(&self, other: &Self) -> bool {
        self.r == other.r && self.g == other.g && self.b == other.b
    }
}

impl Add for Color {
    type Output = Color;

    fn add(self, rhs: Self) -> Self::Output {
        Color {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
        }
    }
}

impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Self) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
    }
}

impl Mul<f64> for Color {
    type Output = Color;
    fn mul(self, rhs: f64) -> Self::Output {
        Color {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
        }
    }
}

impl MulAssign<f64> for Color {
    fn mul_assign(&mut self, rhs: f64) {
        self.r *= rhs;
        self.g *= rhs;
        self.b *= rhs;
    }
}

impl Div<f64> for Color {
    type Output = Color;
    fn div(self, rhs: f64) -> Self::Output {
        Color {
            r: self.r / rhs,
            g: self.g / rhs,
            b: self.b / rhs,
        }
    }
}

impl DivAssign<f64> for Color {
    fn div_assign(&mut self, rhs: f64) {
        self.r /= rhs;
        self.g /= rhs;
        self.b /= rhs;
    }
}

impl Sub for Color {
    type Output = Color;

    fn sub(self, rhs: Self) -> Self::Output {
        Color {
            r: self.r - rhs.r,
            g: self.g - rhs.g,
            b: self.b - rhs.b,
        }
    }
}

impl Div<Color> for Color {
    type Output = Color;
    fn div(self, rhs: Color) -> Self::Output {
        Color {
            r: self.r / rhs.r,
            g: self.g / rhs.g,
            b: self.b / rhs.b,
        }
    }
}

impl Mul<Color> for Color {
    type Output = Color;
    fn mul(self, rhs: Color) -> Self::Output {
        Color {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
        }
    }
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{},{})", self.r, self.g, self.b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_rgb() {
        assert_eq!(
            Color::from_rgb(255, 128, 0),
            Color {
                r: 1.0,
                g: 128.0 / 255.0,
                b: 0.0
            }
        );
    }

    #[test]
    fn to_rgb() {
        assert_eq!(Color::from_rgb(255, 128, 0).to_rgb(), (255, 128, 0));
    }

    #[test]
    fn add() {
        let a = Color {
            r: 1.0,
            g: 2.0,
            b: 3.0,
        };
        let b = Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        };
        assert_eq!(
            a + b,
            Color {
                r: 2.0,
                g: 3.0,
                b: 4.0
            }
        );
    }

    #[test]
    fn add_assign() {
        let mut a = Color {
            r: 1.0,
            g: 2.0,
            b: 3.0,
        };
        a += Color {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        };
        assert_eq!(
            a,
            Color {
                r: 2.0,
                g: 3.0,
                b: 4.0
            }
        );
    }

    #[test]
    fn mul() {
        let a = Color {
            r: 1.0,
            g: 2.0,
            b: 3.0,
        };
        assert_eq!(
            a * 2.0,
            Color {
                r: 2.0,
                g: 4.0,
                b: 6.0
            }
        );
    }

    #[test]
    fn mul_assign() {
        let mut a = Color {
            r: 1.0,
            g: 2.0,
            b: 3.0,
        };
        a *= 2.0;
        assert_eq!(
            a,
            Color {
                r: 2.0,
                g: 4.0,
                b: 6.0
            }
        );
    }

    #[test]
    fn div() {
        let a = Color {
            r: 1.0,
            g: 2.0,
            b: 3.0,
        };
        assert_eq!(
            a / 2.0,
            Color {
                r: 0.5,
                g: 1.0,
                b: 1.5
            }
        );
    }

    #[test]
    fn div_assign() {
        let mut a = Color {
            r: 1.0,
            g: 2.0,
            b: 3.0,
        };
        a *= 0.5;
        assert_eq!(
            a,
            Color {
                r: 0.5,
                g: 1.0,
                b: 1.5
            }
        );
    }
}
