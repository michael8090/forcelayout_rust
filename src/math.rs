#[derive(Debug)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

impl Vector2 {
    pub fn clone(&self) -> Self {
        Vector2{x: self.x, y: self.y,}
    }
    pub fn add(&mut self, a: &Vector2) -> &mut Self {
        self.x += a.x;
        self.y += a.y;
        self
    }
    pub fn sub(&mut self, a: &Vector2) -> &mut Self {
        self.x -= a.x;
        self.y -= a.y;
        self
    }
    pub fn mul(&mut self, a: &Vector2) -> &mut Self {
        self.x *= a.x;
        self.y *= a.y;
        self
    }
    pub fn mul_s(&mut self, a: f64) -> &mut Self {
        self.x *= a;
        self.y *= a;
        self
    }
    pub fn div(&mut self, a: &Vector2) -> &mut Self {
        self.x /= a.x;
        self.y /= a.y;
        self
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
    pub fn norm(&mut self) -> &mut Self {
        let l = self.len();
        self.x /= l;
        self.y /= l;
        self
    }
}
#[derive(Debug)]
pub struct Rect {
    pub origin: Vector2,
    pub width: f64,
    pub height: f64,
}
