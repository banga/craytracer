use crate::{constants::EPSILON, intersection::Intersection, ray::Ray, vector::Color, Scene};

fn get_nearest_intersection<'a>(ray: &Ray, scene: &'a Scene) -> Option<Intersection<'a>> {
    let mut nearest_intersection: Option<Intersection> = None;
    for shape in scene.shapes.iter() {
        if let Some(intersection) = shape.intersect(ray) {
            if intersection.distance > EPSILON
                && (nearest_intersection.is_none()
                    || intersection.distance < nearest_intersection.as_ref().unwrap().distance)
            {
                nearest_intersection = Some(intersection);
            }
        }
    }
    nearest_intersection
}

pub fn trace(ray: &Ray, scene: &Scene, depth: u32) -> Color {
    if depth <= 0 {
        return Color::NULL;
    }
    if let Some(intersection) = get_nearest_intersection(&ray, &scene) {
        intersection
            .shape
            .material()
            .sample(scene, &intersection, ray, depth - 1)
    } else {
        scene.background
    }
}
