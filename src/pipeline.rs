use bevy::{
    prelude::*,
    render::{
        render_resource::{
            BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
            BlendState, BufferBindingType, ColorTargetState, ColorWrites, FragmentState,
            MultisampleState, PrimitiveState, RenderPipelineDescriptor, ShaderStages, ShaderType,
            VertexState,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
    },
};

use crate::{Frametimes, OverlayConfigBuffer};

#[derive(Clone, Resource)]
pub struct OverlayPipeline {
    pub shader: Handle<Shader>,
    pub layout: BindGroupLayout,
}

impl FromWorld for OverlayPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let asset_server = world.resource::<AssetServer>();

        let overlay_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(OverlayConfigBuffer::min_size()),
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
                ],
                label: Some("overlay_bind_group_layout"),
            });

        OverlayPipeline {
            layout: overlay_bind_group_layout,
            shader: asset_server.load("shaders/frametime_display.wgsl"),
        }
    }
}

impl OverlayPipeline {
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
