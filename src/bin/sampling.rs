use clap::Parser;
use craytracer::{
    geometry::vector::Vector,
    n, p,
    sampling::{
        samplers::{IndependentSampler, Sampler, SobolSampler, UniformSampler},
        sampling_fns::sample_disk,
        sampling_fns::sample_hemisphere,
        sampling_fns::sample_triangle,
    },
};
use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};
use rand::{rngs::SmallRng, Rng, SeedableRng};

// For verifying sampling methods
fn draw_samples<F>(num_samples: usize, mut sample_fn: F)
where
    F: FnMut(usize) -> (f64, f64, f64),
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
    let mut sample_index = 0;

    while !window.is_key_released(Key::Escape) {
        if sample_index < num_samples {
            let (mut x, mut y, mut z) = sample_fn(sample_index);
            sample_index += 1;
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
        }
        window
            .update_with_buffer(&buffer, width as usize, height as usize)
            .unwrap();
    }
}

fn test_sampler_quality<S>(sampler: &mut S, width: usize, height: usize, spp: usize)
where
    S: Sampler,
{
    let mut u = 0.0;
    let mut v = 0.0;
    let mut w = 0.0;

    for x in 0..width {
        for y in 0..height {
            for i in 0..spp {
                sampler.start_pixel(x, y, i);
                let (x, y) = sampler.sample_2d().take();
                let z = sampler.sample_1d().take();
                for f in [x, y, z] {
                    assert!(f >= 0.0);
                    assert!(f < 1.0);
                }
                u += x;
                v += y;
                w += z;
            }
        }
    }
    let num_samples = (spp * width * height) as f64;
    u /= num_samples;
    v /= num_samples;
    println!("mean sample_2d: ({u}, {v})");
    w /= num_samples;
    println!("mean sample_1d: {w}");
}

fn visualize_samples<S>(sample_fn: &str, sampler: &mut S)
where
    S: Sampler,
{
    let mut rng = SmallRng::from_entropy();
    match sample_fn {
        "square" => draw_samples(sampler.num_samples(), move |sample_index| {
            sampler.start_pixel(0, 0, sample_index);
            let (x, y) = sampler.sample_2d().take();
            (2.0 * x - 1.0, 2.0 * y - 1.0, 0.0)
        }),
        "hemisphere" => {
            let normal = n!(
                rng.gen_range(-1.0..=1.0),
                rng.gen_range(-1.0..=1.0),
                rng.gen_range(-1.0..=1.0)
            )
            .normalized();

            draw_samples(sampler.num_samples(), move |sample_index| {
                sampler.start_pixel(0, 0, sample_index);
                let Vector(x, y, z) = sample_hemisphere(sampler.sample_2d(), &normal);
                (x, y, z)
            })
        }
        "disk" => draw_samples(sampler.num_samples(), move |sample_index| {
            sampler.start_pixel(0, 0, sample_index);
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

            draw_samples(sampler.num_samples(), move |sample_index| {
                sampler.start_pixel(0, 0, sample_index);
                let (b0, b1) = sample_triangle(sampler.sample_2d());
                let p = p0 + (p1 - p0) * b0 + (p2 - p0) * b1;
                (p.x(), p.y(), p.z())
            })
        }
        name => panic!("Unknown sampling fn: {}", name),
    };
}

#[derive(Parser)]
struct Cli {
    #[clap(long, default_value_t = String::from("uniform"))]
    sampler: String,

    #[clap(long, default_value_t = String::from("square"))]
    sample_fn: String,

    #[clap(long, default_value_t = String::from("visual"))]
    test: String,

    #[clap(long, default_value_t = 64)]
    spp_x: usize,

    #[clap(long, default_value_t = 64)]
    spp_y: usize,
}

fn main() {
    env_logger::init();

    let Cli {
        sampler,
        sample_fn,
        test,
        spp_x,
        spp_y,
    } = Cli::parse();

    let width = 10;
    let height = 10;
    let seed = 0;
    let spp = spp_x * spp_y;

    match test.as_str() {
        "quality" => {
            match sampler.as_str() {
                "uniform" => {
                    test_sampler_quality(&mut UniformSampler::new(spp_x, spp_y), width, height, spp)
                }
                "independent" => test_sampler_quality(
                    &mut IndependentSampler::new(seed, spp),
                    width,
                    height,
                    spp,
                ),
                "sobol" => {
                    test_sampler_quality(&mut SobolSampler::new(seed, spp), width, height, spp)
                }
                _ => panic!("Unknown sampler: {}", sampler),
            };
        }
        "visual" => {
            match sampler.as_str() {
                "uniform" => {
                    visualize_samples(sample_fn.as_str(), &mut UniformSampler::new(spp_x, spp_y))
                }
                "independent" => {
                    visualize_samples(sample_fn.as_str(), &mut IndependentSampler::new(seed, spp))
                }
                "sobol" => visualize_samples(sample_fn.as_str(), &mut SobolSampler::new(seed, spp)),
                _ => panic!("Unknown sampler: {}", sampler),
            };
        }
        _ => panic!("Unknown test: {}", test),
    }
}
