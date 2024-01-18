#[cfg(test)]
pub mod vector {
    use craytracer::{
        geometry::{traits::DotProduct, vector::Vector, X, Y, Z},
        v,
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn normalized() {
        let a = v!(1, 2, 2);
        assert_eq!(a.normalized(), v!(1.0 / 3.0, 2.0 / 3.0, 2.0 / 3.0));
    }

    #[test]
    fn magnitude() {
        let a = v!(1, 2, 2);
        assert_eq!(a.magnitude(), 3.0);
    }

    #[test]
    fn dot() {
        let a = v!(1, 2, 3);
        let b = v!(-2, 2, 0.5);
        assert_eq!(a.dot(&b), 3.5);
    }

    #[test]
    fn cross() {
        assert_eq!(X.cross(&Y), Z);
        assert_eq!(Y.cross(&Z), X);
        assert_eq!(Z.cross(&X), Y);

        let a = v!(1, 1, 0);

        // Cross product with itself is the null vector
        assert_eq!(a.cross(&a), v!(0, 0, 0));

        assert_eq!(a.cross(&X), v!(0, 0, -1));
        assert_eq!(a.cross(&Y), v!(0, 0, 1));
        assert_eq!(a.cross(&Z), v!(1, -1, 0));
    }

    #[test]
    fn equal() {
        let a = v!(1, 2, 3);
        let b = v!(1, 2, 3);
        assert_eq!(a, b);
        assert_ne!(a, v!(2, 1, 3));
    }

    #[test]
    fn add() {
        let a = v!(1, 2, 3);
        let b = v!(1, 1, 1);
        assert_eq!(a + b, v!(2, 3, 4));
    }

    #[test]
    fn add_assign() {
        let mut a = v!(1, 2, 3);
        a += v!(1, 1, 1);
        assert_eq!(a, v!(2, 3, 4));
    }

    #[test]
    fn sub() {
        let a = v!(1, 2, 3);
        let b = v!(1, 1, 1);
        assert_eq!(a - b, v!(0, 1, 2));
    }

    #[test]
    fn sub_assign() {
        let mut a = v!(1, 2, 3);
        a -= v!(1, 1, 1);
        assert_eq!(a, v!(0, 1, 2));
    }

    #[test]
    fn mul() {
        let a = v!(1, 2, 3);
        assert_eq!(a * 2.0, v!(2, 4, 6));
    }

    #[test]
    fn mul_assign() {
        let mut a = v!(1, 2, 3);
        a *= 2.0;
        assert_eq!(a, v!(2, 4, 6));
    }

    #[test]
    fn div() {
        let a = v!(1, 2, 3);
        assert_eq!(a / 2.0, v!(0.5, 1.0, 1.5));
    }

    #[test]
    fn div_assign() {
        let mut a = v!(1, 2, 3);
        a *= 0.5;
        assert_eq!(a, v!(0.5, 1.0, 1.5));
    }
}

#[cfg(test)]
pub mod point {
    use craytracer::geometry::point::Point;
    use craytracer::geometry::vector::Vector;
    use craytracer::{p, v};
    use pretty_assertions::assert_eq;

    #[test]
    fn equal() {
        let a = p!(1, 2, 3);
        let b = p!(1, 2, 3);
        assert_eq!(a, b);
        assert_ne!(a, p!(2, 1, 3));
    }

    #[test]
    fn add() {
        let a = p!(1, 2, 3);
        let b = v!(1, 1, 1);
        assert_eq!(a + b, p!(2, 3, 4));
    }

    #[test]
    fn add_assign() {
        let mut a = p!(1, 2, 3);
        a += v!(1, 1, 1);
        assert_eq!(a, p!(2, 3, 4));
    }

    #[test]
    fn sub() {
        let a = p!(1, 2, 3);
        let b = p!(1, 1, 1);
        assert_eq!(a - b, v!(0, 1, 2));
    }

    #[test]
    fn sub_vector() {
        let a = p!(1, 2, 3);
        let b = v!(1, 1, 1);
        assert_eq!(a - b, p!(0, 1, 2));
    }

    #[test]
    fn sub_assign_vector() {
        let mut a = p!(1, 2, 3);
        a -= v!(1, 1, 1);
        assert_eq!(a, p!(0, 1, 2));
    }
}
