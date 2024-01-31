use rand::{thread_rng, Rng};

use craytracer::{
    bounds::Bounds,
    geometry::{point::Point, O, X, Y, Z},
    p,
    ray::Ray,
};

#[test]
fn test_intersect_axes() {
    let b = Bounds {
        min: p!(-1, -1, -1),
        max: p!(1, 1, 1),
    };

    // X axis
    assert!(b.intersects(&Ray::new(O, X,)));
    assert!(b.intersects(&Ray::new(O, -X,)));

    // Y axis
    assert!(b.intersects(&Ray::new(O, Y,)));
    assert!(b.intersects(&Ray::new(O, -Y,)));

    // Z axis
    assert!(b.intersects(&Ray::new(O, Z,)));
    assert!(b.intersects(&Ray::new(O, -Z,)));
}

#[test]
fn test_intersect_random() {
    let b = Bounds {
        min: Point::new(-1, -1, -1),
        max: Point::new(1, 1, 1),
    };
    let mut rng = thread_rng();

    for _ in 0..100 {
        // Pick a random point on the left face and create a ray pointing to
        // it
        let origin = Point::new(-2, 0, 0);
        let target = Point(
            -1.0,
            rng.gen_range(b.min.y()..b.max.y()),
            rng.gen_range(b.min.z()..b.max.z()),
        );
        let direction = target - origin;
        let distance = direction.magnitude();
        assert!(b.intersects(&Ray::new(origin, direction / distance)));
    }
}

#[test]
fn test_intersect_miss() {
    let b = Bounds {
        min: O,
        max: Point::new(1, 1, 1),
    };

    assert!(!b.intersects(&Ray::new(Point::new(0, 2, 0), X)));
    assert!(!b.intersects(&Ray::new(Point::new(0, -2, 0), -X)));
    assert!(!b.intersects(&Ray::new(Point::new(2, 0, 0), Y)));
    assert!(!b.intersects(&Ray::new(Point::new(-2, 0, 0), -Y)));
}

#[test]
fn test_sum() {
    assert_eq!(
        Bounds {
            min: Point::new(0, 0, 0),
            max: Point::new(1, 0, 0),
        } + Bounds {
            min: Point::new(0, 0, 0),
            max: Point::new(1, 0, 0),
        },
        Bounds {
            min: Point::new(0, 0, 0),
            max: Point::new(1, 0, 0),
        }
    );

    assert_eq!(
        Bounds {
            min: Point::new(0, 0, 0),
            max: Point::new(1, 0, 0),
        } + Bounds {
            min: Point::new(0, 0, 0),
            max: Point::new(0, 1, 0),
        },
        Bounds {
            min: Point::new(0, 0, 0),
            max: Point::new(1, 1, 0),
        }
    );

    assert_eq!(
        Bounds {
            min: Point::new(0, 0, 0),
            max: Point::new(1, 1, 1),
        } + Bounds {
            min: Point::new(2, 2, 2),
            max: Point::new(3, 3, 3),
        },
        Bounds {
            min: Point::new(0, 0, 0),
            max: Point::new(3, 3, 3),
        }
    );
}
