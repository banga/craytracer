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

#[cfg(test)]
mod tests {
    use approx::assert_abs_diff_eq;
    use rand::{thread_rng, Rng};

    use crate::{bounds::Bounds, constants::EPSILON, ray::Ray, vector::Vector};

    #[test]
    fn test_intersect_axes() {
        let b = Bounds {
            min: Vector(-1.0, -1.0, -1.0),
            max: Vector(1.0, 1.0, 1.0),
        };

        // X axis
        // assert_eq!(b.intersect(&Ray::new(Vector::O, Vector::X,)), Some(1.0));
        assert_eq!(b.intersect(&Ray::new(Vector::O, -Vector::X,)), Some(1.0));

        // Y axis
        assert_eq!(b.intersect(&Ray::new(Vector::O, Vector::Y,)), Some(1.0));
        assert_eq!(b.intersect(&Ray::new(Vector::O, -Vector::Y,)), Some(1.0));

        // Z axis
        assert_eq!(b.intersect(&Ray::new(Vector::O, Vector::Z,)), Some(1.0));
        assert_eq!(b.intersect(&Ray::new(Vector::O, -Vector::Z,)), Some(1.0));
    }

    #[test]
    fn test_intersect_random() {
        let b = Bounds {
            min: -Vector::new(1, 1, 1),
            max: Vector::new(1, 1, 1),
        };
        let mut rng = thread_rng();

        for _ in 0..100 {
            // Pick a random point on the left face and create a ray pointing to
            // it
            let origin = Vector::new(-2, 0, 0);
            let target = Vector(
                -1.0,
                rng.gen_range(b.min.y()..b.max.y()),
                rng.gen_range(b.min.z()..b.max.z()),
            );
            let direction = target - origin;
            let distance = direction.magnitude();
            assert_abs_diff_eq!(
                b.intersect(&Ray::new(origin, direction / distance))
                    .unwrap(),
                distance,
                epsilon = EPSILON
            );
        }
    }

    #[test]
    fn test_intersect_miss() {
        let b = Bounds {
            min: Vector::O,
            max: Vector::new(1, 1, 1),
        };

        assert_eq!(
            b.intersect(&Ray::new(Vector::new(0, 2, 0), Vector::X)),
            None
        );
        assert_eq!(
            b.intersect(&Ray::new(Vector::new(0, -2, 0), -Vector::X)),
            None
        );
        assert_eq!(
            b.intersect(&Ray::new(Vector::new(2, 0, 0), Vector::Y)),
            None
        );
        assert_eq!(
            b.intersect(&Ray::new(Vector::new(-2, 0, 0), -Vector::Y)),
            None
        );
    }

    #[test]
    fn test_sum() {
        assert_eq!(
            Bounds {
                min: Vector::new(0, 0, 0),
                max: Vector::new(1, 0, 0),
            } + Bounds {
                min: Vector::new(0, 0, 0),
                max: Vector::new(1, 0, 0),
            },
            Bounds {
                min: Vector::new(0, 0, 0),
                max: Vector::new(1, 0, 0),
            }
        );

        assert_eq!(
            Bounds {
                min: Vector::new(0, 0, 0),
                max: Vector::new(1, 0, 0),
            } + Bounds {
                min: Vector::new(0, 0, 0),
                max: Vector::new(0, 1, 0),
            },
            Bounds {
                min: Vector::new(0, 0, 0),
                max: Vector::new(1, 1, 0),
            }
        );

        assert_eq!(
            Bounds {
                min: Vector::new(0, 0, 0),
                max: Vector::new(1, 1, 1),
            } + Bounds {
                min: Vector::new(2, 2, 2),
                max: Vector::new(3, 3, 3),
            },
            Bounds {
                min: Vector::new(0, 0, 0),
                max: Vector::new(3, 3, 3),
            }
        );
    }
}
