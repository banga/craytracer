use craytracer::color::Color;

#[test]
fn from_rgb() {
    assert_eq!(
        Color::from_rgb(255, 128, 0),
        Color {
            r: 1.0,
            g: 128.0 / 255.0,
            b: 0.0
        }
    );
}

#[test]
fn to_rgb() {
    assert_eq!(Color::from_rgb(255, 128, 0).to_rgb(), (255, 128, 0));
}

#[test]
fn add() {
    let a = Color {
        r: 1.0,
        g: 2.0,
        b: 3.0,
    };
    let b = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
    };
    assert_eq!(
        a + b,
        Color {
            r: 2.0,
            g: 3.0,
            b: 4.0
        }
    );
}

#[test]
fn add_assign() {
    let mut a = Color {
        r: 1.0,
        g: 2.0,
        b: 3.0,
    };
    a += Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
    };
    assert_eq!(
        a,
        Color {
            r: 2.0,
            g: 3.0,
            b: 4.0
        }
    );
}

#[test]
fn mul() {
    let a = Color {
        r: 1.0,
        g: 2.0,
        b: 3.0,
    };
    assert_eq!(
        a * 2.0,
        Color {
            r: 2.0,
            g: 4.0,
            b: 6.0
        }
    );
}

#[test]
fn mul_assign() {
    let mut a = Color {
        r: 1.0,
        g: 2.0,
        b: 3.0,
    };
    a *= 2.0;
    assert_eq!(
        a,
        Color {
            r: 2.0,
            g: 4.0,
            b: 6.0
        }
    );
}

#[test]
fn div() {
    let a = Color {
        r: 1.0,
        g: 2.0,
        b: 3.0,
    };
    assert_eq!(
        a / 2.0,
        Color {
            r: 0.5,
            g: 1.0,
            b: 1.5
        }
    );
}

#[test]
fn div_assign() {
    let mut a = Color {
        r: 1.0,
        g: 2.0,
        b: 3.0,
    };
    a *= 0.5;
    assert_eq!(
        a,
        Color {
            r: 0.5,
            g: 1.0,
            b: 1.5
        }
    );
}
