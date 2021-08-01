use lyon::{geom::{euclid::Point2D, point}, lyon_tessellation::{BuffersBuilder, FillTessellator, StrokeOptions, StrokeTessellator}, math::{Point, Vector}, path::{Path, Winding, traits::PathBuilder}};

use crate::{WithId, drawable::Drawable, graph::{self, Graph}, id_generator::ID_GENERATOR, mesh::Mesh, physics::Physics, project::fit_into_view, shape_builder::{ShapeBuilder}};

use super::math::*;

pub struct EdgeElement {
    pub pull_force: f32,
    pub mesh: Mesh,
}

pub struct BubbleElement {
    pub position: Vector2,
    pub size: f32,
    pub v: Vector2,
    pub a: Vector2,
    pub meshes: [Mesh; 3],
    pub label: String,
}


pub type Edge = graph::Edge<BubbleElement, EdgeElement>;
pub type Bubble = graph::Node<BubbleElement, EdgeElement>;

impl Edge {
    pub fn generate_mesh(&mut self, shape_builder: &mut ShapeBuilder) {
        ID_GENERATOR.with(|id| {
            let mut id = id.borrow_mut();
            self.element.mesh = shape_builder.build_stroke(id.get(), |builder| {
                builder.begin(point(0.0, 0.0));
                builder.line_to(point(1.0, 0.0));
                builder.close();
            });
            self.update_mesh();
        });
    }
    pub fn update_mesh(&mut self) {
        let position_from = self.from.borrow().element.position;
        let position_to = self.to.borrow().element.position;
        let d = position_to.sub(&position_from);
        let l = d.len();
        let p = Vector::new(d.x, d.y);
        let mut mesh = self.element.mesh;
        mesh.rotation = p.angle_from_x_axis().get();
        mesh.position = [position_from.x, position_from.y];
        mesh.scale = l;
        mesh.material.color = [0.5, 0.7, 0.7, 0.7];
        mesh.width = 1.0;
    }
}

impl Bubble {
    pub fn generate_mesh(&mut self, builder: &mut ShapeBuilder) {
        ID_GENERATOR.with(|id| {
            let mut id = id.borrow_mut();
            let bubble_mesh = builder.build_fill(id.get(), |builder| {
                builder.add_circle(Point2D::new(0.0, 0.0), 1.0, Winding::Positive);
            });
    
            let bubble_edge_mesh = builder.build_stroke(id.get(), |builder| {
                builder.add_circle(Point2D::new(0.0, 0.0), 1.0, Winding::Positive);
            });
    
            let bubble_v_mesh = builder.build_stroke(id.get(), |builder| {
                builder.begin(point(0.0, 0.0));
                builder.line_to(point(1.0, 0.0));
                builder.close();
            });
    
            // let bubble_label_mesh = builder.buil
            
            self.element.meshes = [bubble_mesh, bubble_edge_mesh, bubble_v_mesh];
            self.update_mesh();
        });
    }

    pub fn update_mesh(&mut self) {
        let element = self.element;
        for mesh in element.meshes.iter_mut() {
            mesh.position = [element.position.x, element.position.y];
        }
        let view_scale_factor = 0.1;
        let bubble_mesh = &mut element.meshes[0];
        bubble_mesh.material.color = [element.a.len() * 5.0, 0.5, 0.5, 1.0];
        bubble_mesh.scale = element.size * 0.9 * view_scale_factor;

        let bubble_edge_mesh = &mut element.meshes[1];
        bubble_edge_mesh.scale = element.size * 0.95 * view_scale_factor;
        
        bubble_edge_mesh.width = 0.1;
        bubble_edge_mesh.material.color = [0.9, 0.5, 0.5, 1.0];

        let bubble_v_mesh = &mut element.meshes[2];
        let v = &element.v;
        let p = Vector::new(v.x, v.y);
        bubble_v_mesh.rotation = p.angle_from_x_axis().get();
        let v_len = (v.len() + 1.0).log10() * 30.0;
        bubble_v_mesh.scale = v_len;

        bubble_v_mesh.material.color = [1.0, 0.8, 0.2, 0.1];
        bubble_v_mesh.width = 0.2;
    }
}

impl Physics for Bubble {
    fn get_m(&self) -> f32 {
        self.element.size
    }

    fn get_v(&self) -> &Vector2 {
        &self.element.v
    }

    fn set_v(&mut self, v: &Vector2) -> () {
        Vector2::assign(&mut self.element.v, v);
    }

    fn get_p(&self) -> &Vector2 {
        &self.element.position
    }

    fn set_p(&mut self, p: &Vector2) -> () {
        Vector2::assign(&mut self.element.position, p);
    }

    fn get_a(&self) -> &Vector2 {
        &self.element.a
    }

    fn set_a(&mut self, a: &Vector2) -> () {
        Vector2::assign(&mut self.element.a, a);
    }
}
