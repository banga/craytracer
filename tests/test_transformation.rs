#[cfg(test)]
pub mod matrix {
    use approx::assert_abs_diff_eq;
    use craytracer::transformation::Matrix;
    use pretty_assertions::assert_eq;

    #[test]
    pub fn test_mul() {
        let m1 = Matrix::new([
            // Dürer's magic square
            [16, 3, 2, 13],
            [5, 10, 11, 8],
            [9, 6, 7, 12],
            [4, 15, 14, 1],
        ]);
        let m2 = Matrix::new([
            // Sagrada Família magic square
            [1, 14, 14, 4],
            [11, 7, 6, 9],
            [8, 10, 10, 5],
            [13, 2, 3, 15],
        ]);
        let m3 = Matrix::new([
            [234, 291, 301, 296],
            [307, 266, 264, 285],
            [287, 262, 268, 305],
            [294, 303, 289, 236],
        ]);
        assert_eq!(&m1 * &m2, m3);

        assert_eq!(&m1 * &Matrix::I, m1);
        assert_eq!(&m2 * &Matrix::I, m2);
        assert_eq!(&Matrix::I * &m1, m1);
        assert_eq!(&Matrix::I * &m2, m2);
    }

    #[test]
    pub fn test_inverse() {
        let m = Matrix::new([
            //
            [1, 3, 5, 4],
            [1, 3, 1, 2],
            [0, 3, 4, 3],
            [0, 2, 0, 1],
        ]);
        assert_abs_diff_eq!(
            m.inverse().unwrap(),
            Matrix {
                m: [
                    [-1.0 / 4.0, 5.0 / 4.0, 0.0, -3.0 / 2.0],
                    [-1.0, 1.0, 1.0, -1.0],
                    [-3.0 / 4.0, 3.0 / 4.0, 1.0, -3.0 / 2.0],
                    [2.0, -2.0, -2.0, 3.0],
                ]
            }
        );
        assert_eq!(&m * &(m.inverse().unwrap()), Matrix::I);

        let non_invertible = Matrix::new([
            //
            [1, 0, 0, 0],
            [0, 1, 0, 0],
            [0, 0, 1, 0],
            [1, 0, 0, 0],
        ]);
        assert_eq!(None, non_invertible.inverse());
    }
}

#[cfg(test)]
pub mod transformation {
    use approx::assert_abs_diff_eq;
    use craytracer::bounds::Bounds;
    use craytracer::geometry::normal::Normal;
    use craytracer::geometry::point::Point;
    use craytracer::geometry::vector::Vector;
    use craytracer::ray::Ray;
    use craytracer::transformation::{Transformable, Transformation};
    use pretty_assertions::assert_eq;

    #[test]
    pub fn test_translation() {
        let t = Transformation::translate(&Vector(5.0, -3.0, 2.0));

        assert_eq!(t.transform(&Point(-3.0, 4.0, 5.0)), Point(2.0, 1.0, 7.0));
        assert_eq!(t.transform(&Vector(-3.0, 4.0, 5.0)), Vector(-3.0, 4.0, 5.0));
        assert_eq!(t.transform(&Normal(-3.0, 4.0, 5.0)), Normal(-3.0, 4.0, 5.0));
        assert_eq!(
            t.transform(&Ray::new(Point(-3.0, 4.0, 5.0), Vector(0.0, 0.0, 1.0))),
            Ray::new(Point(2.0, 1.0, 7.0), Vector(0.0, 0.0, 1.0))
        );
        assert_eq!(
            t.transform(&Bounds::new(Point::O, Point(1.0, 2.0, 3.0))),
            Bounds::new(Point(5.0, -3.0, 2.0), Point(6.0, -1.0, 5.0))
        );
    }

    #[test]
    pub fn test_scale() {
        let t = Transformation::scale(2.0, -3.0, 0.5);

        assert_eq!(t.transform(&Point(-3.0, 4.0, 5.0)), Point(-6.0, -12.0, 2.5));
        assert_eq!(
            t.transform(&Vector(-3.0, 4.0, 5.0)),
            Vector(-6.0, -12.0, 2.5)
        );
        assert_eq!(
            t.transform(&Normal(-3.0, 4.0, 5.0)),
            Normal(-3.0 / 2.0, -4.0 / 3.0, 5.0 / 0.5)
        );
        assert_eq!(
            t.transform(&Ray::new(Point(-3.0, 4.0, 5.0), Vector(0.0, 0.0, 1.0))),
            Ray::new(Point(-6.0, -12.0, 2.5), Vector(0.0, 0.0, 0.5))
        );
        assert_eq!(
            t.transform(&Bounds::new(Point::O, Point(1.0, 2.0, 3.0))),
            Bounds::new(Point::O, Point(2.0, -6.0, 1.5))
        );
    }

    #[test]
    pub fn test_rotate_x() {
        let t = Transformation::rotate_x(90.0);

        assert_abs_diff_eq!(t.transform(&Point(2.0, 1.0, 3.0)), Point(2.0, -3.0, 1.0),);
        assert_abs_diff_eq!(t.transform(&Vector(2.0, 1.0, 3.0)), Vector(2.0, -3.0, 1.0),);
        assert_abs_diff_eq!(t.transform(&Normal(2.0, 1.0, 3.0)), Normal(2.0, -3.0, 1.0),);

        let ray = t.transform(&Ray::new(Point(2.0, 1.0, 3.0), Vector(0.0, 0.0, 1.0)));
        assert_abs_diff_eq!(ray.origin, Point(2.0, -3.0, 1.0),);
        assert_abs_diff_eq!(ray.direction, Vector(0.0, -1.0, 0.0));
    }

    #[test]
    pub fn test_rotate_y() {
        let t = Transformation::rotate_y(90.0);

        assert_abs_diff_eq!(t.transform(&Point(2.0, 1.0, 3.0)), Point(3.0, 1.0, -2.0),);
        assert_abs_diff_eq!(t.transform(&Vector(2.0, 1.0, 3.0)), Vector(3.0, 1.0, -2.0),);
        assert_abs_diff_eq!(t.transform(&Normal(2.0, 1.0, 3.0)), Normal(3.0, 1.0, -2.0),);

        let ray = t.transform(&Ray::new(Point(2.0, 1.0, 3.0), Vector(0.0, 0.0, 1.0)));
        assert_abs_diff_eq!(ray.origin, Point(3.0, 1.0, -2.0),);
        assert_abs_diff_eq!(ray.direction, Vector(1.0, 0.0, 0.0));
    }

    #[test]
    pub fn test_rotate_z() {
        let t = Transformation::rotate_z(90.0);

        assert_abs_diff_eq!(t.transform(&Point(2.0, 1.0, 3.0)), Point(-1.0, 2.0, 3.0),);
        assert_abs_diff_eq!(t.transform(&Vector(2.0, 1.0, 3.0)), Vector(-1.0, 2.0, 3.0),);
        assert_abs_diff_eq!(t.transform(&Normal(2.0, 1.0, 3.0)), Normal(-1.0, 2.0, 3.0),);

        let ray = t.transform(&Ray::new(Point(2.0, 1.0, 3.0), Vector(1.0, 0.0, 0.0)));
        assert_abs_diff_eq!(ray.origin, Point(-1.0, 2.0, 3.0),);
        assert_abs_diff_eq!(ray.direction, Vector(0.0, 1.0, 0.0));
    }
}
