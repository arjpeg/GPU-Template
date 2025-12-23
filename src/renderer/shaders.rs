use wgpu::{Device, ShaderModule, include_wgsl};

/// All compiled and hot reloadable shaders used in the application.
pub struct Shaders {
    /// The shader used for drawing a triangle.
    pub triangle_shader: ShaderModule,
}

impl Shaders {
    /// Creates and compiles all shaders.
    pub fn new(device: &Device) -> Self {
        let triangle_shader =
            device.create_shader_module(include_wgsl!("../../assets/triangle_shader.wgsl"));

        Self { triangle_shader }
    }
}
