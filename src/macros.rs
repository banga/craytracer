pub use crate::geometry::vector::Vector;

#[macro_export]
macro_rules! v {
    ($x:expr, $y:expr, $z:expr) => {
        $crate::macros::Vector($x as f64, $y as f64, $z as f64)
    };
}

pub use crate::geometry::point::Point;

#[macro_export]
macro_rules! p {
    ($x:expr, $y:expr, $z:expr) => {
        $crate::macros::Point($x as f64, $y as f64, $z as f64)
    };
}

pub use crate::geometry::normal::Normal;

#[macro_export]
macro_rules! n {
    ($x:expr, $y:expr, $z:expr) => {
        $crate::macros::Normal($x as f64, $y as f64, $z as f64)
    };
}
