use std::vec;

use rand::Rng;

use crate::{
    camera::ProjectionCamera,
    color::Color,
    material::{Glass, LambertianMaterial, Material, Mirror, EmissiveMaterial},
    scene::Scene,
    shape::{Shape, Sphere},
    vector::Vector,
};

#[allow(dead_code)]
pub fn simple() -> Scene {
    // let num_camera_samples: usize = 256;
    // let film_width: usize = 3072;
    // let film_height: usize = 1920;
    let num_camera_samples: usize = 64;
    let film_width: usize = 600;
    let film_height: usize = 400;

    Scene {
        max_depth: 8,
        gamma: 2.2,
        film_width,
        film_height,
        background: Color::from_rgb(0, 10, 60),
        camera: Box::new(ProjectionCamera::new(
            Vector(0.0, 3.0, -10.0),
            Vector(0.0, 1.5, 10.0),
            Vector::Y,
            4.0,
            num_camera_samples,
            film_width,
            film_height,
        )),
        shapes: vec![
            Box::new(Sphere {
                origin: Vector(0.0, 1.5, 12.5),
                radius: 1.5,
                material: Box::new(Glass {
                    eta: 2.4,
                    transmittance: Color::from_rgb(240, 250, 255),
                }),
            }),
            Box::new(Sphere {
                origin: Vector(-2.5, 4.0, 13.0),
                radius: 0.5,
                material: Box::new(EmissiveMaterial {
                    emittance: Color::from_rgb(255, 230, 120),
                }),
            }),
            Box::new(Sphere {
                origin: Vector(0.0, -10000.0, 10.0),
                radius: 10000.0,
                material: Box::new(LambertianMaterial {
                    reflectance: Color::WHITE,
                    num_samples: 8,
                }),
            }),
        ],
    }
}

#[allow(dead_code)]
pub fn random_spheres() -> Scene {
    let num_camera_samples: usize = 64;
    let film_width: usize = 500;
    let film_height: usize = 500;

    let mut rng = rand::thread_rng();

    let mut shapes: Vec<Box<dyn Shape>> = vec![
        // Ground
        Box::new(Sphere {
            origin: Vector(0.0, -1000.0, 10.0),
            radius: 1000.0,
            material: Box::new(LambertianMaterial {
                reflectance: Color::from_rgb(250, 255, 250),
                num_samples: 1,
            }),
        }),
    ];

    for x in -2..2 {
        for z in 6..14 {
            let radius = rng.gen_range(0.15..0.3);
            let material: Box<dyn Material> = match rng.gen_range(0..10) {
                0 => Box::new(Mirror {
                    reflectance: Color::from_rgb(
                        rng.gen_range(0..255),
                        rng.gen_range(0..255),
                        rng.gen_range(0..255),
                    ),
                }),
                1..=3 => Box::new(Glass {
                    transmittance: Color::from_rgb(
                        rng.gen_range(128..255),
                        rng.gen_range(128..255),
                        rng.gen_range(128..255),
                    ),
                    eta: 1.0 + rng.gen::<f64>(),
                }),
                _ => Box::new(LambertianMaterial {
                    reflectance: Color::from_rgb(
                        rng.gen_range(0..255),
                        rng.gen_range(0..255),
                        rng.gen_range(0..255),
                    ),
                    num_samples: 1,
                }),
            };
            shapes.push(Box::new(Sphere {
                origin: Vector(x as f64, radius, z as f64) + Vector(rng.gen(), 0.0, rng.gen()),
                radius,
                material,
            }));
        }
    }

    Scene {
        max_depth: 5,
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
