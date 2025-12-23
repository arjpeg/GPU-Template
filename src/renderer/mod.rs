pub mod pipelines;
pub mod shaders;

use std::sync::Arc;

use wgpu::*;
use winit::{dpi::PhysicalSize, window::Window};

use crate::renderer::{pipelines::Pipelines, shaders::Shaders};

/// Manages all GPU state and renders all game content.
pub struct Renderer {
    /// A handle to the physical device used to render (usually the GPU).
    pub device: Device,
    /// A queue by which commands are sent to the rendering device.
    pub queue: Queue,

    /// The primary surface (window) being rendered onto.
    pub surface: Surface<'static>,
    /// The configuration of the `surface`.
    pub surface_config: SurfaceConfiguration,

    /// All shaders used in the rendering process.
    shaders: Shaders,
    /// All (compute and render) pipelines and bind group layouts used in the application.
    pipelines: Pipelines,
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

        Ok(Self {
            device,
            queue,
            surface,
            surface_config,
            shaders,
            pipelines,
        })
    }

    /// Renders all world content onto the surface.
    pub fn render(&mut self, pre_present: impl FnOnce()) {
        let output = self.surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor::default());

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

            pass.set_pipeline(&self.pipelines.triangle_pipeline);
            pass.draw(0..3, 0..1);
        }

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
}
