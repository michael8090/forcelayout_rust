use std::{convert::TryInto, mem::size_of};

use futures::executor::block_on;
use wgpu::{BindGroup, BindGroupLayoutEntry, Buffer, BufferUsage, Instance};

use crate::math::Vector2;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PhysicsEntity {
    pub m: f32,
    pub p: [f32; 2],
    pub v: [f32; 2],
    pub a: [f32; 2],
}
unsafe impl bytemuck::Pod for PhysicsEntity {}
unsafe impl bytemuck::Zeroable for PhysicsEntity {}

pub struct GpuForcelayout {
    instance: Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
    edge_buffer: Buffer,
    bubble_buffer: Buffer,
    globals_buffer: Buffer,
    staging_buffer: Buffer,
    bubble_buffer_size: u64,
    bind_group: BindGroup,
    compute_repulsion_pipeline: wgpu::ComputePipeline,
    compute_pull_pipeline: wgpu::ComputePipeline,
    compute_position_pipeline: wgpu::ComputePipeline,
    bubble_count: u32,
    edge_count: u32,
}

type Globals = [u32; 2];

pub type EdgeEntity = [u32; 2];
// unsafe impl bytemuck::Pod for GpuForcelayout {}
// unsafe impl bytemuck::Zeroable for GpuForcelayout {}

fn create_buffer(device: &wgpu::Device, size: u64, usage: BufferUsage) -> Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        // usage: BufferUsage::COPY_DST | BufferUsage::COPY_SRC | BufferUsage::STORAGE | BufferUsage::MAP_READ,
        usage,
        mapped_at_creation: false,
    })
}

impl GpuForcelayout {
    pub fn new(bubbles: Vec<PhysicsEntity>, edges: Vec<EdgeEntity>) -> Self {
        let globals:Globals = [bubbles.len() as u32, edges.len() as u32];
        
        let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);
        // create an adapter
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
        }))
        .unwrap();
        // create a device and a queue
        let (device, queue) = block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::default(),
                limits: wgpu::Limits::default(),
            },
            None,
        ))
        .unwrap();

        let compute_repulsion_module = &device.create_shader_module(&wgpu::include_spirv!("./../shaders/compute_repulsion.comp.spv"));
        let compute_pull_module = &device.create_shader_module(&wgpu::include_spirv!("./../shaders/compute_pull.comp.spv"));
        let compute_position_module = &device.create_shader_module(&wgpu::include_spirv!("./../shaders/compute_position.comp.spv"));

        let bubble_buffer_size = (size_of::<PhysicsEntity>() * bubbles.len()) as u64;
        let edge_buffer_size = (size_of::<EdgeEntity>() * edges.len()) as u64;
        let globals_buffer_size = size_of::<Globals>() as u64;
        let edge_buffer = create_buffer(&device, edge_buffer_size, BufferUsage::STORAGE | BufferUsage::COPY_DST);
        let bubble_buffer = create_buffer(&device, bubble_buffer_size, BufferUsage::COPY_DST | BufferUsage::COPY_SRC | BufferUsage::STORAGE);
        let globals_buffer = create_buffer(&device, globals_buffer_size, BufferUsage::COPY_SRC| BufferUsage::COPY_DST | BufferUsage::STORAGE);
        let staging_buffer = create_buffer(&device, bubble_buffer_size, BufferUsage::COPY_DST | BufferUsage::MAP_READ);

        let create_bind_group_layout_desc = |binding: u32, size: u64| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStage::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only: false },
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(size),
            },
            count: None,
        };
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                create_bind_group_layout_desc(0, bubble_buffer_size),
                create_bind_group_layout_desc(1, globals_buffer_size),
                create_bind_group_layout_desc(2, edge_buffer_size),
            ],
        });

        let create_bind_group_entry_desc = |binding, buffer| wgpu::BindGroupEntry {
            binding,
            resource: wgpu::BindingResource::Buffer {
                buffer,
                offset: 0,
                size: None,
            },
        };
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                create_bind_group_entry_desc(0, &bubble_buffer),
                create_bind_group_entry_desc(1, &globals_buffer),
                create_bind_group_entry_desc(2, &edge_buffer),
            ],
        });

        queue.write_buffer(&bubble_buffer, 0, bytemuck::cast_slice(&bubbles));
        queue.write_buffer(&globals_buffer, 0, bytemuck::cast_slice(&globals));
        queue.write_buffer(&edge_buffer, 0, bytemuck::cast_slice(&edges));
        queue.submit(None);

        let create_compute_pipeline = |module: &wgpu::ShaderModule| {
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: Some(
                    &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                        label: None,
                        bind_group_layouts: &[&bind_group_layout],
                        push_constant_ranges: &[],
                    }),
                ),
                module,
                entry_point: "main",
            })
        };

        let compute_repulsion_pipeline = create_compute_pipeline(&compute_repulsion_module);
        let compute_pull_pipeline = create_compute_pipeline(&compute_pull_module);
        let compute_position_pipeline = create_compute_pipeline(&compute_position_module);

        Self {
            instance,
            adapter,
            device,
            queue,
            edge_buffer,
            bubble_buffer,
            globals_buffer,
            staging_buffer,
            compute_repulsion_pipeline,
            compute_pull_pipeline,
            compute_position_pipeline,
            bubble_count: bubbles.len() as u32,
            edge_count: edges.len() as u32,
            bind_group,
            bubble_buffer_size,
        }
    }

    async fn get_compute_result(&self) -> Vec<PhysicsEntity> {
        // Note that we're not calling `.await` here.
        let staging_buffer = &self.staging_buffer;
        let buffer_slice = staging_buffer.slice(..);
        // Gets the future representing when `staging_buffer` can be read from
        let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);

        // Poll the device in a blocking manner so that our future resolves.
        // In an actual application, `device.poll(...)` should
        // be called in an event loop or on another thread.
        self.device.poll(wgpu::Maintain::Wait);

        // Awaits until `buffer_future` can be read from
        if let Ok(()) = buffer_future.await {
            // Gets contents of buffer
            let data = buffer_slice.get_mapped_range();
            // Since contents are got in bytes, this converts these bytes back to u32
            let result = data
                .chunks_exact(size_of::<PhysicsEntity>())
                .map(|entity_bytes| *bytemuck::from_bytes::<PhysicsEntity>(&entity_bytes))
                .collect();

            // With the current interface, we have to make sure all mapped views are
            // dropped before we unmap the buffer.
            drop(data);
            staging_buffer.unmap();
            result
        } else {
            panic!("failed to run compute on gpu!")
        }
    }

    pub async fn compute(&mut self) -> Vec<PhysicsEntity> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_bind_group(0, &self.bind_group, &[]);

            pass.set_pipeline(&self.compute_repulsion_pipeline);
            pass.dispatch(self.bubble_count, 1, 1);
            
            pass.set_pipeline(&self.compute_pull_pipeline);
            pass.dispatch(self.edge_count, 1, 1);

            pass.set_pipeline(&self.compute_position_pipeline);
            pass.dispatch(self.bubble_count, 1, 1);
        }
        encoder.copy_buffer_to_buffer(&self.bubble_buffer, 0, &self.staging_buffer, 0, self.bubble_buffer_size);
        self.queue.submit(Some(encoder.finish()));
        self.get_compute_result().await
    }
}
