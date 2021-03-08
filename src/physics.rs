use super::math::*;
pub trait Physics {
    fn get_m(&self) -> f32;
    fn get_v(&self) -> &Vector2;
    fn set_v(&mut self, v: &Vector2) -> ();
    fn get_p(&self) -> &Vector2;
    fn set_p(&mut self, p: &Vector2) -> ();
    fn get_a(&self) -> &Vector2;
    fn set_a(&mut self, a: &Vector2) -> ();
}