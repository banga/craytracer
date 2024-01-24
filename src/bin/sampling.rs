use std::env::args;

use craytracer::{
    geometry::{normal::Normal, point::Point, vector::Vector},
    n, p,
    sampling::{
        samplers::{IndependentSampler, Sampler},
        sampling_fns::sample_disk,
        sampling_fns::sample_hemisphere,
        sampling_fns::sample_triangle,
    },
};
use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};
use rand::{rngs::SmallRng, Rng, SeedableRng};

// For verifying sampling methods
fn draw_samples<F>(mut sample_fn: F)
where
    F: FnMut() -> (f64, f64, f64),
{
    let width = 400;
    let height = 400;
    let mut window = Window::new(
        "craytracer",
        width,
        height,
        WindowOptions {
            borderless: false,
            transparency: false,
            title: true,
            resize: true,
            scale: Scale::X1,
            scale_mode: ScaleMode::AspectRatioStretch,
            topmost: false,
            none: false,
        },
    )
    .unwrap();

    let mut buffer = vec![0u32; width * height];
    let offset_z = 1.5;

    while !window.is_key_released(Key::Escape) {
        let (mut x, mut y, mut z) = sample_fn();
        assert!(x >= -1.0);
        assert!(x <= 1.0);
        assert!(y >= -1.0);
        assert!(y <= 1.0);
        assert!(z >= -1.0);
        assert!(z <= 1.0);

        let r = ((z * 0.5 + 0.5) * 255.0) as u32;
        let g = 0;
        let b = 128;

        z += offset_z;
        x = x / z;
        y = y / z;

        let bx = ((x + 1.0) * 0.5 * (width as f64 - 1.0)).round() as usize;
        let by = ((y + 1.0) * 0.5 * (height as f64 - 1.0)).round() as usize;
        buffer[bx + (by * width)] = 0xFF000000 | (r << 16) | (g << 8) | b;
        window
            .update_with_buffer(&buffer, width as usize, height as usize)
            .unwrap();
    }
}

fn main() {
    let mut rng = SmallRng::from_entropy();
    let mut sampler = IndependentSampler::new(0);
    match args()
        .nth(1)
        .expect("Expected name of sampling function")
        .as_str()
    {
        "hemisphere" => draw_samples(move || {
            let normal = n!(
                rng.gen_range(-1.0..1.0),
                rng.gen_range(-1.0..1.0),
                rng.gen_range(-1.0..1.0)
            )
            .normalized();
            let Vector(x, y, z) = sample_hemisphere(sampler.sample_2d(), &normal);
            (x, y, z)
        }),
        "disk" => draw_samples(move || {
            let (x, y) = sample_disk(sampler.sample_2d());
            (x, y, 0.0)
        }),
        "triangle" => {
            let p0 = p!(
                rng.gen_range(-1.0..1.0),
                rng.gen_range(-1.0..1.0),
                rng.gen_range(-1.0..1.0)
            );
            let p1 = p!(
                rng.gen_range(-1.0..1.0),
                rng.gen_range(-1.0..1.0),
                rng.gen_range(-1.0..1.0)
            );
            let p2 = p!(
                rng.gen_range(-1.0..1.0),
                rng.gen_range(-1.0..1.0),
                rng.gen_range(-1.0..1.0)
            );

            draw_samples(move || {
                let (b0, b1) = sample_triangle(sampler.sample_2d());
                let p = p0 + (p1 - p0) * b0 + (p2 - p0) * b1;
                (p.x(), p.y(), p.z())
            })
        }
        name => panic!("Unknown sampling fn: {}", name),
    };
}
