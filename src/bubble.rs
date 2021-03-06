use raqote::{DrawOptions, PathBuilder, SolidSource, Source};

use crate::{drawable::Drawable, project::fit_into_view};

use super::math::*;
use super::physics::*;
#[derive(Debug)]
pub struct Bubble {
    pub position: Vector2,
    pub size: f64,
    pub v: Vector2,
    pub a: Vector2,
}

impl Physics for Bubble {
    fn get_m(&self) -> f64 {
        self.size
    }

    fn get_v(&self) -> &Vector2 {
        &self.v
    }

    fn set_v(&mut self, v: &Vector2) -> () {
        Vector2::assign(&mut self.v, v);
    }

    fn get_p(&self) -> &Vector2 {
        &self.position
    }

    fn set_p(&mut self, p: &Vector2) -> () {
        Vector2::assign(&mut self.position, p);
    }

    fn get_a(&self) -> &Vector2 {
        &self.a
    }

    fn set_a(&mut self, a: &Vector2) -> () {
        Vector2::assign(&mut self.a, a);
    }
}

impl Drawable for Bubble {
    fn draw(&self, dt: &mut raqote::DrawTarget, source_rect: &Rect, target_rect: &Rect) {
        let mut pb = PathBuilder::new();
        let p = fit_into_view(&self.position, source_rect, target_rect);

        pb.arc(p.x as f32, p.y as f32, self.size as f32, 0.0, 360.0);
        let path = pb.finish();
        dt.fill(&path, &Source::Solid(SolidSource{r: (self.a.sqrt_len()/1000.0 * 255.0) as u8, g: 100, b: 100, a: 255}), &DrawOptions::new())
    }
}
