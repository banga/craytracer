use rand::Rng;

use crate::{
    film::Film,
    geometry::{point::Point, vector::Vector},
    ray::Ray,
    sampling::sample_2d,
    transformation::{Transformable, Transformation},
};

#[derive(Debug, PartialEq)]
pub enum CameraType {
    Perspective,
    Orthographic,
}

#[derive(Debug, PartialEq)]
pub struct Camera {
    pub film: Film,
    camera_from_raster: Transformation,
    world_from_camera: Transformation,
    camera_type: CameraType,
}

impl Camera {
    fn projective(
        screen_from_camera: Transformation,
        world_from_camera: Transformation,
        film: Film,
        camera_type: CameraType,
    ) -> Camera {
        // Screen goes from [-0.5, 0.5] in the narrow dimension and [-a/2, a/2]
        // in the wider dimension, where a is the aspect ratio
        let (screen_width, screen_height) = if film.width > film.height {
            (film.width as f64 / film.height as f64, 1.0)
        } else {
            (1.0, film.height as f64 / film.width as f64)
        };

        // Raster co-ordinates will be from [0, 0] to [film.width, film.height],
        // where the y axis points downwards. This transform converts them to
        // screen co-ordinates s.t. y points upwards and (0, 0) on screen goes
        // through the center of the film.
        let screen_from_raster = &Transformation::scale(
            screen_width / (film.width as f64),
            -screen_height / (film.height as f64),
            1.0,
        ) * &Transformation::translate(
            -(film.width as f64) * 0.5,
            -(film.height as f64) * 0.5,
            0.0,
        );
        let camera_from_raster = &screen_from_camera.inverse() * &screen_from_raster;

        debug_assert!(camera_from_raster.is_valid());
        debug_assert!(world_from_camera.is_valid());

        Camera {
            film,
            camera_from_raster,
            world_from_camera,
            camera_type,
        }
    }

    pub fn perspective(film: Film, origin: Point, target: Point, up: Vector, fov: f64) -> Camera {
        let screen_from_camera = Transformation::perspective(
            fov,
            // Using the same values as pbrt-v3 here. Not sure why these were
            // picked, but they don't make much of a difference.
            1e-2, 1000.0,
        );
        let world_from_camera = Transformation::look_at(origin, target, up);

        Self::projective(
            screen_from_camera,
            world_from_camera,
            film,
            CameraType::Perspective,
        )
    }

    pub fn orthographic(film: Film, origin: Point, target: Point, up: Vector) -> Camera {
        let screen_from_camera = Transformation::orthographic(
            // Also using the same values as pbrt-v3 here
            0.0, 1.0,
        );
        let world_from_camera = Transformation::look_at(origin, target, up);

        Self::projective(
            screen_from_camera,
            world_from_camera,
            film,
            CameraType::Orthographic,
        )
    }

    pub fn sample<R>(&self, rng: &mut R, raster_x: usize, raster_y: usize) -> Ray
    where
        R: Rng,
    {
        let (dx, dy) = sample_2d(rng);
        let p_raster = Point(raster_x as f64 + dx, raster_y as f64 + dy, 0.0);
        let p_camera = self.camera_from_raster.transform(&p_raster);

        let ray = self.generate_ray(p_camera);
        self.world_from_camera.transform(&ray)
    }

    fn generate_ray(&self, p_camera: Point) -> Ray {
        match self.camera_type {
            CameraType::Perspective => Ray::new(p_camera, (p_camera - Point::O).normalized()),
            CameraType::Orthographic => Ray::new(p_camera, Vector::Z),
        }
    }
}
