use rand::{Rng, SeedableRng};
use std::{sync::Arc, vec};

use crate::{
    camera::ProjectionCamera,
    color::Color,
    material::{
        EmissiveMaterial, GlassMaterial, Material, MatteMaterial, MetalMaterial, PlasticMaterial,
    },
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
        emittance: Color::from_rgb(0, 10, 60) * 2.0,
    });
    let ground_material = Arc::new(MatteMaterial::new(Color::WHITE * 0.8, 0.0));
    let glass_material = Arc::new(GlassMaterial::new(Color::WHITE, Color::WHITE * 0.6, 1.75));
    let light_material = Arc::new(EmissiveMaterial {
        emittance: Color::from_rgb(255, 230, 20) * 2.0,
    });

    Scene::new(
        8,
        film_width,
        film_height,
        Box::new(ProjectionCamera::new(
            Vector(0.0, 8.0, -10.0),
            Vector(1.0, 1.25, 12.0),
            Vector::Y,
            5.0,
            num_samples,
            film_width,
            film_height,
        )),
        vec![
            // Sky
            Arc::new(ShapePrimitive {
                shape: Box::new(Sphere::new(Vector(0.0, 0.0, 0.0), 1000.0)),
                material: sky_material,
            }),
            // Ground
            Arc::new(ShapePrimitive {
                shape: Box::new(Sphere::new(Vector(0.0, -10000.0, 10.0), 10000.0)),
                material: ground_material,
            }),
            Arc::new(ShapePrimitive {
                shape: Box::new(Sphere::new(Vector(0.0, 1.5, 12.5), 1.5)),
                material: glass_material,
            }),
            // Light
            Arc::new(ShapePrimitive {
                shape: Box::new(Sphere::new(Vector(-3.0, 4.0, 13.5), 0.5)),
                material: light_material,
            }),
        ],
    )
}

pub fn random_spheres(num_samples: usize, scale: usize) -> Scene {
    let film_width: usize = 600 * scale;
    let film_height: usize = 400 * scale;

    let seed = [19; 32];
    let mut rng = rand::rngs::StdRng::from_seed(seed);

    let mut primitives: Vec<Arc<dyn Primitive>> = vec![
        // Sky
        Arc::new(ShapePrimitive {
            shape: Box::new(Sphere::new(Vector(0.0, 0.0, 10.0), 1000.0)),
            material: Arc::new(EmissiveMaterial {
                emittance: Color::from_rgb(240, 245, 255),
            }),
        }),
        // Ground
        Arc::new(ShapePrimitive {
            shape: Box::new(Sphere::new(Vector(0.0, -1000.0, 10.0), 1000.0)),
            material: Arc::new(MatteMaterial::new(Color::from_rgb(200, 180, 150), 0.0)),
        }),
    ];

    for x in -2..2 {
        for z in 6..14 {
            let radius = rng.gen_range(0.15..0.3);
            let material: Arc<dyn Material> = match rng.gen_range(0..5) {
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
                    Arc::new(MetalMaterial::new(eta, k))
                }
                3..=4 => {
                    let color = Color::from_rgb(
                        rng.gen_range(128..255),
                        rng.gen_range(128..255),
                        rng.gen_range(128..255),
                    );
                    Arc::new(GlassMaterial::new(color, color, rng.gen_range(1.0..3.0)))
                }
                _ => Arc::new(MatteMaterial::new(
                    Color::from_rgb(
                        rng.gen_range(0..255),
                        rng.gen_range(0..255),
                        rng.gen_range(0..255),
                    ),
                    rng.gen_range(0.0..1000.0),
                )),
            };
            primitives.push(Arc::new(ShapePrimitive {
                shape: Box::new(Sphere::new(
                    Vector(x as f64, radius, z as f64)
                        + Vector(rng.gen_range(0.0..0.6), 0.0, rng.gen_range(0.0..0.3)),
                    radius,
                )),
                material,
            }));
        }
    }

    Scene::new(
        4,
        film_width,
        film_height,
        Box::new(ProjectionCamera::new(
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

fn brass() -> MetalMaterial {
    MetalMaterial::new(
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

fn chrome() -> MetalMaterial {
    MetalMaterial::new(
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

fn copper() -> MetalMaterial {
    MetalMaterial::new(
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

fn gold() -> MetalMaterial {
    MetalMaterial::new(
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
            // GlassMaterial::new(
            //     Color::from_rgb(235, 255, 240) * 0.5,
            //     Color::from_rgb(235, 255, 240),
            //     1.5,
            // ),
            // MatteMaterial::new(Color::from_rgb(255, 255, 0), 30.0),
            // PlasticMaterial::new(
            //     Color::from_rgb(255, 0, 0),
            //     Color::from_rgb(255, 255, 255),
            //     100.0,
            // ),
            gold(),
            // brass(),
        ),
    );

    primitives.push(Arc::new(ShapePrimitive {
        shape: Box::new(Sphere::new(Vector(0.0, 0.0, 0.0), 1000.0)),
        material: Arc::new(EmissiveMaterial {
            emittance: Color::from_rgb(200, 220, 235),
        }),
    }));

    primitives.push(Arc::new(ShapePrimitive {
        shape: Box::new(Sphere::new(Vector(0.0, 100.0, -150.0), 50.0)),
        material: Arc::new(EmissiveMaterial {
            emittance: Color::WHITE * 3.0,
        }),
    }));

    primitives.push(Arc::new(ShapePrimitive {
        shape: Box::new(Sphere::new(Vector(0.0, -10040.0, 0.0), 10000.0)),
        material: Arc::new(MatteMaterial::new(Color::from_rgb(255, 255, 255), 0.0)),
    }));

    Scene::new(
        32,
        film_width,
        film_height,
        Box::new(ProjectionCamera::new(
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
            // GlassMaterial::new(Color::WHITE, Color::WHITE, 1.5),
            // MatteMaterial::new(Color::from_rgb(255, 0, 0), 0.0),
            PlasticMaterial::new(
                Color::from_rgb(255, 0, 0),
                Color::from_rgb(255, 255, 255),
                0.0,
            ),
        ),
    );

    primitives.push(Arc::new(ShapePrimitive {
        shape: Box::new(Sphere::new(Vector(0.0, 0.0, 0.0), 1000.0)),
        material: Arc::new(EmissiveMaterial {
            emittance: Color::from_rgb(230, 252, 255),
        }),
    }));

    primitives.push(Arc::new(ShapePrimitive {
        shape: Box::new(Sphere::new(Vector(0.0, -10001.0, 0.0), 10000.0)),
        material: Arc::new(MatteMaterial::new(Color::from_rgb(44, 33, 255), 0.0)),
    }));

    Scene::new(
        8,
        film_width,
        film_height,
        Box::new(ProjectionCamera::new(
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
