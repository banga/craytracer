#[cfg(test)]
pub mod matrix {
    use approx::assert_abs_diff_eq;
    use craytracer::transformation::Matrix;
    use pretty_assertions::assert_eq;

    #[test]
    pub fn mul() {
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
    pub fn inverse() {
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
    use craytracer::geometry::{O, X, Y, Z};
    use craytracer::ray::Ray;
    use craytracer::transformation::{Transformable, Transformation};
    use craytracer::{n, p, v};
    use pretty_assertions::assert_eq;

    #[test]
    pub fn translation() {
        let t = Transformation::translate(5.0, -3.0, 2.0);

        assert_eq!(t.transform(&p!(-3, 4, 5)), p!(2, 1, 7));
        assert_eq!(t.transform(&v!(-3, 4, 5)), v!(-3, 4, 5));
        assert_eq!(t.transform(&n!(-3, 4, 5)), n!(-3, 4, 5));
        assert_eq!(
            t.transform(&Ray::new(p!(-3, 4, 5), v!(0, 0, 1))),
            Ray::new(p!(2, 1, 7), v!(0, 0, 1))
        );
        assert_eq!(
            t.transform(&Bounds::new(O, p!(1, 2, 3))),
            Bounds::new(p!(5, -3, 2), p!(6, -1, 5))
        );
    }

    #[test]
    pub fn scale() {
        let t = Transformation::scale(2.0, -3.0, 0.5);

        assert_eq!(t.transform(&p!(-3, 4, 5)), p!(-6, -12, 2.5));
        assert_eq!(t.transform(&v!(-3, 4, 5)), v!(-6, -12, 2.5));
        assert_eq!(
            t.transform(&n!(-3, 4, 5)),
            n!(-3.0 / 2.0, -4.0 / 3.0, 5.0 / 0.5)
        );
        assert_eq!(
            t.transform(&Ray::new(p!(-3, 4, 5), v!(0, 0, 1))),
            Ray::new(p!(-6, -12, 2.5), v!(0, 0, 0.5))
        );
        assert_eq!(
            t.transform(&Bounds::new(O, p!(1, 2, 3))),
            Bounds::new(O, p!(2, -6, 1.5))
        );
    }

    #[test]
    pub fn rotate_x() {
        let t = Transformation::rotate_x(90.0_f64.to_radians());

        assert_abs_diff_eq!(t.transform(&p!(2, 1, 3)), p!(2, -3, 1),);
        assert_abs_diff_eq!(t.transform(&v!(2, 1, 3)), v!(2, -3, 1),);
        assert_abs_diff_eq!(t.transform(&n!(2, 1, 3)), n!(2, -3, 1),);

        let ray = t.transform(&Ray::new(p!(2, 1, 3), v!(0, 0, 1)));
        assert_abs_diff_eq!(ray.origin, p!(2, -3, 1),);
        assert_abs_diff_eq!(ray.direction, v!(0, -1, 0));
    }

    #[test]
    pub fn rotate_y() {
        let t = Transformation::rotate_y(90.0_f64.to_radians());

        assert_abs_diff_eq!(t.transform(&p!(2, 1, 3)), p!(3, 1, -2),);
        assert_abs_diff_eq!(t.transform(&v!(2, 1, 3)), v!(3, 1, -2),);
        assert_abs_diff_eq!(t.transform(&n!(2, 1, 3)), n!(3, 1, -2),);

        let ray = t.transform(&Ray::new(p!(2, 1, 3), v!(0, 0, 1)));
        assert_abs_diff_eq!(ray.origin, p!(3, 1, -2),);
        assert_abs_diff_eq!(ray.direction, v!(1, 0, 0));
    }

    #[test]
    pub fn rotate_z() {
        let t = Transformation::rotate_z(90.0_f64.to_radians());

        assert_abs_diff_eq!(t.transform(&p!(2, 1, 3)), p!(-1, 2, 3),);
        assert_abs_diff_eq!(t.transform(&v!(2, 1, 3)), v!(-1, 2, 3),);
        assert_abs_diff_eq!(t.transform(&n!(2, 1, 3)), n!(-1, 2, 3),);

        let ray = t.transform(&Ray::new(p!(2, 1, 3), v!(1, 0, 0)));
        assert_abs_diff_eq!(ray.origin, p!(-1, 2, 3),);
        assert_abs_diff_eq!(ray.direction, v!(0, 1, 0));
    }

    #[test]
    pub fn look_at() {
        // Look along x axis with z axis as the up direction
        let t = Transformation::look_at(p!(9, 0, 0), p!(10, 0, 0), v!(0, 0, 1));

        assert_abs_diff_eq!(t.transform(&O), p!(9, 0, 0));
        assert_abs_diff_eq!(t.transform(&Z), X);
        assert_abs_diff_eq!(t.transform(&Y), Z);
        assert_abs_diff_eq!(t.transform(&X), Y);
    }

    #[test]
    pub fn perspective() {
        let t = Transformation::perspective(90.0, 50.0, 100.0);

        // z' = (f / (f - n)) / (z / (z - n))
        assert_abs_diff_eq!(t.transform(&p!(0, 0, 50)), p!(0, 0, 0));
        assert_abs_diff_eq!(t.transform(&p!(0, 0, 100)), p!(0, 0, 1));
        assert_abs_diff_eq!(
            t.transform(&p!(0, 0, 75)),
            p!(0, 0, (100.0 / (100.0 - 50.0)) / (75.0 / (75.0 - 50.0)))
        );
    }
}
