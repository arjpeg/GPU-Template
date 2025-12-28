use std::sync::Arc;

use wgpu::*;
use winit::{dpi::PhysicalSize, window::Window};

/// The raw format used for the rendering surface.
pub const SURFACE_FORMAT: TextureFormat = TextureFormat::Bgra8Unorm;
/// The view format used for the rendering surface (target of the render pass).
pub const SURFACE_VIEW_FORMAT: TextureFormat = TextureFormat::Bgra8UnormSrgb;

/// Owns the core GPU objects required to submit work to the graphics device.
pub struct GpuContext {
    /// A handle to the physical device used to render (usually the GPU).
    pub device: Device,
    /// A queue by which commands are sent to the rendering device.
    pub queue: Queue,

    /// The window being rendered onto.
    pub window: Arc<Window>,
    /// The primary surface texture being rendered onto.
    pub surface: Surface<'static>,
    /// The configuration of the `surface`.
    pub surface_config: SurfaceConfiguration,
}

impl GpuContext {
    /// Initializes the handle to the rendering device with wgpu.
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

        let PhysicalSize { width, height } = window.inner_size();

        let surface_config = SurfaceConfiguration {
            width: width.max(1),
            height: height.max(1),
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: SURFACE_FORMAT,
            present_mode: PresentMode::AutoVsync,
            desired_maximum_frame_latency: 1,
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![SURFACE_VIEW_FORMAT],
        };

        surface.configure(&device, &surface_config);

        Ok(Self {
            device,
            queue,
            window,
            surface,
            surface_config,
        })
    }

    /// Resizes the target [`Surface`] to match the new window size.
    pub fn resize(&mut self, size: PhysicalSize<u32>) {
        let PhysicalSize { width, height } = size;

        self.surface_config.width = width;
        self.surface_config.height = height;

        self.surface.configure(&self.device, &self.surface_config);
    }
}
