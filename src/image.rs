use std::{fs::File, io::Write, path::Path};

pub fn write_ppm(
    filename: &str,
    pixels: Vec<u8>,
    width: usize,
    height: usize,
) -> Result<(), std::io::Error> {
    let mut file = File::create(Path::new(filename))?;

    write!(file, "P6 {} {} 255\n", width, height)?;

    file.write_all(&pixels)?;

    Ok(())
}
