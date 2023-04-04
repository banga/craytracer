use std::{iter::Sum, mem::swap, ops::Add};

use crate::{
    constants::EPSILON,
    ray::Ray,
    vector::{Vector, AXES},
};

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Bounds {
    pub min: Vector,
    pub max: Vector,
}

impl Bounds {
    pub fn new(min: Vector, max: Vector) -> Bounds {
        assert!(min.x() <= max.x());
        assert!(min.y() <= max.y());
        assert!(min.z() <= max.z());
        Bounds { min, max }
    }
    pub fn contains(&self, point: &Vector) -> bool {
        self.min.x() <= point.x()
            && self.min.y() <= point.y()
            && self.min.z() <= point.z()
            && self.max.x() >= point.x()
            && self.max.y() >= point.y()
            && self.max.z() >= point.z()
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

        if min_distance > EPSILON && min_distance <= ray.max_distance {
            Some(min_distance)
        } else if max_distance > EPSILON && max_distance <= ray.max_distance {
            Some(max_distance)
        } else {
            None
        }
    }
}

impl Bounds {
    pub fn span(&self) -> Vector {
        self.max - self.min
    }
}

impl Add for Bounds {
    type Output = Bounds;

    fn add(self, rhs: Self) -> Self::Output {
        Bounds {
            min: Vector(
                self.min.x().min(rhs.min.x()),
                self.min.y().min(rhs.min.y()),
                self.min.z().min(rhs.min.z()),
            ),
            max: Vector(
                self.max.x().max(rhs.max.x()),
                self.max.y().max(rhs.max.y()),
                self.max.z().max(rhs.max.z()),
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
