use approx::assert_abs_diff_eq;
use pretty_assertions::assert_eq;
use rand::{thread_rng, Rng};

use craytracer::{
    bounds::Bounds,
    constants::EPSILON,
    geometry::{point::Point, vector::Vector},
    ray::Ray,
};

#[test]
fn test_intersect_axes() {
    let b = Bounds {
        min: Point(-1.0, -1.0, -1.0),
        max: Point(1.0, 1.0, 1.0),
    };

    // X axis
    // assert_eq!(b.intersect(&Ray::new(Point::O, Vector::X,)), Some(1.0));
    assert_eq!(b.intersect(&Ray::new(Point::O, -Vector::X,)), Some(1.0));

    // Y axis
    assert_eq!(b.intersect(&Ray::new(Point::O, Vector::Y,)), Some(1.0));
    assert_eq!(b.intersect(&Ray::new(Point::O, -Vector::Y,)), Some(1.0));

    // Z axis
    assert_eq!(b.intersect(&Ray::new(Point::O, Vector::Z,)), Some(1.0));
    assert_eq!(b.intersect(&Ray::new(Point::O, -Vector::Z,)), Some(1.0));
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
        min: Point::O,
        max: Point::new(1, 1, 1),
    };

    assert_eq!(b.intersect(&Ray::new(Point::new(0, 2, 0), Vector::X)), None);
    assert_eq!(
        b.intersect(&Ray::new(Point::new(0, -2, 0), -Vector::X)),
        None
    );
    assert_eq!(b.intersect(&Ray::new(Point::new(2, 0, 0), Vector::Y)), None);
    assert_eq!(
        b.intersect(&Ray::new(Point::new(-2, 0, 0), -Vector::Y)),
        None
    );
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
