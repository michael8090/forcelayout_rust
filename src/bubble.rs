use lyon::{geom::{Rect, euclid::{Point2D, Size2D}}, lyon_tessellation::{BuffersBuilder, FillOptions, FillTessellator, StrokeOptions, StrokeTessellator}, math::{Vector, point}, path::{FillRule, Path, Winding, traits::PathBuilder}};

use crate::{WithId, drawable::Drawable, id_generator::IdGenerator, mesh::Mesh, project::{fit_into_view, project_direction_vector}, shape_builder::*};

use super::math::*;
use super::physics::*;
pub struct Bubble {
    pub position: Vector2,
    pub size: f32,
    pub v: Vector2,
    pub a: Vector2,
    pub meshes: Vec<Mesh>,
}

impl Bubble {
    pub fn generate_mesh(&mut self, id: &mut IdGenerator, builder: &mut ShapeBuilder) {
        let view_scale_factor = 0.1;

        let bubble_mesh = builder.build_fill(id.get(), |builder| {
            builder.add_circle(Point2D::new(0.0, 0.0), self.size * 0.9 * view_scale_factor, Winding::Positive);
        });

        let mut bubble_edge_mesh = builder.build_stroke(id.get(), |builder| {
            builder.add_circle(Point2D::new(0.0, 0.0), self.size * 0.95 * view_scale_factor, Winding::Positive);
        });

        bubble_edge_mesh.width = 0.1;

        bubble_edge_mesh.material.color = [0.9, 0.5, 0.5, 1.0];

        let mut bubble_v_mesh = builder.build_stroke(id.get(), |builder| {
            builder.begin(point(0.0, 0.0));
            builder.line_to(point(1.0, 0.0));
            builder.close();
        });
        bubble_v_mesh.material.color = [0.8, 0.8, 0.8, 1.0];
        bubble_v_mesh.width = 0.2;
        
        self.meshes = vec![bubble_mesh, bubble_edge_mesh, bubble_v_mesh];
        self.update_mesh();
    }

    pub fn update_mesh(&mut self) {
        for mesh in self.meshes.iter_mut() {
            mesh.position = [self.position.x, self.position.y];
        }
        let bubble_mesh = &mut self.meshes[0];
        bubble_mesh.material.color = [self.a.len() * 5.0, 0.5, 0.5, 1.0];

        let bubble_v_mesh = &mut self.meshes[2];
        let v = &self.v;
        let p = Vector::new(v.x, v.y);
        bubble_v_mesh.rotation = p.angle_from_x_axis().get();
        let v_len = (v.len() + 1.0).log10() * 30.0;
        bubble_v_mesh.scale = v_len;
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
