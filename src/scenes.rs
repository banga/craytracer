use rand::{Rng, SeedableRng};
use std::{sync::Arc, vec};

use crate::{
    bvh::BvhNode,
    camera::ProjectionCamera,
    color::Color,
    material::{EmissiveMaterial, Glass, LambertianMaterial, Material, Mirror},
    obj::load_obj,
    primitive::{Primitive, ShapePrimitive},
    scene::Scene,
    shape::Sphere,
    vector::Vector,
};

pub fn simple(num_samples: usize, scale: usize) -> Scene {
    let film_width: usize = 896 * scale;
    let film_height: usize = 560 * scale;

    let sky_material = Arc::new(EmissiveMaterial {
        emittance: Color::from_rgb(0, 10, 60),
    });
    let ground_material = Arc::new(LambertianMaterial {
        reflectance: Color::WHITE,
    });
    let glass_material = Arc::new(Glass {
        eta: 1.8,
        transmittance: Color::from_rgb(240, 250, 255),
    });
    let light_material = Arc::new(EmissiveMaterial {
        emittance: Color::from_rgb(255, 230, 180) * 2.0,
    });

    Scene {
        max_depth: 3,
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
        bvh: BvhNode::new(vec![
            // Sky
            Arc::new(ShapePrimitive {
                shape: Box::new(Sphere {
                    origin: Vector(0.0, 0.0, 0.0),
                    radius: 1000.0,
                }),
                material: sky_material,
            }),
            // Ground
            Arc::new(ShapePrimitive {
                shape: Box::new(Sphere {
                    origin: Vector(0.0, -10000.0, 10.0),
                    radius: 10000.0,
                }),
                material: ground_material,
            }),
            Arc::new(ShapePrimitive {
                shape: Box::new(Sphere {
                    origin: Vector(0.0, 1.5, 12.5),
                    radius: 1.5,
                }),
                material: glass_material,
            }),
            // Light
            Arc::new(ShapePrimitive {
                shape: Box::new(Sphere {
                    origin: Vector(-3.0, 4.0, 13.5),
                    radius: 0.5,
                }),
                material: light_material,
            }),
        ]),
    }
}

pub fn random_spheres(num_samples: usize, scale: usize) -> Scene {
    let film_width: usize = 600 * scale;
    let film_height: usize = 400 * scale;

    let seed = [19; 32];
    let mut rng = rand::rngs::StdRng::from_seed(seed);

    let mut primitives: Vec<Arc<dyn Primitive>> = vec![
        // Sky
        Arc::new(ShapePrimitive {
            shape: Box::new(Sphere {
                origin: Vector(0.0, 0.0, 10.0),
                radius: 1000.0,
            }),
            material: Arc::new(EmissiveMaterial {
                emittance: Color::from_rgb(240, 245, 255),
            }),
        }),
        // Ground
        Arc::new(ShapePrimitive {
            shape: Box::new(Sphere {
                origin: Vector(0.0, -1000.0, 10.0),
                radius: 1000.0,
            }),
            material: Arc::new(LambertianMaterial {
                reflectance: Color::from_rgb(200, 180, 150),
            }),
        }),
    ];

    for x in -2..2 {
        for z in 6..14 {
            let radius = rng.gen_range(0.15..0.3);
            let material: Arc<dyn Material> = match rng.gen_range(0..10) {
                0 => Arc::new(Mirror {
                    reflectance: Color::from_rgb(
                        rng.gen_range(0..255),
                        rng.gen_range(0..255),
                        rng.gen_range(0..255),
                    ),
                }),
                1..=3 => Arc::new(Glass {
                    transmittance: Color::from_rgb(
                        rng.gen_range(128..255),
                        rng.gen_range(128..255),
                        rng.gen_range(128..255),
                    ),
                    eta: 1.0 + rng.gen::<f64>(),
                }),
                4 => Arc::new(EmissiveMaterial {
                    emittance: Color::from_rgb(
                        rng.gen_range(0..255),
                        rng.gen_range(0..255),
                        rng.gen_range(0..255),
                    ) * 10.0,
                }),
                _ => Arc::new(LambertianMaterial {
                    reflectance: Color::from_rgb(
                        rng.gen_range(0..255),
                        rng.gen_range(0..255),
                        rng.gen_range(0..255),
                    ),
                }),
            };
            primitives.push(Arc::new(ShapePrimitive {
                shape: Box::new(Sphere {
                    origin: Vector(x as f64, radius, z as f64)
                        + Vector(rng.gen_range(0.0..0.6), 0.0, rng.gen_range(0.0..0.3)),
                    radius,
                }),
                material,
            }));
        }
    }

    Scene {
        max_depth: 4,
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
        bvh: BvhNode::new(primitives),
    }
}

pub fn obj(num_samples: usize, scale: usize) -> Scene {
    let film_width: usize = 400 * scale;
    let film_height: usize = 400 * scale;

    let mut primitives = load_obj(
        "objs/xyzrgb_dragon.obj",
        Arc::new(
            Glass {
                eta: 1.5,
                transmittance: Color::from_rgb(148, 177, 160),
            },
            // LambertianMaterial {
            //     reflectance: Color::from_rgb(200, 255, 200),
            // },
        ),
    );

    primitives.push(Arc::new(ShapePrimitive {
        shape: Box::new(Sphere {
            origin: Vector(0.0, 0.0, 0.0),
            radius: 1000.0,
        }),
        material: Arc::new(EmissiveMaterial {
            emittance: Color::WHITE,
        }),
    }));

    // primitives.push(Arc::new(ShapePrimitive {
    //     shape: Box::new(Sphere {
    //         origin: Vector(0.0, 150.0, 0.0),
    //         radius: 50.0,
    //     }),
    //     material: Arc::new(EmissiveMaterial {
    //         emittance: Color::from_rgb(255, 225, 100) * 2.0,
    //     }),
    // }));

    primitives.push(Arc::new(ShapePrimitive {
        shape: Box::new(Sphere {
            origin: Vector(0.0, -10040.0, 0.0),
            radius: 10000.0,
        }),
        material: Arc::new(LambertianMaterial {
            reflectance: Color::WHITE * 0.75,
        }),
    }));

    Scene {
        max_depth: 8,
        film_width,
        film_height,
        camera: Box::new(ProjectionCamera::new(
            Vector(150.0, 20.0, -150.0),
            Vector(30.0, -10.0, 0.0),
            Vector::Y,
            1.0,
            num_samples,
            film_width,
            film_height,
        )),
        bvh: BvhNode::new(primitives),
    }
}
