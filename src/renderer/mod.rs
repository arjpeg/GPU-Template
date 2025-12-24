pub mod camera;
pub mod pipelines;
pub mod shaders;

use std::sync::Arc;

use wgpu::*;
use winit::{dpi::PhysicalSize, window::Window};

use crate::renderer::{camera::Camera, pipelines::Pipelines, shaders::Shaders};

/// Manages all GPU state and renders all game content.
#[allow(unused)]
pub struct Renderer {
    /// A handle to the physical device used to render (usually the GPU).
    pub device: Device,
    /// A queue by which commands are sent to the rendering device.
    pub queue: Queue,

    /// The window being rendered onto.
    window: Arc<Window>,
    /// The primary surface texture being rendered onto.
    pub surface: Surface<'static>,
    /// The configuration of the `surface`.
    pub surface_config: SurfaceConfiguration,

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
        let instance = Instance::new(&InstanceDescriptor {
            backends: Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(Arc::clone(&window))?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await?;

        let (device, queue) = adapter.request_device(&DeviceDescriptor::default()).await?;

        let surface_config = Self::get_surface_config(&window);
        surface.configure(&device, &surface_config);

        let shaders = Shaders::new(&device);
        let pipelines = Pipelines::new(&device, &shaders);

        let ui_renderer = egui_wgpu::Renderer::new(
            &device,
            TextureFormat::Bgra8Unorm,
            egui_wgpu::RendererOptions::default(),
        );

        let camera_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Renderer::camera_buffer"),
            size: size_of::<glam::Mat4>() as _,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("Renderer::camera_bind_group"),
            layout: &pipelines.camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        Ok(Self {
            device,
            queue,
            window,
            surface,
            surface_config,
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
        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

        self.queue.write_buffer(
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

        self.queue.submit([encoder.finish()]);

        pre_present();
        output.present();
    }

    /// Resizes the internal rendering surface to match the new target size.
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        let PhysicalSize { width, height } = size;

        self.surface_config.width = width;
        self.surface_config.height = height;

        self.surface.configure(&self.device, &self.surface_config);
    }

    /// Returns an appropriate default [`SurfaceConfiguration`] for rendering to the given window.
    fn get_surface_config(window: &Window) -> SurfaceConfiguration {
        let PhysicalSize { width, height } = window.inner_size();

        let width = width.max(1);
        let height = height.max(1);

        SurfaceConfiguration {
            width,
            height,
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: TextureFormat::Bgra8Unorm,
            present_mode: PresentMode::AutoVsync,
            desired_maximum_frame_latency: 1,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
        }
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
                .update_texture(&self.device, &self.queue, *id, &image_delta);
        }

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: self.window.inner_size().into(),
            pixels_per_point: self.window.scale_factor() as _,
        };

        self.ui_renderer.update_buffers(
            &self.device,
            &self.queue,
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
