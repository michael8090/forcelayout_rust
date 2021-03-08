use lyon::{geom::{Rect, euclid::{Point2D, Size2D}}, lyon_tessellation::{BuffersBuilder, FillOptions, FillTessellator, StrokeTessellator}, path::{FillRule, Path, Winding, traits::PathBuilder}};

use crate::{GpuVertexBuilder, drawable::Drawable, mesh::Mesh, project::{fit_into_view, project_direction_vector}};

use super::math::*;
use super::physics::*;
pub struct Bubble {
    pub position: Vector2,
    pub size: f32,
    pub v: Vector2,
    pub a: Vector2,
    pub mesh: Mesh,
}

impl Bubble {
    pub fn generate_mesh(&mut self) {
        let mut fill_tess = FillTessellator::new();
        let mut stroke_tess = StrokeTessellator::new();
        let tolerance = 0.02;

        let mut builder = Path::builder();
        // builder.add_rectangle(&Rect {
        //     origin: Point2D::new(0.0, 0.0),
        //     size: Size2D::new(self.size, self.size),
        // }, Winding::Positive);
        builder.add_circle(Point2D::new(0.0, 0.0), self.size, Winding::Positive);
        let path = builder.build();

        fill_tess.tessellate_path(
            &path,
            &FillOptions::tolerance(tolerance).with_fill_rule(FillRule::NonZero),
            &mut BuffersBuilder::new(&mut self.mesh.geometry, GpuVertexBuilder()),
        ).unwrap();

        self.mesh.position = [self.position.x, self.position.y];
        self.mesh.material.color = [0.0, 0.0, 1.0, 1.0];

    }

    pub fn update_mesh(&mut self) {
        self.mesh.position = [self.position.x, self.position.y];
        self.mesh.material.color = [0.0, 0.0, 1.0, 1.0];
    }
}

impl Physics for Bubble {
    fn get_m(&self) -> f32 {
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

// impl Drawable for Bubble {
//     fn draw(&self, dt: &mut raqote::DrawTarget, source_rect: &Rect, target_rect: &Rect) {
//         let p = fit_into_view(&self.position, source_rect, target_rect);
//         let mut pb = PathBuilder::new();
//         pb.arc(p.x as f32, p.y as f32, self.size as f32, 0.0, 360.0);
//         let path = pb.finish();
//         dt.fill(&path, &Source::Solid(SolidSource{r: (self.a.len() * 5.0 * 255.0) as u8, g: 100, b: 100, a: 255}), &DrawOptions::new());
//         let mut pb = PathBuilder::new();
//         pb.arc(p.x as f32, p.y as f32, (self.size * 1.05) as f32, 0.0, 360.0);
//         let path = pb.finish();
//         let mut stroke_style = StrokeStyle::default();
//         stroke_style.width = ((self.size * 0.1) as f32).max(1.0);
//         dt.stroke(&path,&Source::Solid(SolidSource{r: 200, g: 200, b: 200, a: 255}), &stroke_style,&DrawOptions::new());

//         let mut v = project_direction_vector(&self.v, source_rect.width, source_rect.height, target_rect.width, target_rect.height);
//         let mut pb = PathBuilder::new();
//         let v_len = (v.len() + 1.0).log10() * 30.0;
//         v = v.norm().mul_s(v_len);
//         let end_point = v.add(&p);
//         pb.move_to(p.x as f32, p.y as f32);
//         pb.line_to(end_point.x as f32, end_point.y as f32);
//         let path = pb.finish();
//         dt.stroke(&path,&Source::Solid(SolidSource{r: 100, g: 255, b: 100, a: 255}), &StrokeStyle::default(),&DrawOptions::new());
//     }
// }
