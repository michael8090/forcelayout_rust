use super::math::*;
pub trait Physics {
    fn getM(&self) -> f64;
    fn getV(&self) -> &Vector2;
    fn setV(&mut self, v: &Vector2) -> ();
    fn getP(&self) -> &Vector2;
    fn setP(&mut self, p: &Vector2) -> ();
    fn getA(&self) -> &Vector2;
    fn setA(&mut self, a: &Vector2) -> ();
}