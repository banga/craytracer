use approx::assert_abs_diff_eq;

use craytracer::{
    bxdf::{reflect, refract},
    geometry::{normal::Normal, traits::DotProduct, vector::Vector},
};

#[test]
fn reflect_test() {
    assert_abs_diff_eq!(
        reflect(&Vector(1.0, -1.0, 0.0).normalized(), &Normal(0.0, 1.0, 0.0)),
        Vector(1.0, 1.0, 0.0).normalized()
    );
}

#[test]
fn refract_test() {
    let direction = &Vector::new(1, -1, 0).normalized();
    let normal = Normal(0.0, 1.0, 0.0);
    assert_abs_diff_eq!(
        refract(&direction, &normal, -direction.dot(&normal), 1.0, 1.0).unwrap(),
        Vector::new(1, -1, 0).normalized()
    );
}
