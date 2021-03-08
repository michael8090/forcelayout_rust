use bytemuck::cast_slice;
use lyon::lyon_tessellation::VertexBuffers;
use wgpu::Buffer;

use crate::{GpuVertex, Primitive};


#[derive(Default)]
pub struct Material {
    pub color: [f32; 4],
}



pub struct Mesh {
    pub geometry:  VertexBuffers<GpuVertex, u16>,
    pub material: Material,
    pub position: [f32; 2],
    pub rotation: f32,
    pub scale: [f32; 2],

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
            ..Primitive::DEFAULT
        }
    } 
}

impl Default for Mesh {
    fn default() -> Self {
        Mesh {
            geometry: VertexBuffers {
                vertices: vec![],
                indices: vec![],
            },
            material: Material::default(),
            position: [0.0, 0.0],
            rotation: 0.0,
            scale: [0.0, 0.0],
            ibo: None,
            vbo: None,
        }
        
    }
}