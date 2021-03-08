use lyon::{geom::euclid::Point2D, lyon_tessellation::{BuffersBuilder, FillTessellator, StrokeOptions, StrokeTessellator}, path::Path};

use crate::{GpuVertexBuilder, drawable::Drawable, mesh::Mesh, project::fit_into_view};

use super::math::*;
pub struct Edge {
    pub position_from: Vector2,
    pub position_to: Vector2,
    pub from: usize,
    pub to: usize,
    pub pull_force: f32,
    pub mesh: Mesh,
}

impl  Edge {
    pub fn generate_mesh(&mut self) {
        // let mut fill_tess = FillTessellator::new();
        let mut stroke_tess = StrokeTessellator::new();
        let tolerance = 0.02;

        let mut builder = Path::builder().with_svg();
        // builder.add_rectangle(&Rect {
        //     origin: Point2D::new(0.0, 0.0),
        //     size: Size2D::new(self.size, self.size),
        // }, Winding::Positive);
        builder.move_to(Point2D::new(self.position_from.x, self.position_from.y));
        builder.line_to(Point2D::new(self.position_to.x, self.position_to.y));
        let path = builder.build();

        stroke_tess.tessellate_path(
            &path,
            &StrokeOptions::tolerance(tolerance),
            &mut BuffersBuilder::new(&mut self.mesh.geometry, GpuVertexBuilder()),
        ).unwrap();

        self.mesh.position = [0.0, 0.0];
        self.mesh.material.color = [1.0, 0.0, 0.0, 1.0];

    }
    pub fn update_mesh(&mut self) {
        // self.mesh.position = [self.position.x, self.position.y];
        self.mesh.material.color = [1.0, 0.0, 0.0, 1.0];
    }
}

// impl Drawable for Edge {
//     fn draw(&self, dt: &mut raqote::DrawTarget, source_rect: &Rect, target_rect: &Rect) {
        
//         let p_from = fit_into_view(&self.position_from, source_rect, target_rect);
//         let p_to = fit_into_view(&self.position_to, source_rect, target_rect);

//         let mut pb = PathBuilder::new();
//         pb.move_to(p_from.x as f32, p_from.y as f32);
//         pb.line_to(p_to.x as f32, p_to.y as f32);
//         let path = pb.finish();
//         dt.stroke(&path, &Source::Solid(SolidSource{r: (self.pull_force * 1.0 * 255.0) as u8, g: 100, b: 100, a: 255}), &StrokeStyle::default(),&DrawOptions::new());
//     }
// }
