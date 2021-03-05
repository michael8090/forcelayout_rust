#[derive(Debug)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

impl Vector2 {
    pub fn new() -> Self {
        Vector2{x: 0.0, y: 0.0}
    }
    pub fn clone(&self) -> Self {
        Vector2{x: self.x, y: self.y,}
    }
    fn vector_two_operand(&self, a: &Self, f: fn (v0: f64, v1: f64) -> f64) -> Self {
        let mut out = Self::new();
        out.x = f(self.x, a.x);
        out.y = f(self.y, a.y);
        out
    }
    fn scala_two_operand(&self, a: f64, f: fn (v0: f64, v1: f64) -> f64) -> Self {
        let mut out = Self::new();
        out.x = f(self.x, a);
        out.y = f(self.y, a);
        out
    }
    pub fn add(&self, a: &Vector2) -> Self {
        self.vector_two_operand(a, |v0, v1| {v0 + v1})
    }
    pub fn sub(&self, a: &Vector2) -> Self {
        self.vector_two_operand(a, |v0, v1| {v0 - v1})
    }
    pub fn mul(&self, a: &Vector2) -> Self {
        self.vector_two_operand(a, |v0, v1| {v0 * v1})
    }
    pub fn mul_s(&self, a: f64) -> Self {
        self.scala_two_operand(a, |v0, v1| {v0 * v1})
    }
    pub fn div(&mut self, a: &Vector2) -> Self {
        self.vector_two_operand(a, |v0, v1| {v0 / v1})
    }
    pub fn assign(target: &mut Self, source: & Self) {
        target.x = source.x;
        target.y = source.y;
    }
    pub fn set(&mut self, source: &Self) -> &mut Self {
        self.x = source.x;
        self.y = source.y;
        self
    }
    pub fn sqrt_len(&self) -> f64 {
        self.x * self.x + self.y * self.y
    } 
    pub fn len(&self) -> f64{
        f64::sqrt(self.sqrt_len())
    }
    pub fn norm(&self) -> Self {
        let mut out = Self::new();

        let l = self.len();
        out.x = self.x / l;
        out.y = self.y / l;
        out
    }
}
#[derive(Debug)]
pub struct Rect {
    pub origin: Vector2,
    pub width: f64,
    pub height: f64,
}
