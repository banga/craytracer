mod sphere {
    use craytracer::{bounds::Bounds, shape::Shape, vector::Vector};
    use pretty_assertions::assert_eq;

    #[test]
    fn bounds() {
        assert_eq!(
            Shape::new_sphere(Vector(0.0, 0.0, 0.0), 1.0,).bounds(),
            Bounds::new(Vector(-1.0, -1.0, -1.0), Vector(1.0, 1.0, 1.0),)
        );

        assert_eq!(
            Shape::new_sphere(Vector(-2.0, 3.0, 0.0), 1.0,).bounds(),
            Bounds::new(Vector(-3.0, 2.0, -1.0), Vector(-1.0, 4.0, 1.0),)
        );
    }
}

mod triangle {
    use approx::assert_abs_diff_eq;
    use craytracer::{bounds::Bounds, constants::EPSILON, ray::Ray, shape::Shape, vector::Vector};
    use pretty_assertions::assert_eq;
    use rand::{thread_rng, Rng};

    // Triangle in XY plane
    fn triangle() -> Shape {
        Shape::new_triangle(
            Vector(1.0, 0.0, 0.0),
            Vector(1.0, 1.0, 0.0),
            Vector(2.0, 0.0, 0.0),
        )
    }

    #[test]
    fn bounds() {
        assert_eq!(
            triangle().bounds(),
            Bounds::new(Vector(1.0, 0.0, 0.0), Vector(2.0, 1.0, 0.0),)
        );
    }

    #[test]
    fn intersect_vertices() {
        // Shoot ray to hit v0
        let t = triangle();
        match triangle() {
            Shape::Triangle { v0, e1, e2, .. } => {
                for point in [v0, v0 + e1, v0 + e2] {
                    let ray = &mut Ray::new(Vector(point.x(), point.y(), -2.0), Vector::Z);
                    let intersection = t.intersect(ray).unwrap();
                    assert_eq!(ray.max_distance, 2.0);
                    assert_eq!(intersection.normal, Vector(0.0, 0.0, 1.0));
                }
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn from_behind() {
        // Shoot ray from the opposite side
        let t = triangle();
        let ray = &mut Ray::new(Vector(1.0, 0.0, 2.0), -Vector::Z);
        let intersection = t.intersect(ray).unwrap();
        assert_eq!(ray.max_distance, 2.0);
        assert_eq!(intersection.normal, Vector(0.0, 0.0, 1.0));
    }

    #[test]
    fn parallel_to_triangle() {
        assert!(triangle()
            .intersect(&mut Ray::new(
                Vector(0.0, 0.0, 0.0),
                Vector(1.0, 1.0, 0.0).normalized(),
            ))
            .is_none());
    }

    #[test]
    fn random_point() {
        let t = triangle();

        match triangle() {
            Shape::Triangle { v0, e1, e2, .. } => {
                let mut rng = thread_rng();
                let u = rng.gen_range(0.0..1.0);
                let v = rng.gen_range(0.0..1.0);
                let target = v0 + e1 * u + e2 * v;

                let origin = Vector(0.0, 0.0, -2.0);
                let ray = &mut Ray::new(origin, (target - origin).normalized());
                let intersection = t.intersect(ray);

                if u + v <= 1.0 {
                    let intersection = intersection.expect("Expected an intersection");
                    assert_abs_diff_eq!(
                        ray.max_distance,
                        (target - origin).magnitude(),
                        epsilon = EPSILON
                    );
                    assert_eq!(intersection.normal, Vector(0.0, 0.0, 1.0));
                } else {
                    assert!(intersection.is_none());
                }
            }
            _ => unreachable!(),
        }
    }
}
