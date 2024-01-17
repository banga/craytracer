use pretty_assertions::assert_eq;
use std::sync::Arc;

use craytracer::{
    bvh::BvhNode,
    color::Color,
    geometry::{point::Point, vector::Vector},
    material::Material,
    primitive::Primitive,
    ray::Ray,
    shape::Shape,
};

#[test]
fn bvh_node() {
    let node = BvhNode::new(vec![
        Arc::new(Primitive::new(
            Arc::new(Shape::new_sphere(Point(0.5, 0.5, 0.5), 0.5)),
            Arc::new(Material::new_matte(Color::WHITE, 0.0)),
            None,
        )),
        Arc::new(Primitive::new(
            Arc::new(Shape::new_sphere(Point(1.5, 0.5, 0.5), 0.5)),
            Arc::new(Material::new_matte(Color::WHITE, 0.0)),
            None,
        )),
    ]);

    // Intersect from left
    assert_eq!(
        Point(0.0, 0.5, 0.5),
        node.intersect(&mut Ray::new(Point(-1.0, 0.5, 0.5), Vector::X,))
            .unwrap()
            .location
    );

    // Intersect from right
    assert_eq!(
        Point(2.0, 0.5, 0.5),
        node.intersect(&mut Ray::new(Point(3.0, 0.5, 0.5), -Vector::X,))
            .unwrap()
            .location
    );

    // Intersect from inside first sphere
    assert_eq!(
        Point(1.0, 0.5, 0.5),
        node.intersect(&mut Ray::new(Point(0.5, 0.5, 0.5), Vector::X,))
            .unwrap()
            .location
    );
    assert_eq!(
        Point(0.0, 0.5, 0.5),
        node.intersect(&mut Ray::new(Point(0.5, 0.5, 0.5), -Vector::X,))
            .unwrap()
            .location
    );
}
