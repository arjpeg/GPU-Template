use wgpu::*;
use winit::dpi::PhysicalSize;

/// The format used for the depth texture.
pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth32Float;

/// All frame buffers used to render a scene.
pub struct FrameTargets {
    /// The depth texture used to render objects in the correct order relative to the camera.
    pub depth: Texture,
}

impl FrameTargets {
    /// Initializes all frame buffers.
    pub fn new(size: PhysicalSize<u32>, device: &Device) -> Self {
        Self {
            depth: Self::create_depth(device, size),
        }
    }

    /// Resizes all frame buffers to match the new window size.
    pub fn resize(&mut self, device: &Device, size: PhysicalSize<u32>) {
        self.depth = Self::create_depth(device, size);
    }

    /// Creates a new depth texture with the given size.
    fn create_depth(device: &Device, size: PhysicalSize<u32>) -> Texture {
        device.create_texture(&TextureDescriptor {
            label: Some("FrameTargets::depth_texture"),
            size: Extent3d {
                width: size.width,
                height: size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
    }
}
