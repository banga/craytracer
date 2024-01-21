use approx::assert_abs_diff_eq;

use craytracer::{
    bxdf::{reflect, refract},
    geometry::{normal::Normal, traits::DotProduct, vector::Vector},
    n, v,
};

#[test]
fn reflect_test() {
    assert_abs_diff_eq!(
        reflect(&v!(-1, 1, 0).normalized(), &n!(0, 1, 0)),
        v!(1, 1, 0).normalized()
    );
}

#[test]
fn refract_test() {
    let direction = &Vector::new(-1, 1, 0).normalized();
    let normal = n!(0, 1, 0);
    assert_abs_diff_eq!(
        refract(&direction, &normal, direction.dot(&normal), 1.0, 1.0).unwrap(),
        Vector::new(1, -1, 0).normalized()
    );
}
