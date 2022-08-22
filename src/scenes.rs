use std::vec;

use rand::Rng;

use crate::{
    camera::ProjectionCamera,
    color::Color,
    material::{LambertianMaterial, Material, Mirror},
    scene::Scene,
    shape::{Shape, Sphere},
    vector::Vector,
};

#[allow(dead_code)]
pub fn simple() -> Scene {
    let num_samples: usize = 4;
    let num_camera_samples: usize = 256;
    let film_width: usize = 800;
    let film_height: usize = 470;

    let blue = Color::from_rgb(0, 120, 255);

    Scene {
        max_depth: 3,
        gamma: 2.2,
        film_width,
        film_height,
        background: Color::WHITE,
        camera: Box::new(ProjectionCamera::new(
            Vector(0.0, 4.0, -10.0),
            Vector(0.0, 1.0, 10.0),
            Vector::Y,
            4.0,
            num_camera_samples,
            film_width,
            film_height,
        )),
        shapes: vec![
            Box::new(Sphere {
                origin: Vector(-1.0, 1.0, 10.0),
                radius: 1.0,
                material: Box::new(LambertianMaterial {
                    reflectance: blue,
                    num_samples,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(0.5, 0.5, 10.5),
                radius: 0.5,
                material: Box::new(Mirror {
                    reflectance: Color::WHITE,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(0.0, -1000.0, 10.0),
                radius: 1000.0,
                material: Box::new(LambertianMaterial {
                    reflectance: Color::WHITE,
                    num_samples,
                }),
            }),
        ],
    }
}

#[allow(dead_code)]
pub fn random_spheres() -> Scene {
    let num_samples: usize = 8;
    let num_camera_samples: usize = 256;
    let film_width: usize = 600;
    let film_height: usize = 600;

    let mut rng = rand::thread_rng();

    let mut shapes: Vec<Box<dyn Shape>> = vec![
        // Ground
        Box::new(Sphere {
            origin: Vector(0.0, -1000.0, 10.0),
            radius: 1000.0,
            material: Box::new(LambertianMaterial {
                reflectance: Color::WHITE,
                num_samples,
            }),
        }),
    ];

    for x in -2..2 {
        for z in 6..14 {
            let radius = 0.25;
            let reflectance = Color::from_rgb(
                rng.gen_range(0..255),
                rng.gen_range(0..255),
                rng.gen_range(0..255),
            );
            let material: Box<dyn Material> = if rng.gen_bool(0.9) {
                Box::new(LambertianMaterial {
                    reflectance,
                    num_samples,
                })
            } else {
                Box::new(Mirror { reflectance })
            };
            shapes.push(Box::new(Sphere {
                origin: Vector(x as f64, radius, z as f64) + Vector(rng.gen(), 0.0, rng.gen()),
                radius,
                material,
            }));
        }
    }

    Scene {
        max_depth: 3,
        gamma: 2.2,
        film_width,
        film_height,
        background: Color::WHITE,
        camera: Box::new(ProjectionCamera::new(
            Vector(0.0, 4.0, -10.0),
            Vector(0.0, 0.0, 10.0),
            Vector::Y,
            8.0,
            num_camera_samples,
            film_width,
            film_height,
        )),
        shapes,
    }
}

#[allow(dead_code)]
pub fn logo() -> Scene {
    let num_samples: usize = 4;
    let num_camera_samples: usize = 256;
    let film_width: usize = 800;
    let film_height: usize = 470;

    let blue = Color::from_rgb(66, 133, 244);
    let red = Color::from_rgb(219, 68, 55);
    let yellow = Color::from_rgb(244, 180, 0);
    let green = Color::from_rgb(15, 157, 88);

    Scene {
        max_depth: 3,
        gamma: 2.2,
        film_width: film_width,
        film_height: film_height,
        background: Color::WHITE,
        camera: Box::new(ProjectionCamera::new(
            Vector(-10.0, 2.0, -10.0),
            Vector(0.0, 1.0, 10.0),
            Vector::Y,
            4.0,
            num_camera_samples,
            film_width,
            film_height,
        )),
        shapes: vec![
            Box::new(Sphere {
                origin: Vector(-3.0, 1.0, 10.0),
                radius: 1.0,
                material: Box::new(LambertianMaterial {
                    reflectance: blue,
                    num_samples,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(-1.5, 0.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial {
                    reflectance: red,
                    num_samples,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(-0.5, 0.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial {
                    reflectance: yellow,
                    num_samples,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(0.5, 0.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial {
                    reflectance: blue,
                    num_samples,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(0.5, -0.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial {
                    reflectance: blue,
                    num_samples,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(1.5, 0.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial {
                    reflectance: green,
                    num_samples,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(1.5, 0.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial {
                    reflectance: green,
                    num_samples,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(1.5, 1.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial {
                    reflectance: green,
                    num_samples,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(2.5, 0.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial {
                    reflectance: red,
                    num_samples,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(0.0, -1001.0, 10.0),
                radius: 1000.0,
                material: Box::new(LambertianMaterial {
                    reflectance: Color::WHITE,
                    num_samples,
                }),
            }),
        ],
    }
}
