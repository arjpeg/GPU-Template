use wgpu::*;

use crate::renderer::shaders::Shaders;

/// Manages the creation and lifecycle of all pipelines and their associated bind group layouts.
pub struct Pipelines {
    /// The pipeline used for rendering a triangle.
    pub triangle_pipeline: RenderPipeline,
}

impl Pipelines {
    /// Creates all the [`Pipelines`] given their associated shaders.
    pub fn new(device: &Device, shaders: &Shaders) -> Self {
        let triangle_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Pipelines::triangle_pipeline_layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let triangle_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Pipelines::triangle_pipeline"),
            layout: Some(&triangle_pipeline_layout),
            vertex: VertexState {
                module: &shaders.triangle_shader,
                entry_point: Some("vs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(FragmentState {
                module: &shaders.triangle_shader,
                entry_point: Some("fs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Bgra8Unorm,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            multisample: MultisampleState::default(),
            depth_stencil: None,
            multiview: None,
            cache: None,
        });

        Self { triangle_pipeline }
    }
}
