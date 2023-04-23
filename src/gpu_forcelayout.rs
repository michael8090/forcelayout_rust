use std::{convert::TryInto, mem::size_of};

use futures::executor::block_on;
use wgpu::{BindGroup, BindGroupLayoutEntry, Buffer, BufferUsages, Instance};

use crate::math::Vector2;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BubbleGpuEntity {
    pub m: f32,
    // if i don't pad a float here, the os will fill a random number here anyway, and it ruins the buffer size calculation
    // see https://renderdoc.org/vkspec_chunked/chap16.html#interfaces-resources-layout
    pub _pad1: f32, 
    pub p: [f32; 2],
    pub v: [f32; 2],
    pub a: [f32; 2],
}
unsafe impl bytemuck::Pod for BubbleGpuEntity {}
unsafe impl bytemuck::Zeroable for BubbleGpuEntity {}

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

type Globals = [u32; 4];

// and every element in an array should be 16 bytes(4 u32)
// https://www.cnblogs.com/murongxiaopifu/p/9697704.html
pub type EdgeEntity = [u32; 4];
// unsafe impl bytemuck::Pod for GpuForcelayout {}
// unsafe impl bytemuck::Zeroable for GpuForcelayout {}

fn create_buffer(device: &wgpu::Device, size: u64, usage: BufferUsages) -> Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        // usage: BufferUsages::COPY_DST | BufferUsages::COPY_SRC | BufferUsages::STORAGE | BufferUsages::MAP_READ,
        usage,
        mapped_at_creation: false,
    })
}

impl GpuForcelayout {
    pub fn new(bubbles: Vec<BubbleGpuEntity>, edges: Vec<EdgeEntity>) -> Self {
        let globals:Globals = [bubbles.len() as u32, edges.len() as u32, 0, 0];
        
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor{
            backends: wgpu::Backends::PRIMARY,
            dx12_shader_compiler: wgpu::Dx12Compiler::default(),
        });
        // create an adapter
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: false,
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

        let compute_repulsion_module = &device.create_shader_module(wgpu::include_spirv!("./../shaders/compute_repulsion.comp.spv"));
        let compute_pull_module = &device.create_shader_module(wgpu::include_spirv!("./../shaders/compute_pull.comp.spv"));
        let compute_position_module = &device.create_shader_module(wgpu::include_spirv!("./../shaders/compute_position.comp.spv"));

        let bubble_buffer_size = (size_of::<BubbleGpuEntity>() * bubbles.len()) as u64;
        let edge_buffer_size = (size_of::<EdgeEntity>() * edges.len()) as u64;
        let globals_buffer_size = size_of::<Globals>() as u64;
        let edge_buffer = create_buffer(&device, edge_buffer_size, BufferUsages::STORAGE | BufferUsages::COPY_DST);
        let bubble_buffer = create_buffer(&device, bubble_buffer_size, BufferUsages::COPY_DST | BufferUsages::COPY_SRC | BufferUsages::STORAGE);
        let globals_buffer = create_buffer(&device, globals_buffer_size, BufferUsages::COPY_SRC| BufferUsages::COPY_DST | BufferUsages::STORAGE);
        let staging_buffer = create_buffer(&device, bubble_buffer_size, BufferUsages::COPY_DST | BufferUsages::MAP_READ);

        let create_bind_group_layout_desc = |binding: u32, size: u64| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
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

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry{ binding: 0, resource: bubble_buffer.as_entire_binding() },
                wgpu::BindGroupEntry{ binding: 1, resource: globals_buffer.as_entire_binding() },
                wgpu::BindGroupEntry{ binding: 2, resource: edge_buffer.as_entire_binding() },
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

    async fn get_compute_result(&self) -> Vec<BubbleGpuEntity> {
        // Note that we're not calling `.await` here.
        let staging_buffer = &self.staging_buffer;
        let buffer_slice = staging_buffer.slice(..);
        // Gets the future representing when `staging_buffer` can be read from
        let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read, Result::unwrap);

        // Poll the device in a blocking manner so that our future resolves.
        // In an actual application, `device.poll(...)` should
        // be called in an event loop or on another thread.
        self.device.poll(wgpu::Maintain::Wait);

        // Awaits until `buffer_future` can be read from
        // if let Ok(()) = buffer_future.await {
            // Gets contents of buffer
            let data = buffer_slice.get_mapped_range();
            // Since contents are got in bytes, this converts these bytes back to u32
            let result = data
                .chunks_exact(size_of::<BubbleGpuEntity>())
                .map(|entity_bytes| *bytemuck::from_bytes::<BubbleGpuEntity>(&entity_bytes))
                .collect();

            // With the current interface, we have to make sure all mapped views are
            // dropped before we unmap the buffer.
            drop(data);
            staging_buffer.unmap();
            result
        // } else {
        //     panic!("failed to run compute on gpu!")
        // }
    }

    pub async fn compute(&mut self) -> Vec<BubbleGpuEntity> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            pass.set_bind_group(0, &self.bind_group, &[]);

            pass.set_pipeline(&self.compute_repulsion_pipeline);
            pass.dispatch_workgroups(self.bubble_count, 1, 1);
            
            pass.set_pipeline(&self.compute_pull_pipeline);
            pass.dispatch_workgroups(self.bubble_count, 1, 1);

            pass.set_pipeline(&self.compute_position_pipeline);
            pass.dispatch_workgroups(self.bubble_count, 1, 1);
        }
        encoder.copy_buffer_to_buffer(&self.bubble_buffer, 0, &self.staging_buffer, 0, self.bubble_buffer_size);
        self.queue.submit(Some(encoder.finish()));
        self.get_compute_result().await
    }
}
