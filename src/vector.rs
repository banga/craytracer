use crate::constants::EPSILON;
use approx::AbsDiffEq;
use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Index, Mul, MulAssign, Neg, Sub, SubAssign},
};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
}

pub const AXES: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];

#[derive(Clone, Copy, Debug)]
pub struct Vector(pub f64, pub f64, pub f64);

#[allow(dead_code)]
impl Vector {
    pub fn new(x: i32, y: i32, z: i32) -> Vector {
        Vector(x as f64, y as f64, z as f64)
    }
    pub const X: Vector = Vector(1.0, 0.0, 0.0);
    pub const Y: Vector = Vector(0.0, 1.0, 0.0);
    pub const Z: Vector = Vector(0.0, 0.0, 1.0);
    pub const O: Vector = Vector(0.0, 0.0, 0.0);
    pub fn x(&self) -> f64 {
        self.0
    }
    pub fn y(&self) -> f64 {
        self.1
    }
    pub fn z(&self) -> f64 {
        self.2
    }
    pub fn magnitude(&self) -> f64 {
        (self.0 * self.0 + self.1 * self.1 + self.2 * self.2).sqrt()
    }
    pub fn normalized(&self) -> Vector {
        let mag = self.magnitude();
        Vector(self.x() / mag, self.y() / mag, self.z() / mag)
    }
    pub fn dot(&self, other: &Vector) -> f64 {
        self.0 * other.0 + self.1 * other.1 + self.2 * other.2
    }
    pub fn cross(&self, other: &Vector) -> Vector {
        Vector(
            self.y() * other.z() - self.z() * other.y(),
            self.z() * other.x() - self.x() * other.z(),
            self.x() * other.y() - self.y() * other.x(),
        )
    }
    pub fn powf(&self, p: f64) -> Vector {
        Vector(self.x().powf(p), self.y().powf(p), self.z().powf(p))
    }
}

impl PartialEq for Vector {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1 && self.2 == other.2
    }
}

impl Add for Vector {
    type Output = Vector;

    fn add(self, rhs: Self) -> Self::Output {
        Vector(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl AddAssign for Vector {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
        self.2 += rhs.2;
    }
}

impl Sub for Vector {
    type Output = Vector;

    fn sub(self, rhs: Self) -> Self::Output {
        Vector(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl SubAssign for Vector {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
        self.1 -= rhs.1;
        self.2 -= rhs.2;
    }
}

impl Mul<f64> for Vector {
    type Output = Vector;
    fn mul(self, rhs: f64) -> Self::Output {
        Vector(self.0 * rhs, self.1 * rhs, self.2 * rhs)
    }
}

impl MulAssign<f64> for Vector {
    fn mul_assign(&mut self, rhs: f64) {
        self.0 *= rhs;
        self.1 *= rhs;
        self.2 *= rhs;
    }
}

impl Div<f64> for Vector {
    type Output = Vector;
    fn div(self, rhs: f64) -> Self::Output {
        Vector(self.0 / rhs, self.1 / rhs, self.2 / rhs)
    }
}

impl DivAssign<f64> for Vector {
    fn div_assign(&mut self, rhs: f64) {
        self.0 /= rhs;
        self.1 /= rhs;
        self.2 /= rhs;
    }
}

impl Neg for Vector {
    type Output = Vector;

    fn neg(self) -> Self::Output {
        self * -1.0
    }
}

impl Index<Axis> for Vector {
    type Output = f64;

    fn index(&self, index: Axis) -> &Self::Output {
        match index {
            Axis::X => &self.0,
            Axis::Y => &self.1,
            Axis::Z => &self.2,
        }
    }
}

impl AbsDiffEq for Vector {
    type Epsilon = f64;

    fn default_epsilon() -> Self::Epsilon {
        EPSILON
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        self.0.abs_diff_eq(&other.0, epsilon)
            && self.1.abs_diff_eq(&other.1, epsilon)
            && self.2.abs_diff_eq(&other.2, epsilon)
    }
}

impl Display for Vector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{},{})", self.0, self.1, self.z())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized() {
        let a = Vector(1.0, 2.0, 2.0);
        assert_eq!(a.normalized(), Vector(1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0));
    }

    #[test]
    fn magnitude() {
        let a = Vector(1.0, 2.0, 2.0);
        assert_eq!(a.magnitude(), 3.0);
    }

    #[test]
    fn dot() {
        let a = Vector(1.0, 2.0, 3.0);
        let b = Vector(-2.0, 2.0, 0.5);
        assert_eq!(a.dot(&b), 3.5);
    }

    #[test]
    fn cross() {
        // Handedness
        assert_eq!(Vector::X.cross(&Vector::Y), Vector::Z);
        assert_eq!(Vector::Y.cross(&Vector::Z), Vector::X);
        assert_eq!(Vector::Z.cross(&Vector::X), Vector::Y);

        let a = Vector(1.0, 1.0, 0.0);

        // Cross product with itself is the null vector
        assert_eq!(a.cross(&a), Vector::O);

        assert_eq!(a.cross(&Vector::X), Vector(0.0, 0.0, -1.0));
        assert_eq!(a.cross(&Vector::Y), Vector(0.0, 0.0, 1.0));
        assert_eq!(a.cross(&Vector::Z), Vector(1.0, -1.0, 0.0));
    }

    #[test]
    fn equal() {
        let a = Vector(1.0, 2.0, 3.0);
        let b = Vector(1.0, 2.0, 3.0);
        assert_eq!(a, b);
        assert_ne!(a, Vector(2.0, 1.0, 3.0));
    }

    #[test]
    fn add() {
        let a = Vector(1.0, 2.0, 3.0);
        let b = Vector(1.0, 1.0, 1.0);
        assert_eq!(a + b, Vector(2.0, 3.0, 4.0));
    }

    #[test]
    fn add_assign() {
        let mut a = Vector(1.0, 2.0, 3.0);
        a += Vector(1.0, 1.0, 1.0);
        assert_eq!(a, Vector(2.0, 3.0, 4.0));
    }

    #[test]
    fn sub() {
        let a = Vector(1.0, 2.0, 3.0);
        let b = Vector(1.0, 1.0, 1.0);
        assert_eq!(a - b, Vector(0.0, 1.0, 2.0));
    }

    #[test]
    fn sub_assign() {
        let mut a = Vector(1.0, 2.0, 3.0);
        a -= Vector(1.0, 1.0, 1.0);
        assert_eq!(a, Vector(0.0, 1.0, 2.0));
    }

    #[test]
    fn mul() {
        let a = Vector(1.0, 2.0, 3.0);
        assert_eq!(a * 2.0, Vector(2.0, 4.0, 6.0));
    }

    #[test]
    fn mul_assign() {
        let mut a = Vector(1.0, 2.0, 3.0);
        a *= 2.0;
        assert_eq!(a, Vector(2.0, 4.0, 6.0));
    }

    #[test]
    fn div() {
        let a = Vector(1.0, 2.0, 3.0);
        assert_eq!(a / 2.0, Vector(0.5, 1.0, 1.5));
    }

    #[test]
    fn div_assign() {
        let mut a = Vector(1.0, 2.0, 3.0);
        a *= 0.5;
        assert_eq!(a, Vector(0.5, 1.0, 1.5));
    }
}
