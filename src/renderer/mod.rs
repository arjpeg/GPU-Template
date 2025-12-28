pub mod camera;
pub mod gpu_context;
pub mod pipelines;
pub mod shaders;

use std::sync::Arc;

use wgpu::*;
use winit::{dpi::PhysicalSize, window::Window};

use crate::renderer::{
    camera::Camera, gpu_context::GpuContext, pipelines::Pipelines, shaders::Shaders,
};

/// Manages all GPU state and renders all game content.
#[allow(unused)]
pub struct Renderer {
    /// The core shared GPU state.
    pub gpu: GpuContext,

    /// All shaders used in the rendering process.
    shaders: Shaders,
    /// All (compute and render) pipelines and bind group layouts used in the application.
    pipelines: Pipelines,

    /// Manages rendering egui content.
    ui_renderer: egui_wgpu::Renderer,

    /// The bind group holding the `camera_buffer`.
    camera_bind_group: BindGroup,
    /// The uniform buffer holding the camera's view-projection matrix.
    camera_buffer: Buffer,
}

impl Renderer {
    /// Initializes the rendering context, creating a new [`Renderer`].
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let gpu = GpuContext::new(window).await?;

        let shaders = Shaders::new(&gpu.device);
        let pipelines = Pipelines::new(&gpu.device, &shaders);

        let ui_renderer = egui_wgpu::Renderer::new(
            &gpu.device,
            TextureFormat::Bgra8Unorm,
            egui_wgpu::RendererOptions::default(),
        );

        let camera_buffer = gpu.device.create_buffer(&BufferDescriptor {
            label: Some("Renderer::camera_buffer"),
            size: size_of::<glam::Mat4>() as _,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group = gpu.device.create_bind_group(&BindGroupDescriptor {
            label: Some("Renderer::camera_bind_group"),
            layout: &pipelines.camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        Ok(Self {
            gpu,
            shaders,
            pipelines,
            ui_renderer,
            camera_bind_group,
            camera_buffer,
        })
    }

    /// Renders all world content onto the surface.
    pub fn render(
        &mut self,
        camera: &Camera,
        ui_context: &egui::Context,
        ui: egui::FullOutput,
        pre_present: impl FnOnce(),
    ) {
        let output = self.gpu.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        self.gpu.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::bytes_of(&camera.view_projection()),
        );

        {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Renderer::main_render_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.01,
                            g: 0.01,
                            b: 0.01,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_bind_group(0, &self.camera_bind_group, &[]);
            pass.set_pipeline(&self.pipelines.triangle_pipeline);

            pass.draw(0..3, 0..1);
        }

        self.render_ui(&view, &mut encoder, ui_context, ui);

        self.gpu.queue.submit([encoder.finish()]);

        pre_present();
        output.present();
    }

    /// Resizes the internal rendering surface to match the new target size.
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        self.gpu.resize(size);
    }

    fn render_ui(
        &mut self,
        view: &TextureView,
        encoder: &mut CommandEncoder,
        context: &egui::Context,
        output: egui::FullOutput,
    ) {
        let tris = context.tessellate(output.shapes, output.pixels_per_point);

        for (id, image_delta) in &output.textures_delta.set {
            self.ui_renderer
                .update_texture(&self.gpu.device, &self.gpu.queue, *id, &image_delta);
        }

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: self.gpu.window.inner_size().into(),
            pixels_per_point: self.gpu.window.scale_factor() as _,
        };

        self.ui_renderer.update_buffers(
            &self.gpu.device,
            &self.gpu.queue,
            encoder,
            &tris,
            &screen_descriptor,
        );

        let mut pass = encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Load,
                        store: StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                label: Some("Renderer::ui_render_pass"),
                timestamp_writes: None,
                occlusion_query_set: None,
            })
            .forget_lifetime();

        self.ui_renderer
            .render(&mut pass, &tris, &screen_descriptor);

        drop(pass);

        for x in &output.textures_delta.free {
            self.ui_renderer.free_texture(x)
        }
    }
}
