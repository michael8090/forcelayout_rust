use bytemuck::cast_slice;
use lyon::lyon_tessellation::VertexBuffers;
use wgpu::Buffer;

use crate::{GpuVertex, Primitive};


#[derive(Default)]
pub struct Material {
    pub color: [f32; 4],
}



pub struct Mesh {
    pub id: i32,
    pub geometry:  VertexBuffers<GpuVertex, u16>,
    pub material: Material,
    pub position: [f32; 2],
    pub rotation: f32,
    pub scale: f32,
    pub width: f32,

    // for rendering state
    pub ibo: Option<Buffer>,
    pub vbo: Option<Buffer>,
}

impl Mesh {
    pub fn get_uniform_buffer(&self) -> Primitive {
        Primitive {
            color: self.material.color,
            // todo: improve it please...
            translate: [self.position[0], self.position[1]],
            z_index: 10,
            width: self.width,
            angle: self.rotation,
            scale: self.scale,
            ..Primitive::DEFAULT
        }
    } 
}

impl Default for Mesh {
    fn default() -> Self {
        Mesh {
            id: 0,
            geometry: VertexBuffers {
                vertices: vec![],
                indices: vec![],
            },
            material: Material::default(),
            position: [0.0, 0.0],
            rotation: 0.0,
            scale: 1.0,
            ibo: None,
            vbo: None,
            width: 0.0,
        }
        
    }
}