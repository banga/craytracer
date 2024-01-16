#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2,
}

pub const AXES: [Axis; 3] = [Axis::X, Axis::Y, Axis::Z];

pub mod vector {
    use super::normal::Normal;
    use super::traits::DotProduct;
    use super::Axis;
    use crate::constants::EPSILON;
    use approx::AbsDiffEq;
    use std::{
        fmt::Display,
        ops::{
            Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
        },
    };

    #[derive(Clone, Copy, Debug)]
    pub struct Vector(pub f64, pub f64, pub f64);

    impl Vector {
        pub fn new(x: i32, y: i32, z: i32) -> Vector {
            Vector(x as f64, y as f64, z as f64)
        }
        pub const X: Vector = Vector(1.0, 0.0, 0.0);
        pub const Y: Vector = Vector(0.0, 1.0, 0.0);
        pub const Z: Vector = Vector(0.0, 0.0, 1.0);
        pub const NULL: Vector = Vector(0.0, 0.0, 0.0);
        pub fn x(&self) -> f64 {
            self.0
        }
        pub fn y(&self) -> f64 {
            self.1
        }
        pub fn z(&self) -> f64 {
            self.2
        }
        pub fn magnitude_squared(&self) -> f64 {
            self.dot(self)
        }
        pub fn magnitude(&self) -> f64 {
            self.magnitude_squared().sqrt()
        }
        pub fn normalized(&self) -> Vector {
            let mag = self.magnitude();
            Vector(self.x() / mag, self.y() / mag, self.z() / mag)
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

    impl From<&Normal> for Vector {
        fn from(n: &Normal) -> Self {
            Vector(n.x(), n.y(), n.z())
        }
    }

    impl From<Normal> for Vector {
        fn from(n: Normal) -> Self {
            Vector(n.x(), n.y(), n.z())
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

    impl IndexMut<Axis> for Vector {
        fn index_mut(&mut self, index: Axis) -> &mut Self::Output {
            match index {
                Axis::X => &mut self.0,
                Axis::Y => &mut self.1,
                Axis::Z => &mut self.2,
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
}

pub mod point {
    use super::vector::Vector;
    use super::Axis;
    use crate::constants::EPSILON;
    use approx::AbsDiffEq;
    use std::{
        fmt::Display,
        ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Sub, SubAssign},
    };

    #[derive(Clone, Copy, Debug)]
    pub struct Point(pub f64, pub f64, pub f64);

    impl Point {
        pub fn new(x: i32, y: i32, z: i32) -> Point {
            Point(x as f64, y as f64, z as f64)
        }
        pub const O: Point = Point(0.0, 0.0, 0.0);
        pub fn x(&self) -> f64 {
            self.0
        }
        pub fn y(&self) -> f64 {
            self.1
        }
        pub fn z(&self) -> f64 {
            self.2
        }
        pub fn min(&self, other: Point) -> Point {
            Point(
                self.0.min(other.0),
                self.1.min(other.1),
                self.2.min(other.2),
            )
        }
        pub fn max(&self, other: Point) -> Point {
            Point(
                self.0.max(other.0),
                self.1.max(other.1),
                self.2.max(other.2),
            )
        }
    }

    impl PartialEq for Point {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0 && self.1 == other.1 && self.2 == other.2
        }
    }

    impl Sub<Point> for Point {
        type Output = Vector;

        fn sub(self, rhs: Point) -> Self::Output {
            Vector(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
        }
    }

    impl Add<Vector> for Point {
        type Output = Point;

        fn add(self, rhs: Vector) -> Self::Output {
            Point(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
        }
    }

    impl AddAssign<Vector> for Point {
        fn add_assign(&mut self, rhs: Vector) {
            self.0 += rhs.0;
            self.1 += rhs.1;
            self.2 += rhs.2;
        }
    }

    impl Sub<Vector> for Point {
        type Output = Point;

        fn sub(self, rhs: Vector) -> Self::Output {
            Point(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
        }
    }

    impl SubAssign<Vector> for Point {
        fn sub_assign(&mut self, rhs: Vector) {
            self.0 -= rhs.0;
            self.1 -= rhs.1;
            self.2 -= rhs.2;
        }
    }

    impl Div<f64> for Point {
        type Output = Point;
        fn div(self, rhs: f64) -> Self::Output {
            Point(self.0 / rhs, self.1 / rhs, self.2 / rhs)
        }
    }

    impl DivAssign<f64> for Point {
        fn div_assign(&mut self, rhs: f64) {
            self.0 /= rhs;
            self.1 /= rhs;
            self.2 /= rhs;
        }
    }

    impl Index<Axis> for Point {
        type Output = f64;

        fn index(&self, index: Axis) -> &Self::Output {
            match index {
                Axis::X => &self.0,
                Axis::Y => &self.1,
                Axis::Z => &self.2,
            }
        }
    }

    impl IndexMut<Axis> for Point {
        fn index_mut(&mut self, index: Axis) -> &mut Self::Output {
            match index {
                Axis::X => &mut self.0,
                Axis::Y => &mut self.1,
                Axis::Z => &mut self.2,
            }
        }
    }

    impl AbsDiffEq for Point {
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

    impl Display for Point {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "({},{},{})", self.0, self.1, self.z())
        }
    }
}

pub mod normal {
    use super::traits::DotProduct;
    use super::vector::Vector;
    use super::Axis;
    use crate::constants::EPSILON;
    use approx::AbsDiffEq;
    use std::{
        fmt::Display,
        ops::{
            Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign,
        },
    };

    #[derive(Clone, Copy, Debug)]
    pub struct Normal(pub f64, pub f64, pub f64);

    impl Normal {
        pub const X: Normal = Normal(1.0, 0.0, 0.0);
        pub const Y: Normal = Normal(0.0, 1.0, 0.0);
        pub const Z: Normal = Normal(0.0, 0.0, 1.0);

        pub fn new(x: i32, y: i32, z: i32) -> Normal {
            Normal(x as f64, y as f64, z as f64)
        }
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
            self.dot(self).sqrt()
        }
        pub fn normalized(&self) -> Normal {
            let mag = self.magnitude();
            Normal(self.x() / mag, self.y() / mag, self.z() / mag)
        }
    }

    impl From<&Vector> for Normal {
        fn from(v: &Vector) -> Self {
            Normal(v.x(), v.y(), v.z())
        }
    }

    impl From<Vector> for Normal {
        fn from(v: Vector) -> Self {
            Normal(v.x(), v.y(), v.z())
        }
    }

    impl PartialEq for Normal {
        fn eq(&self, other: &Self) -> bool {
            self.0 == other.0 && self.1 == other.1 && self.2 == other.2
        }
    }

    impl Add for Normal {
        type Output = Normal;

        fn add(self, rhs: Self) -> Self::Output {
            Normal(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
        }
    }

    impl AddAssign for Normal {
        fn add_assign(&mut self, rhs: Self) {
            self.0 += rhs.0;
            self.1 += rhs.1;
            self.2 += rhs.2;
        }
    }

    impl Sub for Normal {
        type Output = Normal;

        fn sub(self, rhs: Self) -> Self::Output {
            Normal(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
        }
    }

    impl SubAssign for Normal {
        fn sub_assign(&mut self, rhs: Self) {
            self.0 -= rhs.0;
            self.1 -= rhs.1;
            self.2 -= rhs.2;
        }
    }

    impl Mul<f64> for Normal {
        type Output = Normal;
        fn mul(self, rhs: f64) -> Self::Output {
            Normal(self.0 * rhs, self.1 * rhs, self.2 * rhs)
        }
    }

    impl MulAssign<f64> for Normal {
        fn mul_assign(&mut self, rhs: f64) {
            self.0 *= rhs;
            self.1 *= rhs;
            self.2 *= rhs;
        }
    }

    impl Div<f64> for Normal {
        type Output = Normal;
        fn div(self, rhs: f64) -> Self::Output {
            Normal(self.0 / rhs, self.1 / rhs, self.2 / rhs)
        }
    }

    impl DivAssign<f64> for Normal {
        fn div_assign(&mut self, rhs: f64) {
            self.0 /= rhs;
            self.1 /= rhs;
            self.2 /= rhs;
        }
    }

    impl Neg for Normal {
        type Output = Normal;

        fn neg(self) -> Self::Output {
            self * -1.0
        }
    }

    impl Index<Axis> for Normal {
        type Output = f64;

        fn index(&self, index: Axis) -> &Self::Output {
            match index {
                Axis::X => &self.0,
                Axis::Y => &self.1,
                Axis::Z => &self.2,
            }
        }
    }

    impl IndexMut<Axis> for Normal {
        fn index_mut(&mut self, index: Axis) -> &mut Self::Output {
            match index {
                Axis::X => &mut self.0,
                Axis::Y => &mut self.1,
                Axis::Z => &mut self.2,
            }
        }
    }

    impl AbsDiffEq for Normal {
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

    impl Display for Normal {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "({},{},{})", self.0, self.1, self.z())
        }
    }
}

pub mod traits {
    use super::normal::Normal;
    use super::vector::Vector;

    pub trait DotProduct<T> {
        fn dot(&self, other: &T) -> f64;
    }

    impl DotProduct<Vector> for Vector {
        fn dot(&self, other: &Vector) -> f64 {
            self.0 * other.0 + self.1 * other.1 + self.2 * other.2
        }
    }

    impl DotProduct<Normal> for Vector {
        fn dot(&self, other: &Normal) -> f64 {
            self.0 * other.0 + self.1 * other.1 + self.2 * other.2
        }
    }

    impl DotProduct<Normal> for Normal {
        fn dot(&self, other: &Normal) -> f64 {
            self.0 * other.0 + self.1 * other.1 + self.2 * other.2
        }
    }

    impl DotProduct<Vector> for Normal {
        fn dot(&self, other: &Vector) -> f64 {
            self.0 * other.0 + self.1 * other.1 + self.2 * other.2
        }
    }
}
