use std::convert::TryInto;

use crate::image::Image;
use crate::vector::Vector;

mod image;
mod vector;

fn main() {
    let v1 = Vector::new(1.0, 2.0, 3.0);
    let v2 = Vector::new(3.0, 2.0, 1.0);
    let v3 = Vector::new(3.0, 4.0, 5.0);
    println!("Hi");
    println!("{} {} {}", v1.magnitude(), v2.magnitude(), v1.dot(&v2));
    println!("{} {}", v3, v3.normalized());

    let mut image = Image::new(1024, 1024);

    for x in 0..image.width {
        for y in 0..image.height {
            image.set_pixel(x, y, (x / 4).try_into().unwrap(), (y / 4).try_into().unwrap(), 200);
        }
    }

    image.write("out.ppm").expect("Error writing image")
}
