use raqote::{DrawOptions, PathBuilder, SolidSource, Source, StrokeStyle};

use crate::{drawable::Drawable, project::fit_into_view};

use super::math::*;
#[derive(Debug)]
pub struct Edge {
    pub position_from: Vector2,
    pub position_to: Vector2,
    pub from: usize,
    pub to: usize,
    pub pull_force: f64,
}

impl Drawable for Edge {
    fn draw(&self, dt: &mut raqote::DrawTarget, source_rect: &Rect, target_rect: &Rect) {
        
        let p_from = fit_into_view(&self.position_from, source_rect, target_rect);
        let p_to = fit_into_view(&self.position_to, source_rect, target_rect);

        let mut pb = PathBuilder::new();
        pb.move_to(p_from.x as f32, p_from.y as f32);
        pb.line_to(p_to.x as f32, p_to.y as f32);
        let path = pb.finish();
        dt.stroke(&path, &Source::Solid(SolidSource{r: (self.pull_force/100000.0 * 255.0) as u8, g: 100, b: 100, a: 255}), &StrokeStyle::default(),&DrawOptions::new());
    }
}
