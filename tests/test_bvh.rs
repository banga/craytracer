use pretty_assertions::assert_eq;
use std::sync::Arc;

use craytracer::{
    bvh::Bvh,
    color::Color,
    geometry::{point::Point, X},
    material::Material,
    p,
    primitive::Primitive,
    ray::Ray,
    shape::Shape,
};

#[test]
fn bvh_node() {
    let node = Bvh::new(vec![
        Arc::new(Primitive::new(
            Arc::new(Shape::new_sphere(p!(0.5, 0.5, 0.5), 0.5)),
            Arc::new(Material::new_matte(Color::WHITE, 0.0)),
            None,
        )),
        Arc::new(Primitive::new(
            Arc::new(Shape::new_sphere(p!(1.5, 0.5, 0.5), 0.5)),
            Arc::new(Material::new_matte(Color::WHITE, 0.0)),
            None,
        )),
    ]);

    // Intersect from left
    assert_eq!(
        p!(0, 0.5, 0.5),
        node.intersect(&mut Ray::new(p!(-1, 0.5, 0.5), X,))
            .unwrap()
            .location
    );

    // Intersect from right
    assert_eq!(
        p!(2, 0.5, 0.5),
        node.intersect(&mut Ray::new(p!(3, 0.5, 0.5), -X,))
            .unwrap()
            .location
    );

    // Intersect from inside first sphere
    assert_eq!(
        p!(1, 0.5, 0.5),
        node.intersect(&mut Ray::new(p!(0.5, 0.5, 0.5), X,))
            .unwrap()
            .location
    );
    assert_eq!(
        p!(0, 0.5, 0.5),
        node.intersect(&mut Ray::new(p!(0.5, 0.5, 0.5), -X,))
            .unwrap()
            .location
    );
}
