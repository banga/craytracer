use image::write_ppm;
use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};

use minifb::{Key, Window, WindowOptions};
use rayon::prelude::*;
use scene::Scene;
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};
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

#[allow(dead_code)]
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

#[allow(dead_code)]
fn render_with_preview(scene: Scene) {
    let width = scene.film_width;
    let height = scene.film_height;
    let tile_width = 64;
    let tile_height = 64;

    let mut buffer = vec![0u32; width * height];
    let mut tiles = Vec::<(usize, usize)>::new();
    for ty in (0..height).step_by(tile_height as usize) {
        for tx in (0..width).step_by(tile_width as usize) {
            tiles.push((tx, ty));

            let tile_color = if ((tx / tile_width) + (ty / tile_height)) % 2 == 0 {
                0x999999
            } else {
                0xaaaaaa
            };
            for y in ty..(ty + tile_height).min(height) {
                for x in tx..(tx + tile_width).min(width) {
                    buffer[x + y * width] = tile_color;
                }
            }
        }
    }

    let mut window = Window::new("craytracer", width, height, WindowOptions::default()).unwrap();
    window.update_with_buffer(&buffer, width, height).unwrap();

    let tiles = Arc::new(tiles);
    let tile_index = Arc::new(AtomicUsize::new(0));
    let tile_count = Arc::new(AtomicUsize::new(0));
    let buffer = Arc::new(Mutex::new(buffer));
    let scene = Arc::new(scene);

    let num_threads = num_cpus::get();
    let _handles: &Vec<thread::JoinHandle<()>> = &(0..num_threads)
        .into_iter()
        .map(|_| {
            let tile_index = Arc::clone(&tile_index);
            let tile_count = Arc::clone(&tile_count);
            let tiles = Arc::clone(&tiles);
            let buffer = Arc::clone(&buffer);
            let scene = Arc::clone(&scene);

            thread::spawn(move || loop {
                let tile_index = tile_index.fetch_add(1, Ordering::SeqCst);
                if tile_index >= tiles.len() {
                    break;
                }

                let (tx, ty) = tiles[tile_index];
                for y in ty..(ty + tile_height).min(height) {
                    for x in tx..(tx + tile_width).min(width) {
                        let color = scene.camera.sample(x, y, &scene);
                        let (r, g, b) = color.gamma_correct(scene.gamma).to_rgb();

                        let mut buffer = buffer.lock().unwrap();
                        buffer[x + y * width] = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
                    }
                }

                tile_count.fetch_add(1, Ordering::SeqCst);
            })
        })
        .collect();

    while !window.is_key_released(Key::Escape) && tile_count.load(Ordering::SeqCst) < tiles.len() {
        window
            .update_with_buffer(&buffer.lock().unwrap(), width, height)
            .unwrap();
        thread::sleep(Duration::from_millis(16));
    }

    // TODO: save buffer to file before exiting
}

fn main() {
    let scene = scenes::simple();
    render_with_rayon(scene);
    // render_with_preview(scene);
}
