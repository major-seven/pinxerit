use std::num::NonZeroU64;
use std::time::SystemTime;
use std::{iter, mem};

use bytemuck::cast_slice;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use crate::canvas;
use crate::tessellate::{self, TessellatePipeline, TessellateVertex};
use crate::texture::{self, TexturePipeline};

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let mut builder = winit::window::WindowBuilder::new();
    let window = builder.build(&event_loop).unwrap();

    let mut render_counter = 0;
    {
        let mut state = new(&window).await;
        event_loop
            .run(|event, target| {
                if let Event::WindowEvent {
                    window_id: _,
                    event,
                } = event
                {
                    match event {
                        WindowEvent::Resized(mut new_size) => {
                            // Reconfigure the surface with the new size
                            new_size.width = new_size.width.max(1);
                            new_size.height = new_size.height.max(1);
                            state.config.width = new_size.width;
                            state.config.height = new_size.height;
                            state.surface.configure(&state.device, &state.config);
                            resize(&mut state, new_size);
                            window.request_redraw();
                            // On macos the window needs to be redrawn manually after resizing
                        }
                        WindowEvent::RedrawRequested => {
                            // state.update();
                            match render(&mut state) {
                                Ok(_) => {}
                                // Reconfigure the surface if lost
                                Err(wgpu::SurfaceError::Lost) => {
                                    let size = state.size.clone();
                                    resize(&mut state, size)
                                }
                                // The system is out of memory, we should probably quit
                                Err(wgpu::SurfaceError::OutOfMemory) => target.exit(),
                                // All other errors (Outdated, Timeout) should be resolved by the next frame
                                Err(e) => eprintln!("{:?}", e),
                            }
                        }
                        WindowEvent::CloseRequested => target.exit(),
                        _ => {}
                    };
                }
            })
            .unwrap();
    }
}

// ============================================================================
// State
// ============================================================================

struct State<'w> {
    surface: wgpu::Surface<'w>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    texture_pipeline: TexturePipeline,
    tessellate_pipeline: TessellatePipeline,
}

async fn new<'w>(window: &'w Window) -> State {
    let size = window.inner_size();

    // The instance is a handle to our GPU
    // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    // # Safety
    //
    // The surface needs to live as long as the window that created it.
    // State owns the window so this should be safe.
    let surface = unsafe { instance.create_surface(window) }.unwrap();

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
        .unwrap();
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None, // Trace path
        )
        .await
        .unwrap();

    let surface_caps = surface.get_capabilities(&adapter);
    // Shader code in this tutorial assumes an Srgb surface texture. Using a different
    // one will result all the colors comming out darker. If you want to support non
    // Srgb surfaces, you'll need to account for that when drawing to the frame.
    let surface_format = surface_caps
        .formats
        .iter()
        .copied()
        .find(|f| f.is_srgb())
        .unwrap_or(surface_caps.formats[0]);
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 1,
    };
    surface.configure(&device, &config);

    let texture_pipeline = texture::create_texture_pipeline(&device, &queue, &config);
    let tessellate_pipeline = tessellate::create_tessellate_pipeline(&device, &queue, &config);

    State {
        surface,
        device,
        queue,
        config,
        size,
        texture_pipeline,
        tessellate_pipeline,
    }
}

fn update_vertices(state: &mut State) {
    let mut encoder = state
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Staging Belt Encoder"),
        });
}

fn render(state: &mut State) -> Result<(), wgpu::SurfaceError> {
    let time = SystemTime::now();
    let output = state.surface.get_current_texture()?;
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());

    let mut encoder = state
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

    {
        // let mut canvas = canvas::Canvas::new();
        // let mut line = canvas::Line::start(-1., -1., [0.8, 0.2, 0.5, 1.0]);
        // line.to(
        //     (200. / state.size.width as f32) * 2. - 1.,
        //     (0. / state.size.height as f32) * -2. + 1.,
        // );
        // line.to(
        //     (200. / state.size.width as f32) * 2. - 1.,
        //     (300. / state.size.height as f32) * -2. + 1.,
        // );
        // line.to(
        //     (400. / state.size.width as f32) * 2. - 1.,
        //     (300. / state.size.height as f32) * -2. + 1.,
        // );
        // line.to(
        //     (400. / state.size.width as f32) * 2. - 1.,
        //     (400. / state.size.height as f32) * -2. + 1.,
        // );
        // line.end(&mut canvas);

        // {
        //     let vertex_update_buffer = canvas.tessellates.first().unwrap().vertices.as_slice();
        //     let mut vertex_target_buffer = state.tessellate_pipeline.staging_belt.write_buffer(
        //         &mut encoder,
        //         &state.tessellate_pipeline.vertex_buffer,
        //         0,
        //         NonZeroU64::new(
        //             mem::size_of::<TessellateVertex>() as wgpu::BufferAddress
        //                 * vertex_update_buffer.len() as u64,
        //         )
        //         .unwrap(),
        //         &state.device,
        //     );
        //     vertex_target_buffer.copy_from_slice(cast_slice(&vertex_update_buffer));
        // }
        // {
        //     let index_update_buffer = canvas.tessellates.first().unwrap().indices.as_slice();
        //     let mut index_target_buffer = state.tessellate_pipeline.staging_belt.write_buffer(
        //         &mut encoder,
        //         &state.tessellate_pipeline.index_buffer,
        //         0,
        //         NonZeroU64::new(
        //             mem::size_of::<u16>() as wgpu::BufferAddress * index_update_buffer.len() as u64,
        //         )
        //         .unwrap(),
        //         &state.device,
        //     );
        //     index_target_buffer.copy_from_slice(cast_slice(&index_update_buffer));
        // }
        // state.tessellate_pipeline.staging_belt.finish();
        // index_staging_belt.finish();
        // --------------------------------------------------------

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &state.texture_pipeline.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&state.texture_pipeline.render_pipeline);
        render_pass.set_bind_group(0, &state.texture_pipeline.diffuse_bind_group, &[]);
        render_pass.set_vertex_buffer(0, state.texture_pipeline.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, state.texture_pipeline.instance_buffer.slice(..));
        render_pass.set_index_buffer(
            state.texture_pipeline.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        render_pass.draw_indexed(
            0..state.texture_pipeline.num_indices,
            0,
            0..state.texture_pipeline.instances.len() as u32,
        );

        // TODO: remove -------------------------------------------------------
        render_pass.set_pipeline(&state.tessellate_pipeline.render_pipeline);
        render_pass.set_vertex_buffer(0, state.tessellate_pipeline.vertex_buffer.slice(..));
        render_pass.set_index_buffer(
            state.tessellate_pipeline.index_buffer.slice(..),
            wgpu::IndexFormat::Uint16,
        );
        // render_pass.draw(0..6, 0..1);
        render_pass.draw_indexed(0..12, 0, 0..1);
        // --------------------------------------------------------------------
    }

    state.queue.submit(iter::once(encoder.finish()));
    // index_staging_belt.recall();
    state.tessellate_pipeline.staging_belt.recall();
    output.present();

    println!("{:?}", time.elapsed());
    Ok(())
}

fn resize(state: &mut State, new_size: winit::dpi::PhysicalSize<u32>) {
    if new_size.width > 0 && new_size.height > 0 {
        state.size = new_size;
        state.config.width = new_size.width;
        state.config.height = new_size.height;
        state.surface.configure(&state.device, &state.config);
        // state.camera.aspect = state.config.width as f32 / state.config.height as f32;
        state.texture_pipeline.depth_texture =
            texture::Texture::create_depth_texture(&state.device, &state.config, "depth_texture");
    }
}
