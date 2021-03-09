use lyon::{geom::{euclid::Point2D, point}, lyon_tessellation::{BuffersBuilder, FillTessellator, StrokeOptions, StrokeTessellator}, math::{Point, Vector}, path::Path};

use crate::{WithId, drawable::Drawable, mesh::Mesh, project::fit_into_view};

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
    pub fn generate_mesh(&mut self, id: i32) {
        // let mut fill_tess = FillTessellator::new();
        let mut stroke_tess = StrokeTessellator::new();
        let tolerance = 0.02;

        let mut builder = Path::builder();
        builder.begin(point(0.0, 0.0));
        builder.line_to(point(1.0, 0.0));
        builder.close();
        let path = builder.build();

        stroke_tess.tessellate_path(
            &path,
            &StrokeOptions::tolerance(tolerance),
            &mut BuffersBuilder::new(&mut self.mesh.geometry, WithId(id)),
        ).unwrap();

        self.mesh.width = 0.1;

        self.mesh.id = id;

        self.update_mesh();

    }
    pub fn update_mesh(&mut self) {
        let d = self.position_to.sub(&self.position_from);
        let l = d.len();
        let p = Vector::new(d.x, d.y);
        self.mesh.rotation = p.angle_from_x_axis().get();
        self.mesh.position = [self.position_from.x, self.position_from.y];
        self.mesh.scale = l;
        self.mesh.material.color = [1.0, 0.0, 0.0, 0.1];
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
