use std::vec;

use rand::{Rng, SeedableRng};

use crate::{
    camera::ProjectionCamera,
    color::Color,
    material::{Glass, LambertianMaterial, Material, Mirror, EmissiveMaterial},
    scene::Scene,
    shape::{Shape, Sphere},
    vector::Vector,
};

#[allow(dead_code)]
pub fn simple(num_samples: usize, scale: usize) -> Scene {
    let film_width: usize = 896 * scale;
    let film_height: usize = 560 * scale;

    Scene {
        max_depth: 8,
        film_width,
        film_height,
        camera: Box::new(ProjectionCamera::new(
            Vector(0.0, 8.0, -10.0),
            Vector(0.0, 1.5, 12.0),
            Vector::Y,
            5.0,
            num_samples,
            film_width,
            film_height,
        )),
        shapes: vec![
            // Sky
            Box::new(Sphere {
                origin: Vector(0.0, 0.0, 0.0),
                radius: 1000.0,
                material: Box::new(EmissiveMaterial {
                    emittance: Color::from_rgb(0, 10, 60),
                }),
            }),
            // Ground
            Box::new(Sphere {
                origin: Vector(0.0, -10000.0, 10.0),
                radius: 10000.0,
                material: Box::new(LambertianMaterial {
                    reflectance: Color::WHITE,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(0.0, 1.5, 12.5),
                radius: 1.5,
                material: Box::new(
                    Glass {
                    eta: 1.8,
                    transmittance: Color::from_rgb(240, 250, 255),
                }
            ),
            }),
            // Light
            Box::new(Sphere {
                origin: Vector(-3.0, 4.0, 13.5),
                radius: 0.5,
                material: Box::new(EmissiveMaterial {
                    emittance: Color::from_rgb(255, 230, 180) * 2.0,
                }),
            }),
        ],
    }
}

#[allow(dead_code)]
pub fn random_spheres(num_samples: usize, scale: usize) -> Scene {
    let film_width: usize = 600 * scale;
    let film_height: usize = 400 * scale;

    let seed = [19; 32];
    let mut rng = rand::rngs::StdRng::from_seed(seed);

    let mut shapes: Vec<Box<dyn Shape>> = vec![
        // Sky
        Box::new(Sphere {
            origin: Vector(0.0, 0.0, 10.0),
            radius: 1000.0,
            material: Box::new(EmissiveMaterial {
                emittance: Color::from_rgb(240, 245, 255),
            }),
        }),
        // Ground
        Box::new(Sphere {
            origin: Vector(0.0, -1000.0, 10.0),
            radius: 1000.0,
            material: Box::new(LambertianMaterial {
                reflectance: Color::from_rgb(200, 180, 150),
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
pub fn logo(num_samples: usize, scale: usize) -> Scene {
    let film_width: usize = 400 * scale;
    let film_height: usize = 235 * scale;

    let blue = Color::from_rgb(66, 133, 244);
    let red = Color::from_rgb(219, 68, 55);
    let yellow = Color::from_rgb(244, 180, 0);
    let green = Color::from_rgb(15, 157, 88);

    Scene {
        max_depth: 3,
        film_width,
        film_height,
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
            // Ground
            Box::new(Sphere {
                origin: Vector(0.0, -1001.0, 10.0),
                radius: 1000.0,
                material: Box::new(LambertianMaterial {
                    reflectance: Color::WHITE,
                }),
            }),
            // Sky
            Box::new(Sphere {
                origin: Vector(0.0, 0.0, 0.0),
                radius: 1000.0,
                material: Box::new(EmissiveMaterial {
                    emittance: Color::WHITE,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(-3.0, 1.0, 10.0),
                radius: 1.0,
                material: Box::new(LambertianMaterial { reflectance: blue }),
            }),
            Box::new(Sphere {
                origin: Vector(-1.5, 0.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial { reflectance: red }),
            }),
            Box::new(Sphere {
                origin: Vector(-0.5, 0.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial {
                    reflectance: yellow,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(0.5, 0.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial { reflectance: blue }),
            }),
            Box::new(Sphere {
                origin: Vector(0.5, -0.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial { reflectance: blue }),
            }),
            Box::new(Sphere {
                origin: Vector(1.5, 0.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial { reflectance: green }),
            }),
            Box::new(Sphere {
                origin: Vector(1.5, 0.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial { reflectance: green }),
            }),
            Box::new(Sphere {
                origin: Vector(1.5, 1.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial { reflectance: green }),
            }),
            Box::new(Sphere {
                origin: Vector(2.5, 0.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial { reflectance: red }),
            }),
        ],
    }
}
