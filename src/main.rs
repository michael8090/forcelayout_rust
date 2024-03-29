mod bubble;
mod create_dataset;
mod drawable;
mod edge;
mod forcelayout;
mod gpu_forcelayout;
mod id_generator;
mod math;
mod mesh;
mod physics;
mod project;
mod shape_builder;

use bubble::Bubble;
use edge::Edge;
use forcelayout::*;

use gpu_forcelayout::{BubbleGpuEntity, EdgeEntity};
use lyon::math::*;
use lyon::path::iterator::PathIterator;
use lyon::path::Path;
use lyon::tessellation;
use lyon::tessellation::geometry_builder::*;
use lyon::tessellation::{FillOptions, FillTessellator};
use lyon::tessellation::{StrokeOptions, StrokeTessellator};

use lyon::algorithms::walk;

use mesh::Mesh;
use physics::Physics;
use shape_builder::ShapeBuilder;
use wgpu::Device;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent, DeviceEvent, MouseButton};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::Window;

// For create_buffer_init()
use wgpu::{util::DeviceExt, BlendFactor, BlendOperation, BlendState, Buffer, Queue, RenderPass};

use futures::executor::block_on;
use std::{
    borrow::Borrow,
    f64::consts,
    num::NonZeroI64,
    ops::{Range, Rem},
    usize,
};

use crate::gpu_forcelayout::GpuForcelayout;
use crate::{math::Vector2, project::fit_into_view};

//use log;

#[repr(C)]
#[derive(Copy, Clone)]
struct Globals {
    resolution: [f32; 2],
    scroll_offset: [f32; 2],
    zoom: f32,
    _pad1: f32,
}

unsafe impl bytemuck::Pod for Globals {}
unsafe impl bytemuck::Zeroable for Globals {}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct GpuVertex {
    position: [f32; 2],
    normal: [f32; 2],
    // prim_id: i32,
}
unsafe impl bytemuck::Pod for GpuVertex {}
unsafe impl bytemuck::Zeroable for GpuVertex {}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct Primitive {
    color: [f32; 4],
    translate: [f32; 2],
    z_index: i32,
    width: f32,
    angle: f32,
    scale: f32,
    _pad1: i32,
    _pad2: i32,
}

impl Primitive {
    const DEFAULT: Self = Primitive {
        color: [0.0; 4],
        translate: [0.0; 2],
        z_index: 0,
        width: 0.0,
        angle: 0.0,
        scale: 1.0,
        _pad1: 0,
        _pad2: 0,
    };
}

unsafe impl bytemuck::Pod for Primitive {}
unsafe impl bytemuck::Zeroable for Primitive {}

#[repr(C)]
#[derive(Copy, Clone)]
struct BgPoint {
    point: [f32; 2],
}
unsafe impl bytemuck::Pod for BgPoint {}
unsafe impl bytemuck::Zeroable for BgPoint {}

const DEFAULT_WINDOW_WIDTH: f32 = 800.0;
const DEFAULT_WINDOW_HEIGHT: f32 = 800.0;

fn get_draw_mesh_range<'a>(
    mesh_range: &'a Vec<(&Mesh, &Range<u32>)>,
    target_range: Range<u32>,
) -> Vec<(&'a Mesh, Range<u32>)> {
    let mut ret = vec![];
    for range in mesh_range.iter() {
        if !(range.1.start > target_range.end || range.1.end < target_range.start) {
            ret.push((
                range.0,
                range.1.start.max(target_range.start) - target_range.start
                    ..range.1.end.min(target_range.end) - target_range.start,
            ));
        }
    }
    ret
}

/// Creates a texture that uses MSAA and fits a given swap chain
fn create_multisampled_framebuffer(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
    sample_count: u32,
) -> wgpu::TextureView {
    let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
        label: Some("Multisampled frame descriptor"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    };

    device
        .create_texture(multisampled_frame_descriptor)
        .create_view(&wgpu::TextureViewDescriptor::default())
}

fn draw_mesh<'a, 'b, 'c, 'd>(
    mesh: &'a Mesh,
    pass: &'c mut RenderPass<'a>,
    instance_range: &Range<u32>,
) {
    pass.set_index_buffer(
        mesh.ibo.as_ref().unwrap().slice(..),
        wgpu::IndexFormat::Uint16,
    );
    pass.set_vertex_buffer(0, mesh.vbo.as_ref().unwrap().slice(..));

    pass.draw_indexed(
        0..(mesh.geometry.indices.len() as u32),
        0,
        instance_range.clone(),
    )
}

fn create_forcelayout_instance(bubbles: &Vec<Bubble>, edges: &Vec<Edge>) -> gpu_forcelayout::GpuForcelayout {
    let mut bubble_physics_entities: Vec<BubbleGpuEntity> = vec![];
    let mut edge_entities: Vec<EdgeEntity> = vec![];
    for bubble in bubbles.iter() {
        bubble_physics_entities.push(BubbleGpuEntity {
            m: bubble.get_m(),
            _pad1: 0.0,
            p: [bubble.position.x, bubble.position.y],
            v: [bubble.v.x, bubble.v.y],
            a: [bubble.a.x, bubble.a.y],
        });
    }
    for edge in edges.iter() {
        edge_entities.push([edge.from as u32, edge.to as u32, 0, 0]);
    }

    let gpu_forcelayout_instance =
        gpu_forcelayout::GpuForcelayout::new(bubble_physics_entities, edge_entities);

    gpu_forcelayout_instance
}

struct SwapChainDescriptor {
    usage: wgpu::TextureUsages,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
    present_mode: wgpu::PresentMode,
}

fn main() {
    env_logger::init();
    println!("== wgpu example ==");
    println!("Controls:");
    println!("  Arrow keys: scrolling");
    println!("  PgUp/PgDown: zoom in/out");
    println!("  b: toggle drawing the background");
    println!("  a/z: increase/decrease the stroke width");

    // Number of samples for anti-aliasing
    // Set to 1 to disable
    let sample_count = 1;

    let mut fill_tess = FillTessellator::new();

    let mut bg_geometry: VertexBuffers<BgPoint, u16> = VertexBuffers::new();

    fill_tess
        .tessellate_rectangle(
            &Rect::new(point(-1.0, -1.0), size(2.0, 2.0)),
            &FillOptions::DEFAULT,
            &mut BuffersBuilder::new(&mut bg_geometry, Custom),
        )
        .unwrap();

    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();
    let size = window.inner_size();

    let mut scene = SceneParams {
        target_zoom: 1.0,
        zoom: 1.0,
        target_scroll: vector(0.0, 0.0),
        scroll: vector(0.0, 0.0),
        show_points: false,
        stroke_width: 1.0,
        target_stroke_width: 1.0,
        draw_background: true,
        cursor_position: (0.0, 0.0),
        window_size: PhysicalSize::new(size.width, size.height as u32),
        size_changed: true,
        need_reset: false,
        need_update_gpu: false,
    };

    // create an instance
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());

    // create an surface
    let surface = unsafe { instance.create_surface(&window) };
    let surface = surface.unwrap();


    // create an adapter
    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: Some(&surface),
        force_fallback_adapter: false,
    }))
    .unwrap();

    // create a device and a queue
    let (device, queue) = block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::default(),
            limits: wgpu::Limits::default(),
            // shader_validation: true
            // shader_validation: true,
        },
        None,
    ))
    .unwrap();

    let config = surface
        .get_default_config(&adapter, size.width, size.height)
        .expect("Surface isn't supported by the adapter.");
    surface.configure(&device, &config);

    // init the game
    // println!("{}", device.limits().max_uniform_buffer_binding_size);
    let mut id = id_generator::IdGenerator::new();
    let mut shape_generator = ShapeBuilder::new();

    // let mut bubbles = create_dataset::create_bubbles(bubble_count);
    // let mut edges = create_dataset::create_edges(bubbles.len(), group_size);

    let (mut bubbles, mut edges) = create_dataset::create_dataset_from_file().unwrap();

    // up to about 20000
    let bubble_count = 5000;
    let group_size = bubble_count as usize / 1;

    for bubble in bubbles.first_mut() {
        bubble.generate_mesh(&mut id, &mut shape_generator);
        for mesh in bubble.meshes.iter_mut() {
            mesh.create_buffer_and_upload(&device);
        }
    }

    for edge in edges.first_mut() {
        edge.generate_mesh(&mut id, &mut shape_generator);
        edge.mesh.create_buffer_and_upload(&device);
    }



    let mut gpu_forcelayout_instance = create_forcelayout_instance(&bubbles, &edges);

    // end init

    let primitive_count = bubble_count as usize * bubbles[0].meshes.len() + edges.len();
    let mut primitives: Vec<Primitive> = Vec::with_capacity(primitive_count);
    for _ in 0..primitive_count {
        primitives.push(Primitive::DEFAULT.clone());
    }

    let bg_vbo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&bg_geometry.vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let bg_ibo = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: bytemuck::cast_slice(&bg_geometry.indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    let primitive_item_size = std::mem::size_of::<Primitive>() as u32;
    let primitive_group_item_count = (device.limits().max_uniform_buffer_binding_size
        / primitive_item_size)
        .min(primitive_count as u32);
    let prim_group_buffer_byte_size = (primitive_group_item_count * primitive_item_size) as u64;
    let prim_group_count =
        (primitive_count as f32 / primitive_group_item_count as f32).ceil() as u32;
    let globals_buffer_byte_size = std::mem::size_of::<Globals>() as u64;

    let prims_ubo = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Prims ubo"),
        size: prim_group_buffer_byte_size,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let globals_ubo = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Globals ubo"),
        size: globals_buffer_byte_size,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let vs_module =
        &device.create_shader_module(wgpu::include_spirv!("./../shaders/geometry.vert.spv"));
    let fs_module =
        &device.create_shader_module(wgpu::include_spirv!("./../shaders/geometry.frag.spv"));
    let bg_vs_module =
        &device.create_shader_module(wgpu::include_spirv!("./../shaders/background.vert.spv"));
    let bg_fs_module =
        &device.create_shader_module(wgpu::include_spirv!("./../shaders/background.frag.spv"));

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Bind group layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    // dynamic: false,
                    min_binding_size: wgpu::BufferSize::new(globals_buffer_byte_size),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(prim_group_buffer_byte_size),
                },
                count: None,
            },
        ],
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Bind group"),
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_ubo.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding{
                    buffer: &prims_ubo,
                    offset: 0,
                    size: None,
                }),
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
        label: None,
    });

    let depth_stencil_state = Some(wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Depth32Float,
        depth_write_enabled: true,
        depth_compare: wgpu::CompareFunction::Greater,
        // clamp_depth: false,
        bias: wgpu::DepthBiasState {
            clamp: 0.0,
            constant: 0,
            slope_scale: 0.0,
        },
        stencil: wgpu::StencilState {
            front: wgpu::StencilFaceState::IGNORE,
            back: wgpu::StencilFaceState::IGNORE,
            read_mask: 0,
            write_mask: 0,
        },
    });

    let mut render_pipeline_descriptor = wgpu::RenderPipelineDescriptor {
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &vs_module,
            entry_point: "main",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<GpuVertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        offset: 0,
                        format: wgpu::VertexFormat::Float32x2,
                        shader_location: 0,
                    },
                    wgpu::VertexAttribute {
                        offset: 8,
                        format: wgpu::VertexFormat::Float32x2,
                        shader_location: 1,
                    },
                    // wgpu::VertexAttribute {
                    //     offset: 16,
                    //     format: wgpu::VertexFormat::Float32,
                    //     shader_location: 2,
                    // },
                ],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &fs_module,
            entry_point: "main",
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                // color_blend: blend_state,
                // alpha_blend: blend_state,
                blend: Some(BlendState {
                    color: wgpu::BlendComponent::REPLACE,
                    alpha: wgpu::BlendComponent::OVER
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: depth_stencil_state.clone(),
        label: None,
        multisample: wgpu::MultisampleState {
            count: sample_count,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    };

    let render_pipeline = device.create_render_pipeline(&render_pipeline_descriptor);

    // TODO: this isn't what we want: we'd need the equivalent of VK_POLYGON_MODE_LINE,
    // but it doesn't seem to be exposed by wgpu?
    render_pipeline_descriptor.primitive.topology = wgpu::PrimitiveTopology::LineList;

    let bg_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &bg_vs_module,
            entry_point: "main",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Point>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[wgpu::VertexAttribute {
                    offset: 0,
                    format: wgpu::VertexFormat::Float32x2,
                    shader_location: 0,
                }],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &bg_fs_module,
            entry_point: "main",
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,

                // blend: Some(wgpu::BlendComponent {
                //     src_factor: BlendFactor::SrcAlpha,
                //     dst_factor: BlendFactor::OneMinusSrcAlpha,
                //     operation: BlendOperation::Add,
                //     // color_blend: wgpu::BlendState::REPLACE,
                //     // alpha_blend: wgpu::BlendState::REPLACE,
                // }),
                blend: Some(BlendState {
                    color: wgpu::BlendComponent::REPLACE,
                    alpha: wgpu::BlendComponent::OVER
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: depth_stencil_state.clone(),
        label: None,
        multisample: wgpu::MultisampleState {
            count: sample_count,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    });

    let size = window.inner_size();

    let mut swap_chain_desc = SwapChainDescriptor {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: wgpu::TextureFormat::Bgra8UnormSrgb,
        width: size.width,
        height: size.height,
        present_mode: wgpu::PresentMode::Mailbox,
    };

    let mut multisampled_render_target = None;

    // let mut swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

    let mut depth_texture_view = None;

    let mut frame_count: f32 = 0.0;

    event_loop.run(move |event, _, control_flow| {
        if update_inputs(event, control_flow, &mut scene, &mut bubbles, &mut id, &mut shape_generator, &device) {
            // keep polling inputs.
            return;
        }

        if scene.need_reset {
            scene.need_reset = false;
            for b in &mut bubbles {
                let random_vec2 = Vector2 {
                    x: rand::random(),
                    y: rand::random(),
                };
                b.position = random_vec2.add_s(-0.5).mul_s(100.0);
            }
        }

        if scene.need_update_gpu {
            scene.need_update_gpu = false;
            gpu_forcelayout_instance = create_forcelayout_instance(&bubbles, &edges);
        }

        // do forcelayout

        // cpu force layout
        // forcelayout(&mut bubbles, &mut edges);
        
        // gpu force layout
        let result = block_on(gpu_forcelayout_instance.compute());

        // read layout data back from gpu
        for (e, b) in result.iter().zip(&mut bubbles) {
            b.a.x = e.a[0];
            b.a.y = e.a[1];

            b.v.x = e.v[0];
            b.v.y = e.v[1];

            b.position.x = e.p[0];
            b.position.y = e.p[1];
        }
        for edge in edges.iter_mut() {
            edge.position_from.set(&(&bubbles[edge.from]).position);
            edge.position_to.set(&(&bubbles[edge.to]).position);
        }

        // update mesh
        for bubble in bubbles.iter_mut() {
            bubble.update_mesh();
        }
        for edge in edges.iter_mut() {
            edge.update_mesh();
        }

        {
            // fit into window
            let first_bubble = &bubbles[0];
            let mut min = first_bubble.position.clone();
            let mut max = min.clone();
            for b in &bubbles {
                let p = &b.position;
                if p.x <= min.x {
                    min.x = p.x;
                }
                if p.y <= min.y {
                    min.y = p.y;
                }
                if p.x >= max.x {
                    max.x = p.x;
                }
                if p.y >= max.y {
                    max.y = p.y;
                }
            }
            let bubble_rect = math::Rect {
                origin: min.clone(),
                width: max.x - min.x,
                height: max.y - min.y,
            };

            let half_width = 0.5 * scene.window_size.width as f32;
            let half_height = 0.5 * scene.window_size.height as f32;

            let padding = 100.0;

            let view_rect = math::Rect {
                origin: Vector2 {
                    x: -half_width + padding,
                    y: -half_height + padding,
                },
                width: scene.window_size.width as f32 - 2.0 * padding,
                height: scene.window_size.height as f32 - 2.0 * padding,
            };

            // let view_rect = math::Rect {
            //     origin: Vector2 {
            //         x: -1.0,
            //         y: -1.0,
            //     },
            //     width: 1.0,
            //     height: scene.window_size.height as f32 * 0.125
            // };

            for b in &mut bubbles {
                let new_pos = fit_into_view(&b.position, &bubble_rect, &view_rect);
                let d = new_pos.sub(&b.position);
                for m in &mut b.meshes {
                    m.position =  [m.position[0] + d.x, m.position[1] + d.y];
                }
            }

            for edge in edges.iter_mut() {
                // it's a hack here, and I know the maintenance sucks
                let bubble_from_mesh_pos = &(&bubbles[edge.from]).meshes[0].position;
                let bubble_to_mesh_pos = &(&bubbles[edge.to]).meshes[0].position;

                edge.position_from = Vector2 {
                    x: bubble_from_mesh_pos[0],
                    y: bubble_from_mesh_pos[1],
                };
                edge.position_to = Vector2 {
                    x: bubble_to_mesh_pos[0],
                    y: bubble_to_mesh_pos[1],
                };
                edge.update_mesh();
            }
        }

        // update gpu primitives
        let l = bubbles.len();
        let ml = bubbles[0].meshes.len();
        let mut bubble_ranges = vec![];
        let edge_range = (l * ml) as u32..primitive_count as u32;

        for i in 0..ml {
            bubble_ranges.push((i * l) as u32..(i * l + l) as u32);
            for j in 0..l {
                primitives[j + i * l] = bubbles[j].meshes[i].get_uniform_buffer();
            }
        }
        for (edge, i) in edges.iter_mut().zip(edge_range.clone()) {
            primitives[i as usize] = edge.mesh.get_uniform_buffer();
        }
        // end do forcelayout

        if scene.size_changed {
            scene.size_changed = false;
            let physical = scene.window_size;
            swap_chain_desc.width = physical.width;
            swap_chain_desc.height = physical.height;
            // swap_chain = device.create_swap_chain(&surface, &swap_chain_desc);

            let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Depth texture"),
                size: wgpu::Extent3d {
                    width: swap_chain_desc.width,
                    height: swap_chain_desc.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });

            depth_texture_view =
                Some(depth_texture.create_view(&wgpu::TextureViewDescriptor::default()));

            multisampled_render_target = if sample_count > 1 {
                Some(create_multisampled_framebuffer(
                    &device,
                    swap_chain_desc.format,
                    swap_chain_desc.width,
                    swap_chain_desc.height,
                    sample_count,
                ))
            } else {
                None
            };
        }

        let frame = match surface.get_current_texture() {
            Ok(frame) => frame,
            Err(e) => {
                println!("Swap-chain error: {:?}", e);
                return;
            }
        };
        let frame_view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
        // A resolve target is only supported if the attachment actually uses anti-aliasing
        // So if sample_count == 1 then we must render directly to the swapchain's buffer
        let color_attachment = if let Some(msaa_target) = &multisampled_render_target {
            wgpu::RenderPassColorAttachment {
                view: msaa_target,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: true,
                },
                resolve_target: Some(&frame_view),
            }
        } else {
            wgpu::RenderPassColorAttachment {
                view: &frame_view,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: true,
                },
                resolve_target: None,
            }
        };

        queue.write_buffer(
            &globals_ubo,
            0,
            bytemuck::cast_slice(&[Globals {
                resolution: [
                    scene.window_size.width as f32,
                    scene.window_size.height as f32,
                ],
                zoom: scene.zoom,
                scroll_offset: scene.scroll.to_array(),
                _pad1: 0.0
            }]),
        );

        // draw the bg
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Encoder"),
        });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(color_attachment)],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_texture_view.as_ref().unwrap(),
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: true,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: true,
                    }),
                }),
            });

            // if scene.draw_background {
            pass.set_pipeline(&bg_pipeline);
            pass.set_bind_group(0, &bind_group, &[]);
            pass.set_index_buffer(bg_ibo.slice(..), wgpu::IndexFormat::Uint16);
            pass.set_vertex_buffer(0, bg_vbo.slice(..));

            pass.draw_indexed(0..6, 0, 0..1);
        }

        queue.submit(Some(encoder.finish()));
        // }

        let mut mesh_range = vec![];
        let bubble_meshes = &bubbles[0].meshes;
        for (mesh, range) in bubble_meshes.iter().zip(&bubble_ranges) {
            mesh_range.push((mesh, range));
        }

        let edge_mesh = &edges[0].mesh;
        mesh_range.push((&edge_mesh, &edge_range));

        for group_index in 0..prim_group_count {
            let i = group_index;
            let n = primitive_group_item_count;
            let mut st = (i * n) as usize;
            let mut ed = ((i + 1) * n) as usize;
            st = st.min(primitives.len());
            ed = ed.min(primitives.len());

            let primitive_slice = &primitives[st..ed];
            queue.write_buffer(&prims_ubo, 0, bytemuck::cast_slice(primitive_slice));

            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Encoder"),
            });

            {
                let color_attachment = if let Some(msaa_target) = &multisampled_render_target {
                    wgpu::RenderPassColorAttachment {
                        view: msaa_target,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                        resolve_target: Some(&frame_view),
                    }
                } else {
                    wgpu::RenderPassColorAttachment {
                        view: &frame_view,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                        resolve_target: None,
                    }
                };
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(color_attachment)],
                    depth_stencil_attachment: Some(
                        wgpu::RenderPassDepthStencilAttachment {
                            view: depth_texture_view.as_ref().unwrap(),
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(0.0),
                                store: true,
                            }),
                            stencil_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(0),
                                store: true,
                            }),
                        },
                    ),
                });

                pass.set_pipeline(&render_pipeline);
                pass.set_bind_group(0, &bind_group, &[]);

                // todo: how to loop over the stereo array?

                let target_mesh_ranges = get_draw_mesh_range(&mesh_range, st as u32..ed as u32);

                for (mesh, range) in target_mesh_ranges {
                    draw_mesh(&mesh, &mut pass, &range);
                }

                // let bubble_meshes = &bubbles[0].meshes;
                // for (mesh, range) in bubble_meshes.iter().zip(&bubble_ranges) {
                //     draw_mesh(& mesh, &mut pass, &range);
                // }

                // let edge_mesh = &edges[0].mesh;
                // draw_mesh(& edge_mesh, &mut pass, &edge_range);
            }

            queue.submit(Some(encoder.finish()));

        }

        frame_count += 1.0;
        frame.present();
    });
}

/// This vertex constructor forwards the positions and normals provided by the
/// tessellators and add a shape id.
pub struct WithId();

impl FillVertexConstructor<GpuVertex> for WithId {
    fn new_vertex(&mut self, vertex: tessellation::FillVertex) -> GpuVertex {
        GpuVertex {
            position: vertex.position().to_array(),
            normal: [0.0, 0.0],
            // prim_id: self.0,
        }
    }
}

impl StrokeVertexConstructor<GpuVertex> for WithId {
    fn new_vertex(&mut self, vertex: tessellation::StrokeVertex) -> GpuVertex {
        GpuVertex {
            position: vertex.position_on_path().to_array(),
            normal: vertex.normal().to_array(),
            // prim_id: self.0,
        }
    }
}

pub struct Custom;

impl FillVertexConstructor<BgPoint> for Custom {
    fn new_vertex(&mut self, vertex: tessellation::FillVertex) -> BgPoint {
        BgPoint {
            point: vertex.position().to_array(),
        }
    }
}

struct SceneParams {
    target_zoom: f32,
    zoom: f32,
    target_scroll: Vector,
    scroll: Vector,
    show_points: bool,
    stroke_width: f32,
    target_stroke_width: f32,
    draw_background: bool,
    cursor_position: (f32, f32),
    window_size: PhysicalSize<u32>,
    size_changed: bool,
    need_reset: bool,
    need_update_gpu: bool,
}

fn update_inputs(
    event: Event<()>,
    control_flow: &mut ControlFlow,
    scene: &mut SceneParams,
    bubbles: &mut Vec<Bubble>,
    id: &mut id_generator::IdGenerator,
    builder: &mut ShapeBuilder,
    device: &Device,
) -> bool {
    let mut mouse_position = Vector2::new();
    match event {
        Event::MainEventsCleared => {
            return false;
        }
        Event::WindowEvent {
            event: WindowEvent::Destroyed,
            ..
        }
        | Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => {
            *control_flow = ControlFlow::Exit;
            return false;
        }
        Event::WindowEvent {
            event: WindowEvent::CursorMoved { position, .. },
            ..
        } => {
            scene.cursor_position = (position.x as f32, position.y as f32);
        }
        Event::WindowEvent {
            event: WindowEvent::Resized(size),
            ..
        } => {
            scene.window_size = size;
            scene.size_changed = true
        }
        Event::WindowEvent {
            event:
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(key),
                            ..
                        },
                    ..
                },
            ..
        } => match key {
            VirtualKeyCode::Escape => {
                *control_flow = ControlFlow::Exit;
                return false;
            }
            VirtualKeyCode::PageDown => {
                scene.target_zoom *= 0.8;
            }
            VirtualKeyCode::PageUp => {
                scene.target_zoom *= 1.25;
            }
            VirtualKeyCode::Left => {
                scene.target_scroll.x -= 50.0 / scene.target_zoom;
            }
            VirtualKeyCode::Right => {
                scene.target_scroll.x += 50.0 / scene.target_zoom;
            }
            VirtualKeyCode::Up => {
                scene.target_scroll.y -= 50.0 / scene.target_zoom;
            }
            VirtualKeyCode::Down => {
                scene.target_scroll.y += 50.0 / scene.target_zoom;
            }
            VirtualKeyCode::P => {
                scene.show_points = !scene.show_points;
            }
            VirtualKeyCode::B => {
                scene.draw_background = !scene.draw_background;
            }
            VirtualKeyCode::A => {
                scene.target_stroke_width /= 0.8;
            }
            VirtualKeyCode::Z => {
                scene.target_stroke_width *= 0.8;
            }
            VirtualKeyCode::Space => {
                scene.need_reset = true;
                scene.need_update_gpu = true;
            }
            _key => {}
        },
        Event::WindowEvent { 
            event: WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            },
            ..
        } => {
            let mut bubble = Bubble {
                position: mouse_position.clone(),
                size: 100.0,
                // size: 100.0,
                v: Vector2{x: 0.0, y: 0.0},
                a: Vector2{x: 0.0, y: 0.0},
                meshes: [Mesh::default(), Mesh::default(), Mesh::default()],
                label: String::from("added"),
            };
            bubble.generate_mesh(id, builder);
            for mesh in bubble.meshes.iter_mut() {
                mesh.create_buffer_and_upload(&device);
            }
            bubbles.push(bubble);
            scene.need_update_gpu = true;
        },
        Event::WindowEvent { 
            event: WindowEvent::CursorMoved {
                position,
                ..
            },
            ..
        } => {
            mouse_position.x = position.x as f32;
            mouse_position.y = position.y as f32;
        },
        _evt => {
            //println!("{:?}", _evt);
        }
    }
    //println!(" -- zoom: {}, scroll: {:?}", scene.target_zoom, scene.target_scroll);

    scene.zoom += (scene.target_zoom - scene.zoom) / 3.0;
    scene.scroll = scene.scroll + (scene.target_scroll - scene.scroll) / 3.0;
    scene.stroke_width =
        scene.stroke_width + (scene.target_stroke_width - scene.stroke_width) / 5.0;

    *control_flow = ControlFlow::Poll;

    return true;
}
