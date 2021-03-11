use lyon::{geom::{euclid::Point2D, point}, lyon_tessellation::{BuffersBuilder, FillTessellator, StrokeOptions, StrokeTessellator}, math::{Point, Vector}, path::Path};

use crate::{WithId, drawable::Drawable, id_generator::IdGenerator, mesh::Mesh, project::fit_into_view, shape_builder::{ShapeBuilder}};

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
    pub fn generate_mesh(&mut self, id: &mut IdGenerator, shape_builder: &mut ShapeBuilder) {
        self.mesh = shape_builder.build_stroke(id.get(), |builder| {
            builder.begin(point(0.0, 0.0));
            builder.line_to(point(1.0, 0.0));
            builder.close();
        });
        self.update_mesh();
    }
    pub fn update_mesh(&mut self) {
        let d = self.position_to.sub(&self.position_from);
        let l = d.len();
        let p = Vector::new(d.x, d.y);
        self.mesh.rotation = p.angle_from_x_axis().get();
        self.mesh.position = [self.position_from.x, self.position_from.y];
        self.mesh.scale = l;
        self.mesh.material.color = [0.5, 0.7, 0.7, 0.7];
        self.mesh.width = 0.2;
    }
}
