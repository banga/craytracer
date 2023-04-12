#[cfg(test)]
pub mod vector {
    use craytracer::geometry::vector::Vector;
    use pretty_assertions::assert_eq;

    #[test]
    fn normalized() {
        let a = Vector(1.0, 2.0, 2.0);
        assert_eq!(a.normalized(), Vector(1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0));
    }

    #[test]
    fn magnitude() {
        let a = Vector(1.0, 2.0, 2.0);
        assert_eq!(a.magnitude(), 3.0);
    }

    #[test]
    fn dot() {
        let a = Vector(1.0, 2.0, 3.0);
        let b = Vector(-2.0, 2.0, 0.5);
        assert_eq!(a.dot(&b), 3.5);
    }

    #[test]
    fn cross() {
        assert_eq!(Vector::X.cross(&Vector::Y), Vector::Z);
        assert_eq!(Vector::Y.cross(&Vector::Z), Vector::X);
        assert_eq!(Vector::Z.cross(&Vector::X), Vector::Y);

        let a = Vector(1.0, 1.0, 0.0);

        // Cross product with itself is the null vector
        assert_eq!(a.cross(&a), Vector::NULL);

        assert_eq!(a.cross(&Vector::X), Vector(0.0, 0.0, -1.0));
        assert_eq!(a.cross(&Vector::Y), Vector(0.0, 0.0, 1.0));
        assert_eq!(a.cross(&Vector::Z), Vector(1.0, -1.0, 0.0));
    }

    #[test]
    fn equal() {
        let a = Vector(1.0, 2.0, 3.0);
        let b = Vector(1.0, 2.0, 3.0);
        assert_eq!(a, b);
        assert_ne!(a, Vector(2.0, 1.0, 3.0));
    }

    #[test]
    fn add() {
        let a = Vector(1.0, 2.0, 3.0);
        let b = Vector(1.0, 1.0, 1.0);
        assert_eq!(a + b, Vector(2.0, 3.0, 4.0));
    }

    #[test]
    fn add_assign() {
        let mut a = Vector(1.0, 2.0, 3.0);
        a += Vector(1.0, 1.0, 1.0);
        assert_eq!(a, Vector(2.0, 3.0, 4.0));
    }

    #[test]
    fn sub() {
        let a = Vector(1.0, 2.0, 3.0);
        let b = Vector(1.0, 1.0, 1.0);
        assert_eq!(a - b, Vector(0.0, 1.0, 2.0));
    }

    #[test]
    fn sub_assign() {
        let mut a = Vector(1.0, 2.0, 3.0);
        a -= Vector(1.0, 1.0, 1.0);
        assert_eq!(a, Vector(0.0, 1.0, 2.0));
    }

    #[test]
    fn mul() {
        let a = Vector(1.0, 2.0, 3.0);
        assert_eq!(a * 2.0, Vector(2.0, 4.0, 6.0));
    }

    #[test]
    fn mul_assign() {
        let mut a = Vector(1.0, 2.0, 3.0);
        a *= 2.0;
        assert_eq!(a, Vector(2.0, 4.0, 6.0));
    }

    #[test]
    fn div() {
        let a = Vector(1.0, 2.0, 3.0);
        assert_eq!(a / 2.0, Vector(0.5, 1.0, 1.5));
    }

    #[test]
    fn div_assign() {
        let mut a = Vector(1.0, 2.0, 3.0);
        a *= 0.5;
        assert_eq!(a, Vector(0.5, 1.0, 1.5));
    }
}

#[cfg(test)]
pub mod point {
    use craytracer::geometry::point::Point;
    use craytracer::geometry::vector::Vector;
    use pretty_assertions::assert_eq;

    #[test]
    fn equal() {
        let a = Point(1.0, 2.0, 3.0);
        let b = Point(1.0, 2.0, 3.0);
        assert_eq!(a, b);
        assert_ne!(a, Point(2.0, 1.0, 3.0));
    }

    #[test]
    fn add() {
        let a = Point(1.0, 2.0, 3.0);
        let b = Vector(1.0, 1.0, 1.0);
        assert_eq!(a + b, Point(2.0, 3.0, 4.0));
    }

    #[test]
    fn add_assign() {
        let mut a = Point(1.0, 2.0, 3.0);
        a += Vector(1.0, 1.0, 1.0);
        assert_eq!(a, Point(2.0, 3.0, 4.0));
    }

    #[test]
    fn sub() {
        let a = Point(1.0, 2.0, 3.0);
        let b = Point(1.0, 1.0, 1.0);
        assert_eq!(a - b, Vector(0.0, 1.0, 2.0));
    }

    #[test]
    fn sub_vector() {
        let a = Point(1.0, 2.0, 3.0);
        let b = Vector(1.0, 1.0, 1.0);
        assert_eq!(a - b, Point(0.0, 1.0, 2.0));
    }

    #[test]
    fn sub_assign_vector() {
        let mut a = Point(1.0, 2.0, 3.0);
        a -= Vector(1.0, 1.0, 1.0);
        assert_eq!(a, Point(0.0, 1.0, 2.0));
    }
}
