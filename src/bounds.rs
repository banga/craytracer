use std::{iter::Sum, mem::swap, ops::Add};

use crate::{
    constants::EPSILON,
    geometry::{point::Point, vector::Vector, Axis, AXES},
    ray::Ray,
};

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Bounds {
    pub min: Point,
    pub max: Point,
}

impl Bounds {
    pub fn new(a: Point, b: Point) -> Bounds {
        Bounds {
            min: a.min(b),
            max: a.max(b),
        }
    }
    pub fn centroid(&self) -> Point {
        Point(
            (self.min.x() + self.max.x()) * 0.5,
            (self.min.y() + self.max.y()) * 0.5,
            (self.min.z() + self.max.z()) * 0.5,
        )
    }
    pub fn surface_area(&self) -> f64 {
        let Vector(dx, dy, dz) = self.diagonal();
        2.0 * (dx * dy + dy * dz + dz * dx)
    }
    pub fn diagonal(&self) -> Vector {
        self.max - self.min
    }
    pub fn maximum_extent(&self) -> Axis {
        let diagonal = self.diagonal();
        if diagonal.x() > diagonal.y() && diagonal.x() > diagonal.z() {
            Axis::X
        } else if diagonal.y() > diagonal.z() {
            Axis::Y
        } else {
            Axis::Z
        }
    }
    pub fn contains(&self, point: &Point) -> bool {
        self.min.x() <= point.x()
            && self.min.y() <= point.y()
            && self.min.z() <= point.z()
            && self.max.x() >= point.x()
            && self.max.y() >= point.y()
            && self.max.z() >= point.z()
    }
    /// Convert a `point` within bounds to [0, 1]^3
    pub fn offset(&self, point: &Point) -> Vector {
        Vector(
            (point.x() - self.min.x()) / (self.max.x() - self.min.x()),
            (point.y() - self.min.y()) / (self.max.y() - self.min.y()),
            (point.z() - self.min.z()) / (self.max.z() - self.min.z()),
        )
    }
    pub fn intersect(&self, ray: &Ray) -> Option<f64> {
        let mut min_distance = f64::NEG_INFINITY;
        let mut max_distance = f64::INFINITY;

        // For each axis, constrain the min and max distance
        for axis in AXES {
            let d_i = ray.direction[axis];
            let o_i = ray.origin[axis];
            let mut min_i = self.min[axis];
            let mut max_i = self.max[axis];
            if d_i.is_sign_negative() {
                swap(&mut min_i, &mut max_i);
            }

            max_distance = max_distance.min((max_i - o_i) / d_i);
            if max_distance < EPSILON {
                return None;
            }

            min_distance = min_distance.max((min_i - o_i) / d_i);
            if min_distance > max_distance {
                return None;
            }
        }

        if ray.contains_distance(min_distance) {
            Some(min_distance)
        } else if ray.contains_distance(max_distance) {
            Some(max_distance)
        } else {
            None
        }
    }
}

impl Add for Bounds {
    type Output = Bounds;

    fn add(self, rhs: Self) -> Self::Output {
        Bounds {
            min: Point(
                self.min.x().min(rhs.min.x()),
                self.min.y().min(rhs.min.y()),
                self.min.z().min(rhs.min.z()),
            ),
            max: Point(
                self.max.x().max(rhs.max.x()),
                self.max.y().max(rhs.max.y()),
                self.max.z().max(rhs.max.z()),
            ),
        }
    }
}

impl Add<Point> for Bounds {
    type Output = Bounds;

    fn add(self, rhs: Point) -> Self::Output {
        Bounds {
            min: Point(
                self.min.x().min(rhs.x()),
                self.min.y().min(rhs.y()),
                self.min.z().min(rhs.z()),
            ),
            max: Point(
                self.max.x().max(rhs.x()),
                self.max.y().max(rhs.y()),
                self.max.z().max(rhs.z()),
            ),
        }
    }
}

impl Sum for Bounds {
    fn sum<I: Iterator<Item = Self>>(mut iter: I) -> Self {
        let mut bounds = iter.next().unwrap();
        for b in iter {
            bounds = bounds + b;
        }
        bounds
    }
}
