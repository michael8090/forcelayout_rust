use raqote::*;

use crate::math::Rect;
pub trait Drawable {
    fn draw(&self, dt: &mut DrawTarget, source_rect: &Rect, target_rect: &Rect);
}