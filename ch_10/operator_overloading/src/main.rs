use std::ops::{Add, Div, Mul, Neg, Sub};

struct Vec2 {
    x: f64,
    y: f64,
}

impl Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Neg for Vec2 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

//f64 to denote the type of the second element in the mult
//fn signature (docs docs)
impl Mul<f64> for Vec2 {
    type Output = Self;

    fn mul(self, val: f64) -> Self {
        Self {
            x: self.x * val,
            y: self.y * val,
        }
    }
}

impl PartialEq for Vec2 {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl std::fmt::Display for Vec2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl Div<f64> for Vec2 {
    type Output = Self;

    fn div(self, d: f64) -> Self {
        Self {
            x: self.x / d,
            y: self.y / d,
        }
    }
}

impl From<(f64, f64)> for Vec2 {
    fn from(vals: (f64, f64)) -> Self {
        Self {
            x: vals.0,
            y: vals.1,
        }
    }
}

impl Vec2 {
    fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y
    }
}

fn normalize(v: Vec2) -> Vec2 {
    let mut d = (v.x * v.x) + (v.y * v.y);
    d = d.sqrt();

    v.div(d)
}

fn main() {
    let v = (Vec2::from((3.0, 4.0)) * 2.0 + Vec2::from((1.0, 0.0))).dot(&Vec2::from((1.0, 0.0)));
    println!("{}", v);
}
