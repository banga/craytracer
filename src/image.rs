use std::{fs::File, io::Write, path::Path};

pub struct Image {
    pub width: usize,
    pub height: usize,
    pixels: Vec<u8>,
}

impl Image {
    pub fn new(width: usize, height: usize) -> Image {
        let mut pixels = Vec::new();
        pixels.resize(width * height * 3, 0);
        Image {
            width,
            height,
            pixels,
        }
    }

    pub fn set_pixel(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        let offset = (x + y * self.width) * 3;
        self.pixels[offset] = r;
        self.pixels[offset + 1] = g;
        self.pixels[offset + 2] = b;
    }

    pub fn write(&self, filename: &str) -> Result<(), std::io::Error> {
        let mut file = File::create(Path::new(filename))?;

        write!(file, "P6 {} {} 255\n", self.width, self.height)?;

        file.write_all(&self.pixels)?;

        Ok(())
    }
}
