mod sphere {
    use craytracer::{
        bounds::Bounds,
        geometry::{normal::Normal, point::Point, vector::Vector, AXES},
        intersection::ShapeIntersection,
        n, p,
        ray::Ray,
        shape::Shape,
        v,
    };
    use pretty_assertions::assert_eq;

    fn check_intersection(
        (origin, radius): (Point, f64),
        (ray_origin, ray_direction): (Point, Vector),
        (location, normal): (Point, Normal),
    ) {
        let s = Shape::new_sphere(origin, radius);

        let ray = &mut Ray::new(ray_origin, ray_direction);
        let intersection = s.intersect(ray);
        eprintln!("sphere: {origin}, {radius} ray: {ray_origin}, {ray_direction} intersection: {location}, {normal}");
        assert_eq!(intersection, Some(ShapeIntersection { location, normal }));
    }

    #[test]
    fn intersect_along_axes() {
        let radius = 2.0;
        let origin = p!(0, 0, 0);
        let offsets = [0.0, -1.0, 1.0, 0.001, -0.001, -1e9, 1e9];

        for ox in offsets {
            for oy in offsets {
                for oz in offsets {
                    let offset = Vector(ox, oy, oz);
                    for sign in [1.0, -1.0] {
                        for axis in AXES {
                            let mut ray_origin = p!(0, 0, 0);
                            ray_origin[axis] = (radius + 1.0) * sign;

                            let ray_direction = origin - ray_origin;

                            let mut intersection_location = p!(0, 0, 0);
                            intersection_location[axis] = radius * sign;

                            let mut normal = n!(0, 0, 0);
                            normal[axis] = sign;

                            check_intersection(
                                (origin + offset, radius),
                                (ray_origin + offset, ray_direction),
                                (intersection_location + offset, normal),
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn intersect_internal() {
        let origin = p!(0, 0, 0);
        let radius = 2.0;
        let offsets = [0.0, -1.0, 1.0, 0.001, -0.001, -1e9, 1e9];

        for ox in offsets {
            for oy in offsets {
                for oz in offsets {
                    let offset = Vector(ox, oy, oz);
                    for sign in [1.0, -1.0] {
                        for axis in AXES {
                            let ray_origin = origin;

                            let mut ray_direction = v!(0, 0, 0);
                            ray_direction[axis] = sign;

                            let mut intersection_location = p!(0, 0, 0);
                            intersection_location[axis] = radius * sign;

                            let mut normal = n!(0, 0, 0);
                            normal[axis] = sign;

                            check_intersection(
                                (origin + offset, radius),
                                (ray_origin + offset, ray_direction),
                                (intersection_location + offset, normal),
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn bounds() {
        assert_eq!(
            Shape::new_sphere(p!(0, 0, 0), 1.0,).bounds(),
            Bounds::new(p!(-1, -1, -1), p!(1, 1, 1),)
        );

        assert_eq!(
            Shape::new_sphere(p!(-2, 3, 0), 1.0,).bounds(),
            Bounds::new(p!(-3, 2, -1), p!(-1, 4, 1),)
        );
    }
}

mod triangle {
    use approx::assert_abs_diff_eq;
    use craytracer::{
        bounds::Bounds,
        constants::EPSILON,
        geometry::{point::Point, O, Z},
        n, p,
        ray::Ray,
        shape::Shape,
        v,
    };
    use pretty_assertions::assert_eq;
    use rand::{thread_rng, Rng};

    // Triangle in XY plane
    fn triangle() -> Shape {
        Shape::new_triangle(p!(1, 0, 0), p!(1, 1, 0), p!(2, 0, 0)).unwrap()
    }

    #[test]
    fn bounds() {
        assert_eq!(triangle().bounds(), Bounds::new(p!(1, 0, 0), p!(2, 1, 0),));
    }

    #[test]
    fn intersect_vertices() {
        // Shoot ray to hit v0
        let t = triangle();
        match triangle() {
            Shape::Triangle { v0, e1, e2, .. } => {
                for point in [v0, v0 + e1, v0 + e2] {
                    let ray = &mut Ray::new(Point(point.x(), point.y(), -2.0), Z);
                    let intersection = t.intersect(ray).unwrap();
                    assert_eq!(ray.max_distance, 2.0);
                    assert_eq!(intersection.normal, n!(0, 0, 1));
                }
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn from_behind() {
        // Shoot ray from the opposite side
        let t = triangle();
        let ray = &mut Ray::new(p!(1, 0, 2), -Z);
        let intersection = t.intersect(ray).unwrap();
        assert_eq!(ray.max_distance, 2.0);
        assert_eq!(intersection.normal, n!(0, 0, 1));
    }

    #[test]
    fn parallel_to_triangle() {
        assert!(triangle()
            .intersect(&mut Ray::new(O, v!(1, 1, 0).normalized(),))
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

                let origin = p!(0, 0, -2);
                let ray = &mut Ray::new(origin, (target - origin).normalized());
                let intersection = t.intersect(ray);

                if u + v <= 1.0 {
                    let intersection = intersection.expect("Expected an intersection");
                    assert_abs_diff_eq!(
                        ray.max_distance,
                        (target - origin).magnitude(),
                        epsilon = EPSILON
                    );
                    assert_eq!(intersection.normal, n!(0, 0, 1));
                } else {
                    assert!(intersection.is_none());
                }
            }
            _ => unreachable!(),
        }
    }
}
