use craytracer::{geometry::normal::Normal, n, sampling::sample_hemisphere};
use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};
use rand::{rngs::SmallRng, SeedableRng};

// For verifying sampling methods
fn draw_samples() {
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
    let normal = n!(0, 0, 1).normalized();
    let mut rng = SmallRng::from_entropy();

    while !window.is_key_released(Key::Escape) {
        for _ in 0..10 {
            let v = sample_hemisphere(&mut rng, &normal);
            let x = v.0;
            let y = v.1;
            let bx = ((x + 1.0) * 0.5 * (width - 1) as f64).round();
            let by = ((y + 1.0) * 0.5 * (height - 1) as f64).round();
            assert!(bx >= 0.0 && bx < width as f64);
            assert!(by >= 0.0 && by < height as f64);
            buffer[(bx + by * width as f64) as usize] = 0xFFFF0000;
        }
        window.update_with_buffer(&buffer, width, height).unwrap();
    }
}

fn main() {
    draw_samples();
}
