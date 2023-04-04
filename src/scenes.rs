use rand::{Rng, SeedableRng};
use std::{sync::Arc, vec};

use crate::{
    camera::Camera, color::Color, material::Material, obj::load_obj, primitive::Primitive,
    scene::Scene, shape::Shape, vector::Vector,
};

pub fn random_spheres(num_samples: usize, scale: usize) -> Scene {
    let film_width: usize = 600 * scale;
    let film_height: usize = 400 * scale;

    let seed = [19; 32];
    let mut rng = rand::rngs::StdRng::from_seed(seed);

    let mut primitives: Vec<Arc<Primitive>> = vec![
        // Sky
        Arc::new(Primitive::new_shape_primitive(
            Arc::new(Shape::new_sphere(Vector(0.0, 0.0, 10.0), 1000.0)),
            Arc::new(Material::new_emissive(Color::from_rgb(240, 245, 255))),
        )),
        // Ground
        Arc::new(Primitive::new_shape_primitive(
            Arc::new(Shape::new_sphere(Vector(0.0, -1000.0, 10.0), 1000.0)),
            Arc::new(Material::new_matte(Color::from_rgb(200, 180, 150), 0.0)),
        )),
    ];

    for x in -2..2 {
        for z in 6..14 {
            let radius = rng.gen_range(0.15..0.3);
            let material: Material = match rng.gen_range(0..5) {
                0..=2 => {
                    let eta = Color {
                        r: rng.gen_range(0.0..2.0),
                        g: rng.gen_range(0.0..2.0),
                        b: rng.gen_range(0.0..2.0),
                    };
                    let k = Color {
                        r: rng.gen_range(0.0..2.0),
                        g: rng.gen_range(0.0..2.0),
                        b: rng.gen_range(0.0..2.0),
                    };
                    Material::new_metal(eta, k)
                }
                3..=4 => {
                    let color = Color::from_rgb(
                        rng.gen_range(128..255),
                        rng.gen_range(128..255),
                        rng.gen_range(128..255),
                    );
                    Material::new_glass(color, color, rng.gen_range(1.0..3.0))
                }
                _ => Material::new_matte(
                    Color::from_rgb(
                        rng.gen_range(0..255),
                        rng.gen_range(0..255),
                        rng.gen_range(0..255),
                    ),
                    rng.gen_range(0.0..1000.0),
                ),
            };
            primitives.push(Arc::new(Primitive::new_shape_primitive(
                Arc::new(Shape::new_sphere(
                    Vector(x as f64, radius, z as f64)
                        + Vector(rng.gen_range(0.0..0.6), 0.0, rng.gen_range(0.0..0.3)),
                    radius,
                )),
                Arc::new(material),
            )));
        }
    }

    Scene::new(
        4,
        film_width,
        film_height,
        Box::new(Camera::new_projection_camera(
            Vector(2.0, 2.0, 0.0),
            Vector(0.0, 0.0, 10.0),
            Vector::Y,
            5.0,
            num_samples,
            film_width,
            film_height,
        )),
        primitives,
    )
}

fn brass() -> Material {
    Material::new_metal(
        Color {
            r: 0.44400,
            g: 0.52700,
            b: 1.09400,
        },
        Color {
            r: 3.69500,
            g: 2.76500,
            b: 1.82900,
        },
    )
}

fn chrome() -> Material {
    Material::new_metal(
        Color {
            r: 0.944,
            g: 0.776,
            b: 0.373,
        },
        Color {
            r: 4.0,
            g: 3.0,
            b: 2.0,
        },
    )
}

fn copper() -> Material {
    Material::new_metal(
        Color {
            r: 0.27105,
            g: 0.67693,
            b: 1.31640,
        },
        Color {
            r: 3.60920,
            g: 2.62480,
            b: 2.29210,
        },
    )
}

fn gold() -> Material {
    Material::new_metal(
        Color {
            r: 0.18299,
            g: 0.42108,
            b: 1.37340,
        },
        Color {
            r: 3.42420,
            g: 2.34590,
            b: 1.77040,
        },
    )
}

pub fn dragon(num_samples: usize, scale: usize) -> Scene {
    let film_width: usize = 600 * scale;
    let film_height: usize = 400 * scale;

    let mut primitives = load_obj(
        "objs/xyzrgb_dragon.obj",
        Arc::new(
            // Material::new_glass(
            //     Color::from_rgb(235, 255, 240) * 0.5,
            //     Color::from_rgb(235, 255, 240),
            //     1.5,
            // ),
            // Material::new_matte(Color::from_rgb(255, 255, 0), 30.0),
            // Material::new_plastic(
            //     Color::from_rgb(255, 0, 0),
            //     Color::from_rgb(255, 255, 255),
            //     100.0,
            // ),
            gold(),
            // brass(),
        ),
    );

    primitives.push(Arc::new(Primitive::new_shape_primitive(
        Arc::new(Shape::new_sphere(Vector(0.0, 0.0, 0.0), 1000.0)),
        Arc::new(Material::new_emissive(Color::from_rgb(200, 220, 235))),
    )));

    primitives.push(Arc::new(Primitive::new_shape_primitive(
        Arc::new(Shape::new_sphere(Vector(0.0, 100.0, -150.0), 50.0)),
        Arc::new(Material::new_emissive(Color::WHITE * 3.0)),
    )));

    primitives.push(Arc::new(Primitive::new_shape_primitive(
        Arc::new(Shape::new_sphere(Vector(0.0, -10040.0, 0.0), 10000.0)),
        Arc::new(Material::new_matte(Color::from_rgb(255, 255, 255), 0.0)),
    )));

    Scene::new(
        32,
        film_width,
        film_height,
        Box::new(Camera::new_projection_camera(
            Vector(150.0, 20.0, -150.0),
            Vector(30.0, -10.0, 0.0),
            Vector::Y,
            1.0,
            num_samples,
            film_width,
            film_height,
        )),
        primitives,
    )
}

pub fn suzanne(num_samples: usize, scale: usize) -> Scene {
    let film_width: usize = 400 * scale;
    let film_height: usize = 400 * scale;

    let mut primitives = load_obj(
        "objs/suzanne.obj",
        Arc::new(
            // Material::new_glass(Color::WHITE, Color::WHITE, 1.5),
            // Material::new_matte(Color::from_rgb(255, 0, 0), 0.0),
            Material::new_plastic(
                Color::from_rgb(255, 0, 0),
                Color::from_rgb(255, 255, 255),
                0.0,
            ),
        ),
    );

    primitives.push(Arc::new(Primitive::new_shape_primitive(
        Arc::new(Shape::new_sphere(Vector(0.0, 0.0, 0.0), 1000.0)),
        Arc::new(Material::new_emissive(Color::from_rgb(230, 252, 255))),
    )));

    primitives.push(Arc::new(Primitive::new_shape_primitive(
        Arc::new(Shape::new_sphere(Vector(0.0, -10001.0, 0.0), 10000.0)),
        Arc::new(Material::new_matte(Color::from_rgb(44, 33, 255), 0.0)),
    )));

    Scene::new(
        8,
        film_width,
        film_height,
        Box::new(Camera::new_projection_camera(
            Vector(0.75, 0.75, 3.0),
            Vector(0.0, 0.0, 0.0),
            Vector::Y,
            1.0,
            num_samples,
            film_width,
            film_height,
        )),
        primitives,
    )
}
