use std::sync::Arc;

use craytracer::{
    bvh::BvhNode, color::Color, material::Material, primitive::Primitive, ray::Ray, shape::Shape,
    vector::Vector,
};

#[test]
fn bvh_node() {
    let node = BvhNode::new(vec![
        Arc::new(Primitive::new_shape_primitive(
            Arc::new(Shape::new_sphere(Vector(0.5, 0.5, 0.5), 0.5)),
            Arc::new(Material::new_matte(Color::WHITE, 0.0)),
        )),
        Arc::new(Primitive::new_shape_primitive(
            Arc::new(Shape::new_sphere(Vector(1.5, 0.5, 0.5), 0.5)),
            Arc::new(Material::new_matte(Color::WHITE, 0.0)),
        )),
    ]);

    // Intersect from left
    assert_eq!(
        Vector(0.0, 0.5, 0.5),
        node.intersect(&mut Ray::new(Vector(-1.0, 0.5, 0.5), Vector::X,))
            .unwrap()
            .location
    );

    // Intersect from right
    assert_eq!(
        Vector(2.0, 0.5, 0.5),
        node.intersect(&mut Ray::new(Vector(3.0, 0.5, 0.5), -Vector::X,))
            .unwrap()
            .location
    );

    // Intersect from inside first sphere
    assert_eq!(
        Vector(1.0, 0.5, 0.5),
        node.intersect(&mut Ray::new(Vector(0.5, 0.5, 0.5), Vector::X,))
            .unwrap()
            .location
    );
    assert_eq!(
        Vector(0.0, 0.5, 0.5),
        node.intersect(&mut Ray::new(Vector(0.5, 0.5, 0.5), -Vector::X,))
            .unwrap()
            .location
    );
}
