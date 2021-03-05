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
    fn getM(&self) -> f64 {
        self.size
    }

    fn getV(&self) -> &Vector2 {
        &self.v
    }

    fn setV(&mut self, v: &Vector2) -> () {
        Vector2::assign(&mut self.v, v);
    }

    fn getP(&self) -> &Vector2 {
        &self.position
    }

    fn setP(&mut self, p: &Vector2) -> () {
        Vector2::assign(&mut self.position, p);
    }

    fn getA(&self) -> &Vector2 {
        &self.a
    }

    fn setA(&mut self, a: &Vector2) -> () {
        Vector2::assign(&mut self.a, a);
    }
}

impl Drawable for Bubble {
    fn draw(&self, dt: &mut raqote::DrawTarget, source_rect: &Rect, target_rect: &Rect) {
        let mut pb = PathBuilder::new();
        // println!("{}， {}", self.position.x, self.position.y);
        let p = fit_into_view(&self.position, source_rect, target_rect);
        println!("{}， {}", p.x, p.y);

        pb.arc(p.x as f32, p.y as f32, self.size as f32, 0.0, 360.0);
        let path = pb.finish();
        dt.fill(&path, &Source::Solid(SolidSource{r: (self.a.sqrt_len()/1000.0 * 255.0) as u8, g: 100, b: 100, a: 255}), &DrawOptions::new())
    }
}
