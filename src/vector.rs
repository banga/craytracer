use std::{
    fmt::Display,
    iter::FromIterator,
    ops::{Add, Sub},
};

#[derive(Debug)]
pub struct Vector(f64, f64, f64);

impl Vector {
    pub const X: Vector = Vector(1.0, 0.0, 0.0);
    pub const Y: Vector = Vector(0.0, 1.0, 0.0);
    pub const Z: Vector = Vector(0.0, 0.0, 1.0);
    pub const NULL: Vector = Vector(0.0, 0.0, 0.0);

    pub fn new(x: f64, y: f64, z: f64) -> Vector {
        Vector(x, y, z)
    }
    pub fn iter(&self) -> VectorIterator {
        VectorIterator::new(&self)
    }
    pub fn magnitude(&self) -> f64 {
        self.iter().map(|x| x * x).sum::<f64>().sqrt()
    }
    pub fn normalized(&self) -> Vector {
        let mag = self.magnitude();
        Vector::new(self.0 / mag, self.1 / mag, self.2 / mag)
    }
    pub fn dot(&self, other: &Vector) -> f64 {
        self.iter().zip(other.iter()).map(|(a, b)| a * b).sum()
    }
    pub fn cross(&self, other: &Vector) -> Vector {
        Vector(
            self.1 * other.2 - self.2 * other.1,
            self.2 * other.0 - self.0 * other.2,
            self.0 * other.1 - self.1 * other.0,
        )
    }
}

impl FromIterator<f64> for Vector {
    fn from_iter<T: IntoIterator<Item = f64>>(iter: T) -> Self {
        let mut i = iter.into_iter();
        Vector(
            i.next().unwrap_or(0.0),
            i.next().unwrap_or(0.0),
            i.next().unwrap_or(0.0),
        )
    }
}

impl PartialEq for Vector {
    fn eq(&self, other: &Self) -> bool {
        self.iter().zip(other.iter()).all(|(x, y)| x == y)
    }
}

impl Add for Vector {
    type Output = Vector;

    fn add(self, rhs: Self) -> Self::Output {
        self.iter().zip(rhs.iter()).map(|(x, y)| x + y).collect()
    }
}

impl Sub for Vector {
    type Output = Vector;

    fn sub(self, rhs: Self) -> Self::Output {
        self.iter().zip(rhs.iter()).map(|(x, y)| x - y).collect()
    }
}

impl Display for Vector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{},{})", self.0, self.1, self.2)
    }
}

pub struct VectorIterator<'a>(&'a Vector, i8);

impl VectorIterator<'_> {
    fn new(v: &Vector) -> VectorIterator {
        VectorIterator(v, -1)
    }
}

impl Iterator for VectorIterator<'_> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        self.1 += 1;
        match self.1 {
            0 => Some(self.0 .0),
            1 => Some(self.0 .1),
            2 => Some(self.0 .2),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized() {
        let a = Vector::new(1.0, 2.0, 2.0);
        assert_eq!(a.normalized(), Vector::new(1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0));
    }

    #[test]
    fn magnitude() {
        let a = Vector::new(1.0, 2.0, 2.0);
        assert_eq!(a.magnitude(), 3.0);
    }

    #[test]
    fn dot() {
        let a = Vector::new(1.0, 2.0, 3.0);
        let b = Vector::new(-2.0, 2.0, 0.5);
        assert_eq!(a.dot(&b), 3.5);
    }

    #[test]
    fn cross() {
        // Handedness
        assert_eq!(Vector::X.cross(&Vector::Y), Vector::Z);
        assert_eq!(Vector::Y.cross(&Vector::Z), Vector::X);
        assert_eq!(Vector::Z.cross(&Vector::X), Vector::Y);

        let a = Vector::new(1.0, 1.0, 0.0);

        // Cross product with itself is the null vector
        assert_eq!(a.cross(&a), Vector::NULL);

        assert_eq!(a.cross(&Vector::X), Vector::new(0.0, 0.0, -1.0));
        assert_eq!(a.cross(&Vector::Y), Vector::new(0.0, 0.0, 1.0));
        assert_eq!(a.cross(&Vector::Z), Vector::new(1.0, -1.0, 0.0));
    }

    #[test]
    fn equal() {
        let a = Vector::new(1.0, 2.0, 3.0);
        let b = Vector::new(1.0, 2.0, 3.0);
        assert_eq!(a, b);
        assert_ne!(a, Vector::new(2.0, 1.0, 3.0));
    }

    #[test]
    fn add() {
        let a = Vector::new(1.0, 2.0, 3.0);
        let b = Vector::new(1.0, 1.0, 1.0);
        assert_eq!(a + b, Vector::new(2.0, 3.0, 4.0));
    }

    #[test]
    fn sub() {
        let a = Vector::new(1.0, 2.0, 3.0);
        let b = Vector::new(1.0, 1.0, 1.0);
        assert_eq!(a - b, Vector::new(0.0, 1.0, 2.0));
    }
}
