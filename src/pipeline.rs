use bevy::{
    prelude::*,
    render::{
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState,
            BufferBindingType, ColorTargetState, ColorWrites, FragmentState, MultisampleState,
            PrimitiveState, RenderPipelineDescriptor, SamplerBindingType, ShaderStages, ShaderType,
            TextureSampleType, TextureViewDimension, VertexState,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
    },
};

use crate::{Frametimes, OverlayBindGroups, OverlayConfigUniform};

#[derive(Clone, Resource)]
pub struct OverlayPipeline {
    pub shader: Handle<Shader>,
    pub layout: BindGroupLayout,
    pub bind_group: BindGroup,
}

impl FromWorld for OverlayPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let asset_server = world.resource::<AssetServer>();
        let buffer = world.resource::<OverlayBindGroups>();

        let layout = OverlayPipeline::layout(render_device);
        let bind_group = OverlayPipeline::create_bind_group(render_device, &layout, buffer);

        OverlayPipeline {
            layout,
            shader: asset_server.load("shaders/frametime_display.wgsl"),
            bind_group,
        }
    }
}

impl OverlayPipeline {
    pub fn update_bind_group(
        &mut self,
        render_device: &RenderDevice,
        bind_group_buffers: &OverlayBindGroups,
    ) {
        self.bind_group =
            OverlayPipeline::create_bind_group(render_device, &self.layout, bind_group_buffers);
    }

    pub fn create_bind_group(
        render_device: &RenderDevice,
        layout: &BindGroupLayout,
        buffer: &OverlayBindGroups,
    ) -> BindGroup {
        render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("frametime bind group"),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: buffer.config_buffer.binding().unwrap(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: buffer.frametimes_buffer.binding().unwrap(),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: buffer.font_image_texture.get_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: buffer.font_image_sampler.get_binding(),
                },
            ],
            layout,
        })
    }

    fn layout(render_device: &RenderDevice) -> BindGroupLayout {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: Some(OverlayConfigUniform::min_size()),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: Some(Frametimes::min_size()),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    count: None,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    visibility: ShaderStages::FRAGMENT,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    count: None,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    visibility: ShaderStages::FRAGMENT,
                },
            ],
            label: Some("overlay_bind_group_layout"),
        })
    }

    pub fn descriptor(&self) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("Overlay Pipeline".into()),
            layout: Some(vec![self.layout.clone()]),
            vertex: VertexState {
                shader: self.shader.clone(),
                shader_defs: vec![],
                entry_point: "vertex".into(),
                buffers: vec![],
            },
            fragment: Some(FragmentState {
                shader: self.shader.clone(),
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: BevyDefault::bevy_default(),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
        }
    }
}
