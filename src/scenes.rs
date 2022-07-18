use std::vec;

use rand::Rng;

use crate::{
    camera::ProjectionCamera,
    material::{LambertianMaterial, Material, Mirror},
    scene::Scene,
    shape::{Shape, Sphere},
    vector::Vector,
};

const NUM_SAMPLES: usize = 4;
const NUM_CAMERA_SAMPLES: usize = 32;
const FILM_WIDTH: usize = 600;
const FILM_HEIGHT: usize = 600;

#[allow(dead_code)]
pub fn simple() -> Scene {
    let blue = Vector(0.0, 120.0, 255.0) / 255.0;

    Scene {
        max_depth: 3,
        gamma: 2.2,
        film_width: FILM_WIDTH,
        film_height: FILM_HEIGHT,
        background: Vector(1.0, 1.0, 1.0),
        camera: Box::new(ProjectionCamera::new(
            Vector(0.0, 4.0, -10.0),
            Vector(0.0, 1.0, 10.0),
            Vector::Y,
            4.0,
            NUM_CAMERA_SAMPLES,
            FILM_WIDTH,
            FILM_HEIGHT,
        )),
        shapes: vec![
            Box::new(Sphere {
                origin: Vector(-1.0, 1.0, 10.0),
                radius: 1.0,
                material: Box::new(LambertianMaterial {
                    reflectance: blue,
                    num_samples: NUM_SAMPLES,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(0.5, 0.5, 10.5),
                radius: 0.5,
                material: Box::new(Mirror {
                    reflectance: Vector(1.0, 1.0, 1.0),
                }),
            }),
            Box::new(Sphere {
                origin: Vector(0.0, -1000.0, 10.0),
                radius: 1000.0,
                material: Box::new(LambertianMaterial {
                    reflectance: Vector(1.0, 1.0, 1.0),
                    num_samples: NUM_SAMPLES,
                }),
            }),
        ],
    }
}

#[allow(dead_code)]
pub fn random_spheres() -> Scene {
    let mut rng = rand::thread_rng();

    let mut shapes: Vec<Box<dyn Shape>> = vec![
        // Ground
        Box::new(Sphere {
            origin: Vector(0.0, -1000.0, 10.0),
            radius: 1000.0,
            material: Box::new(LambertianMaterial {
                reflectance: Vector(1.0, 1.0, 1.0),
                num_samples: NUM_SAMPLES,
            }),
        }),
    ];

    for x in -4..4 {
        for z in 0..15 {
            let radius = 0.25;
            let reflectance = Vector(rng.gen(), rng.gen(), rng.gen());
            let material: Box<dyn Material> = if rng.gen_bool(0.9) {
                Box::new(LambertianMaterial {
                    reflectance,
                    num_samples: NUM_SAMPLES,
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
        film_width: FILM_WIDTH,
        film_height: FILM_HEIGHT,
        background: Vector(1.0, 1.0, 1.0),
        camera: Box::new(ProjectionCamera::new(
            Vector(0.0, 4.0, -10.0),
            Vector(0.0, 0.0, 10.0),
            Vector::Y,
            8.0,
            NUM_CAMERA_SAMPLES,
            FILM_WIDTH,
            FILM_HEIGHT,
        )),
        shapes,
    }
}

#[allow(dead_code)]
pub fn logo() -> Scene {
    let blue = Vector(66.0, 133.0, 244.0) / 255.0;
    let red = Vector(219.0, 68.0, 55.0) / 255.0;
    let yellow = Vector(244.0, 180.0, 0.0) / 255.0;
    let green = Vector(15.0, 157.0, 88.0) / 255.0;

    Scene {
        max_depth: 3,
        gamma: 2.2,
        film_width: FILM_WIDTH,
        film_height: FILM_HEIGHT,
        background: Vector(1.0, 1.0, 1.0),
        camera: Box::new(ProjectionCamera::new(
            Vector(-10.0, 2.0, -10.0),
            Vector(0.0, 1.0, 10.0),
            Vector::Y,
            4.0,
            NUM_CAMERA_SAMPLES,
            FILM_WIDTH,
            FILM_HEIGHT,
        )),
        shapes: vec![
            Box::new(Sphere {
                origin: Vector(-3.0, 1.0, 10.0),
                radius: 1.0,
                material: Box::new(LambertianMaterial {
                    reflectance: blue,
                    num_samples: NUM_SAMPLES,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(-1.5, 0.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial {
                    reflectance: red,
                    num_samples: NUM_SAMPLES,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(-0.5, 0.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial {
                    reflectance: yellow,
                    num_samples: NUM_SAMPLES,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(0.5, 0.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial {
                    reflectance: blue,
                    num_samples: NUM_SAMPLES,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(0.5, -0.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial {
                    reflectance: blue,
                    num_samples: NUM_SAMPLES,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(1.5, 0.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial {
                    reflectance: green,
                    num_samples: NUM_SAMPLES,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(1.5, 0.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial {
                    reflectance: green,
                    num_samples: NUM_SAMPLES,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(1.5, 1.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial {
                    reflectance: green,
                    num_samples: NUM_SAMPLES,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(2.5, 0.5, 10.0),
                radius: 0.5,
                material: Box::new(LambertianMaterial {
                    reflectance: red,
                    num_samples: NUM_SAMPLES,
                }),
            }),
            Box::new(Sphere {
                origin: Vector(0.0, -1001.0, 10.0),
                radius: 1000.0,
                material: Box::new(LambertianMaterial {
                    reflectance: Vector(1.0, 1.0, 1.0),
                    num_samples: NUM_SAMPLES,
                }),
            }),
        ],
    }
}
