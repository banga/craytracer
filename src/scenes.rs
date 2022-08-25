use std::vec;

use rand::{Rng, SeedableRng};

use crate::{
    camera::ProjectionCamera,
    color::Color,
    material::{EmissiveMaterial, Glass, LambertianMaterial, Material, Mirror},
    scene::Scene,
    shape::{Shape, Sphere},
    vector::Vector,
};

#[allow(dead_code)]
pub fn simple(num_samples: usize) -> Scene {
    let film_width: usize = 600;
    let film_height: usize = 400;

    Scene {
        max_depth: 8,
        film_width,
        film_height,
        background: Color::from_rgb(0, 10, 60),
        camera: Box::new(ProjectionCamera::new(
            Vector(0.0, 3.0, -10.0),
            Vector(0.0, 1.5, 10.0),
            Vector::Y,
            4.0,
            num_samples,
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
pub fn random_spheres(num_samples: usize) -> Scene {
    let film_width: usize = 600;
    let film_height: usize = 400;

    let seed = [19; 32];
    let mut rng = rand::rngs::StdRng::from_seed(seed);

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
                4 => Box::new(EmissiveMaterial {
                    emittance: Color::from_rgb(
                        rng.gen_range(0..255),
                        rng.gen_range(0..255),
                        rng.gen_range(0..255),
                    ) * 10.0,
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
                origin: Vector(x as f64, radius, z as f64)
                    + Vector(rng.gen_range(0.0..0.6), 0.0, rng.gen_range(0.0..0.3)),
                radius,
                material,
            }));
        }
    }

    Scene {
        max_depth: 8,
        film_width,
        film_height,
        background: Color::from_rgb(80, 88, 90),
        camera: Box::new(ProjectionCamera::new(
            Vector(2.0, 2.0, 0.0),
            Vector(0.0, 0.0, 10.0),
            Vector::Y,
            5.0,
            num_samples,
            film_width,
            film_height,
        )),
        shapes,
    }
}

#[allow(dead_code)]
pub fn logo(num_samples: usize) -> Scene {
    let film_width: usize = 800;
    let film_height: usize = 470;

    let blue = Color::from_rgb(66, 133, 244);
    let red = Color::from_rgb(219, 68, 55);
    let yellow = Color::from_rgb(244, 180, 0);
    let green = Color::from_rgb(15, 157, 88);

    Scene {
        max_depth: 3,
        film_width,
        film_height,
        background: Color::WHITE,
        camera: Box::new(ProjectionCamera::new(
            Vector(-10.0, 2.0, -10.0),
            Vector(0.0, 1.0, 10.0),
            Vector::Y,
            4.0,
            num_samples,
            film_width,
            film_height,
        )),
        shapes: vec![
            Box::new(Sphere {
                origin: Vector(-3.0, 1.0, 10.0),
                radius: 1.0,
                material: Box::new(LambertianMaterial {
                    reflectance: blue,
                    num_samples: 1,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(-1.5, 0.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial {
                    reflectance: red,
                    num_samples: 1,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(-0.5, 0.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial {
                    reflectance: yellow,
                    num_samples: 1,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(0.5, 0.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial {
                    reflectance: blue,
                    num_samples: 1,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(0.5, -0.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial {
                    reflectance: blue,
                    num_samples: 1,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(1.5, 0.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial {
                    reflectance: green,
                    num_samples: 1,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(1.5, 0.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial {
                    reflectance: green,
                    num_samples: 1,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(1.5, 1.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial {
                    reflectance: green,
                    num_samples: 1,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(2.5, 0.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial {
                    reflectance: red,
                    num_samples: 1,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(0.0, -1001.0, 10.0),
                radius: 1000.0,
                material: Box::new(LambertianMaterial {
                    reflectance: Color::WHITE,
                    num_samples: 1,
                }),
            }),
        ],
    }
}
