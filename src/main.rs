use image::write_ppm;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};

use rayon::prelude::*;
use scene::Scene;
use std::sync::Arc;
use trace::trace;

mod camera;
mod color;
mod constants;
mod image;
mod intersection;
mod material;
mod ray;
mod sampling;
mod scene;
mod scenes;
mod shape;
mod trace;
mod vector;

fn render_with_rayon(scene: Scene) {
    let width = scene.film_width;
    let height = scene.film_height;
    let num_pixels = width * height;

    // Progress bar
    let pb = ProgressBar::new(num_pixels as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} {pos}/{len} pixels {wide_bar} [{elapsed} / {duration}]"),
    );
    pb.set_message(format!("{}x{}x{}", width, height, scene.max_depth));

    // Rendering
    let scene = Arc::new(scene);
    let pixels: Vec<u8> = (0..num_pixels)
        .into_par_iter()
        .progress_with(pb)
        .map(|pixel| {
            let x = pixel % width;
            let y = pixel / width;
            let color = scene.camera.sample(x, y, &scene);
            let (r, g, b) = color.gamma_correct(scene.gamma).to_rgb();
            [r, g, b]
        })
        .flatten_iter()
        .collect();

    write_ppm("out.ppm", pixels, width, height).expect("Failed to write out.ppm");
}

fn main() {
    let scene = scenes::random_spheres();
    render_with_rayon(scene);
}
