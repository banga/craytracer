use approx::AbsDiffEq;
use std::{fmt::Display, ops::Mul};

use crate::{
    bounds::Bounds,
    constants::EPSILON,
    geometry::{normal::Normal, point::Point, vector::Vector},
    intersection::ShapeIntersection,
    ray::Ray,
};

#[derive(Debug, PartialEq, Clone)]
pub struct Matrix {
    pub m: [[f64; 4]; 4],
}

impl Matrix {
    pub fn new(m: [[i32; 4]; 4]) -> Matrix {
        Matrix {
            m: [
                [
                    m[0][0] as f64,
                    m[0][1] as f64,
                    m[0][2] as f64,
                    m[0][3] as f64,
                ],
                [
                    m[1][0] as f64,
                    m[1][1] as f64,
                    m[1][2] as f64,
                    m[1][3] as f64,
                ],
                [
                    m[2][0] as f64,
                    m[2][1] as f64,
                    m[2][2] as f64,
                    m[2][3] as f64,
                ],
                [
                    m[3][0] as f64,
                    m[3][1] as f64,
                    m[3][2] as f64,
                    m[3][3] as f64,
                ],
            ],
        }
    }

    pub const I: Matrix = Matrix {
        m: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    pub fn transpose(&self) -> Matrix {
        let m = self.m;
        Matrix {
            m: [
                [m[0][0], m[1][0], m[2][0], m[3][0]],
                [m[0][1], m[1][1], m[2][1], m[3][1]],
                [m[0][2], m[1][2], m[2][2], m[3][2]],
                [m[0][3], m[1][3], m[2][3], m[3][3]],
            ],
        }
    }

    /// Calculates the inverse of the matrix using Cramer's rule.
    pub fn inverse(&self) -> Option<Matrix> {
        let m = self.m;
        let mut inv: [[f64; 4]; 4] = [
            [
                m[1][1] * m[2][2] * m[3][3]
                    - m[1][1] * m[2][3] * m[3][2]
                    - m[2][1] * m[1][2] * m[3][3]
                    + m[2][1] * m[1][3] * m[3][2]
                    + m[3][1] * m[1][2] * m[2][3]
                    - m[3][1] * m[1][3] * m[2][2],
                -m[0][1] * m[2][2] * m[3][3]
                    + m[0][1] * m[2][3] * m[3][2]
                    + m[2][1] * m[0][2] * m[3][3]
                    - m[2][1] * m[0][3] * m[3][2]
                    - m[3][1] * m[0][2] * m[2][3]
                    + m[3][1] * m[0][3] * m[2][2],
                m[0][1] * m[1][2] * m[3][3]
                    - m[0][1] * m[1][3] * m[3][2]
                    - m[1][1] * m[0][2] * m[3][3]
                    + m[1][1] * m[0][3] * m[3][2]
                    + m[3][1] * m[0][2] * m[1][3]
                    - m[3][1] * m[0][3] * m[1][2],
                -m[0][1] * m[1][2] * m[2][3]
                    + m[0][1] * m[1][3] * m[2][2]
                    + m[1][1] * m[0][2] * m[2][3]
                    - m[1][1] * m[0][3] * m[2][2]
                    - m[2][1] * m[0][2] * m[1][3]
                    + m[2][1] * m[0][3] * m[1][2],
            ],
            [
                -m[1][0] * m[2][2] * m[3][3]
                    + m[1][0] * m[2][3] * m[3][2]
                    + m[2][0] * m[1][2] * m[3][3]
                    - m[2][0] * m[1][3] * m[3][2]
                    - m[3][0] * m[1][2] * m[2][3]
                    + m[3][0] * m[1][3] * m[2][2],
                m[0][0] * m[2][2] * m[3][3]
                    - m[0][0] * m[2][3] * m[3][2]
                    - m[2][0] * m[0][2] * m[3][3]
                    + m[2][0] * m[0][3] * m[3][2]
                    + m[3][0] * m[0][2] * m[2][3]
                    - m[3][0] * m[0][3] * m[2][2],
                -m[0][0] * m[1][2] * m[3][3]
                    + m[0][0] * m[1][3] * m[3][2]
                    + m[1][0] * m[0][2] * m[3][3]
                    - m[1][0] * m[0][3] * m[3][2]
                    - m[3][0] * m[0][2] * m[1][3]
                    + m[3][0] * m[0][3] * m[1][2],
                m[0][0] * m[1][2] * m[2][3]
                    - m[0][0] * m[1][3] * m[2][2]
                    - m[1][0] * m[0][2] * m[2][3]
                    + m[1][0] * m[0][3] * m[2][2]
                    + m[2][0] * m[0][2] * m[1][3]
                    - m[2][0] * m[0][3] * m[1][2],
            ],
            [
                m[1][0] * m[2][1] * m[3][3]
                    - m[1][0] * m[2][3] * m[3][1]
                    - m[2][0] * m[1][1] * m[3][3]
                    + m[2][0] * m[1][3] * m[3][1]
                    + m[3][0] * m[1][1] * m[2][3]
                    - m[3][0] * m[1][3] * m[2][1],
                -m[0][0] * m[2][1] * m[3][3]
                    + m[0][0] * m[2][3] * m[3][1]
                    + m[2][0] * m[0][1] * m[3][3]
                    - m[2][0] * m[0][3] * m[3][1]
                    - m[3][0] * m[0][1] * m[2][3]
                    + m[3][0] * m[0][3] * m[2][1],
                m[0][0] * m[1][1] * m[3][3]
                    - m[0][0] * m[1][3] * m[3][1]
                    - m[1][0] * m[0][1] * m[3][3]
                    + m[1][0] * m[0][3] * m[3][1]
                    + m[3][0] * m[0][1] * m[1][3]
                    - m[3][0] * m[0][3] * m[1][1],
                -m[0][0] * m[1][1] * m[2][3]
                    + m[0][0] * m[1][3] * m[2][1]
                    + m[1][0] * m[0][1] * m[2][3]
                    - m[1][0] * m[0][3] * m[2][1]
                    - m[2][0] * m[0][1] * m[1][3]
                    + m[2][0] * m[0][3] * m[1][1],
            ],
            [
                -m[1][0] * m[2][1] * m[3][2]
                    + m[1][0] * m[2][2] * m[3][1]
                    + m[2][0] * m[1][1] * m[3][2]
                    - m[2][0] * m[1][2] * m[3][1]
                    - m[3][0] * m[1][1] * m[2][2]
                    + m[3][0] * m[1][2] * m[2][1],
                m[0][0] * m[2][1] * m[3][2]
                    - m[0][0] * m[2][2] * m[3][1]
                    - m[2][0] * m[0][1] * m[3][2]
                    + m[2][0] * m[0][2] * m[3][1]
                    + m[3][0] * m[0][1] * m[2][2]
                    - m[3][0] * m[0][2] * m[2][1],
                -m[0][0] * m[1][1] * m[3][2]
                    + m[0][0] * m[1][2] * m[3][1]
                    + m[1][0] * m[0][1] * m[3][2]
                    - m[1][0] * m[0][2] * m[3][1]
                    - m[3][0] * m[0][1] * m[1][2]
                    + m[3][0] * m[0][2] * m[1][1],
                m[0][0] * m[1][1] * m[2][2]
                    - m[0][0] * m[1][2] * m[2][1]
                    - m[1][0] * m[0][1] * m[2][2]
                    + m[1][0] * m[0][2] * m[2][1]
                    + m[2][0] * m[0][1] * m[1][2]
                    - m[2][0] * m[0][2] * m[1][1],
            ],
        ];

        let det =
            m[0][0] * inv[0][0] + m[0][1] * inv[1][0] + m[0][2] * inv[2][0] + m[0][3] * inv[3][0];

        if det != 0.0 {
            let inv_det = 1.0 / det;

            for j in 0..4 {
                for i in 0..4 {
                    inv[i][j] *= inv_det;
                }
            }
            Some(Matrix { m: inv })
        } else {
            None
        }
    }

    pub fn is_valid(&self) -> bool {
        self.m.iter().flatten().all(|f| f.is_finite())
    }
}

impl Mul<&Matrix> for &Matrix {
    type Output = Matrix;

    fn mul(self, rhs: &Matrix) -> Self::Output {
        let mut result = Matrix { m: [[0.0; 4]; 4] };

        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    result.m[i][j] += self.m[i][k] * rhs.m[k][j];
                }
            }
        }

        result
    }
}

impl AbsDiffEq for Matrix {
    type Epsilon = f64;

    fn default_epsilon() -> Self::Epsilon {
        EPSILON
    }

    fn abs_diff_eq(&self, other: &Self, epsilon: Self::Epsilon) -> bool {
        for i in 0..4 {
            for j in 0..4 {
                if !self.m[i][j].abs_diff_eq(&other.m[i][j], epsilon) {
                    return false;
                }
            }
        }
        true
    }
}

impl Display for Matrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..4 {
            for j in 0..4 {
                write!(f, "{:10.2}", self.m[i][j])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Transformation {
    pub matrix: Matrix,
    pub inverse: Matrix,
}

impl Transformation {
    pub fn is_valid(&self) -> bool {
        self.matrix.is_valid() && self.inverse.is_valid()
    }

    pub fn inverse(&self) -> Self {
        Transformation {
            matrix: self.inverse.clone(),
            inverse: self.matrix.clone(),
        }
    }

    pub fn translate(dx: f64, dy: f64, dz: f64) -> Self {
        Transformation {
            matrix: Matrix {
                m: [
                    [1.0, 0.0, 0.0, dx],
                    [0.0, 1.0, 0.0, dy],
                    [0.0, 0.0, 1.0, dz],
                    [0.0, 0.0, 0.0, 1.0],
                ],
            },
            inverse: Matrix {
                m: [
                    [1.0, 0.0, 0.0, -dx],
                    [0.0, 1.0, 0.0, -dy],
                    [0.0, 0.0, 1.0, -dz],
                    [0.0, 0.0, 0.0, 1.0],
                ],
            },
        }
    }

    pub fn scale(x: f64, y: f64, z: f64) -> Self {
        Transformation {
            matrix: Matrix {
                m: [
                    [x, 0.0, 0.0, 0.0],
                    [0.0, y, 0.0, 0.0],
                    [0.0, 0.0, z, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
            },
            inverse: Matrix {
                m: [
                    [1.0 / x, 0.0, 0.0, 0.0],
                    [0.0, 1.0 / y, 0.0, 0.0],
                    [0.0, 0.0, 1.0 / z, 0.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
            },
        }
    }

    pub fn rotate_x(degrees: f64) -> Self {
        let radians = degrees.to_radians();
        let sin = radians.sin();
        let cos = radians.cos();
        let matrix = Matrix {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, cos, -sin, 0.0],
                [0.0, sin, cos, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };
        let inverse = matrix.transpose();
        Transformation { matrix, inverse }
    }

    pub fn rotate_y(degrees: f64) -> Self {
        let radians = degrees.to_radians();
        let sin = radians.sin();
        let cos = radians.cos();
        let matrix = Matrix {
            m: [
                [cos, 0.0, sin, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [-sin, 0.0, cos, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };
        let inverse = matrix.transpose();
        Transformation { matrix, inverse }
    }

    pub fn rotate_z(degrees: f64) -> Self {
        let radians = degrees.to_radians();
        let sin = radians.sin();
        let cos = radians.cos();
        let matrix = Matrix {
            m: [
                [cos, -sin, 0.0, 0.0],
                [sin, cos, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };
        let inverse = matrix.transpose();
        Transformation { matrix, inverse }
    }

    pub fn look_at(origin: Point, target: Point, up: Vector) -> Self {
        let z = (target - origin).normalized();
        let x = up.normalized().cross(&z).normalized();
        let y = z.cross(&x).normalized();
        let matrix = Matrix {
            m: [
                [x.x(), y.x(), z.x(), origin.x()],
                [x.y(), y.y(), z.y(), origin.y()],
                [x.z(), y.z(), z.z(), origin.z()],
                [0.0, 0.0, 0.0, 1.0],
            ],
        };
        let inverse = matrix.inverse().unwrap();
        Transformation { matrix, inverse }
    }

    pub fn perspective(fov: f64, near: f64, far: f64) -> Self {
        let matrix = Matrix {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, far / (far - near), -far * near / (far - near)],
                [0.0, 0.0, 1.0, 0.0],
            ],
        };
        let inverse = matrix.inverse().unwrap();
        let persp = Transformation { matrix, inverse };
        let inv_tan_ang = 1.0 / (fov.to_radians() * 0.5).tan();
        persp.mul(&Transformation::scale(inv_tan_ang, inv_tan_ang, 1.0))
    }

    pub fn orthographic(near: f64, far: f64) -> Self {
        &Self::scale(1.0, 1.0, 1.0 / (far - near)) * &Self::translate(0.0, 0.0, -near)
    }
}

impl Mul for &Transformation {
    type Output = Transformation;

    fn mul(self, rhs: Self) -> Transformation {
        Transformation {
            matrix: &self.matrix * &rhs.matrix,
            inverse: &rhs.inverse * &self.inverse,
        }
    }
}

pub trait Transformable<T> {
    fn transform(&self, t: &T) -> T;
}

impl Transformable<Point> for Transformation {
    fn transform(&self, p: &Point) -> Point {
        let m = &self.matrix.m;
        // A point is represented as (x, y, z, 1) in homogeneous coordinates,
        // transformed and then converted back by dividing by w
        Point(
            m[0][0] * p.0 + m[0][1] * p.1 + m[0][2] * p.2 + m[0][3],
            m[1][0] * p.0 + m[1][1] * p.1 + m[1][2] * p.2 + m[1][3],
            m[2][0] * p.0 + m[2][1] * p.1 + m[2][2] * p.2 + m[2][3],
        ) / (m[3][0] * p.0 + m[3][1] * p.1 + m[3][2] * p.2 + m[3][3])
    }
}

impl Transformable<Vector> for Transformation {
    fn transform(&self, v: &Vector) -> Vector {
        let m = &self.matrix.m;
        // A vector is represented as (x, y, z, 0) in homogeneous coordinates
        Vector(
            m[0][0] * v.0 + m[0][1] * v.1 + m[0][2] * v.2,
            m[1][0] * v.0 + m[1][1] * v.1 + m[1][2] * v.2,
            m[2][0] * v.0 + m[2][1] * v.1 + m[2][2] * v.2,
        )
    }
}

impl Transformable<Normal> for Transformation {
    fn transform(&self, n: &Normal) -> Normal {
        let inv = &self.inverse.m;
        // Normals need to be transformed by multiplying via the inverse
        // transpose to preserve their perpendicularity
        Normal(
            inv[0][0] * n.0 + inv[1][0] * n.1 + inv[2][0] * n.2,
            inv[0][1] * n.0 + inv[1][1] * n.1 + inv[2][1] * n.2,
            inv[0][2] * n.0 + inv[1][2] * n.1 + inv[2][2] * n.2,
        )
    }
}

impl Transformable<Ray> for Transformation {
    fn transform(&self, ray: &Ray) -> Ray {
        let mut transformed_ray =
            Ray::new(self.transform(&ray.origin), self.transform(&ray.direction));
        transformed_ray.update_max_distance(ray.max_distance);
        transformed_ray
    }
}

impl Transformable<Bounds> for Transformation {
    fn transform(&self, bounds: &Bounds) -> Bounds {
        [
            Point(bounds.min.0, bounds.min.1, bounds.min.2),
            Point(bounds.min.0, bounds.min.1, bounds.max.2),
            Point(bounds.min.0, bounds.max.1, bounds.min.2),
            Point(bounds.min.0, bounds.max.1, bounds.max.2),
            Point(bounds.max.0, bounds.min.1, bounds.min.2),
            Point(bounds.max.0, bounds.min.1, bounds.max.2),
            Point(bounds.max.0, bounds.max.1, bounds.min.2),
            Point(bounds.max.0, bounds.max.1, bounds.max.2),
        ]
        .iter()
        .map(|point| self.transform(point))
        .map(|point| Bounds::new(point, point))
        .sum()
    }
}

impl Transformable<ShapeIntersection> for Transformation {
    fn transform(&self, intersection: &ShapeIntersection) -> ShapeIntersection {
        ShapeIntersection {
            location: self.transform(&intersection.location),
            normal: self.transform(&intersection.normal),
        }
    }
}
