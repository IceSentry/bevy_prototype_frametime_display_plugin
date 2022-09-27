use bevy::{
    prelude::*,
    render::{
        render_graph::{Node, NodeRunError, RenderGraphContext, SlotInfo, SlotType},
        render_phase::TrackedRenderPass,
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
            BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, BlendState,
            BufferBindingType, CachedRenderPipelineId, ColorTargetState, ColorWrites,
            FragmentState, LoadOp, MultisampleState, Operations, PipelineCache, PrimitiveState,
            RenderPassDescriptor, RenderPipelineDescriptor, ShaderStages, ShaderType,
            StorageBuffer, UniformBuffer, VertexState,
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        texture::BevyDefault,
        view::ViewTarget,
    },
};

use crate::{
    plugin::{CameraOverlay, FrametimeOverlayConfig, Frametimes},
    OVERLAY_SHADER_HANDLE,
};

#[derive(Clone, Resource)]
pub struct OverlayPipeline {
    layout: BindGroupLayout,
}

impl FromWorld for OverlayPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.get_resource::<RenderDevice>().unwrap();

        let overlay_bind_group_layout =
            render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Buffer {
                            ty: BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: Some(FrametimeOverlayConfig::min_size()),
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
        }
    }
}

impl OverlayPipeline {
    fn descriptor(&self) -> RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: Some("Overlay Pipeline".into()),
            layout: Some(vec![self.layout.clone()]),
            vertex: VertexState {
                shader: OVERLAY_SHADER_HANDLE.typed(),
                shader_defs: vec![],
                entry_point: "vertex".into(),
                buffers: vec![],
            },
            fragment: Some(FragmentState {
                shader: OVERLAY_SHADER_HANDLE.typed(),
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

#[derive(Resource)]
pub struct FrametimeOverlayBuffer {
    pub config_buffer: UniformBuffer<FrametimeOverlayConfig>,
    pub frametimes_buffer: StorageBuffer<Frametimes>,
    pub bind_group: BindGroup,
}

impl FromWorld for FrametimeOverlayBuffer {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();
        let config = world.resource::<FrametimeOverlayConfig>();
        let frametimes = world.resource::<Frametimes>();
        let pipeline = world.resource::<OverlayPipeline>();

        let mut config_buffer = UniformBuffer::default();
        config_buffer.set(config.clone());
        config_buffer.write_buffer(render_device, render_queue);

        let mut frametimes_buffer = StorageBuffer::default();
        frametimes_buffer.set(frametimes.clone());
        frametimes_buffer.write_buffer(render_device, render_queue);

        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: Some("frametime bind group"),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: config_buffer.binding().unwrap(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: frametimes_buffer.binding().unwrap(),
                },
            ],
            layout: &pipeline.layout,
        });

        FrametimeOverlayBuffer {
            config_buffer,
            frametimes_buffer,
            bind_group,
        }
    }
}

pub(crate) mod graph {
    pub const NAME: &str = "OVERLAY";
    pub const NODE: &str = "OVERLAY_PASS";
    pub const NODE_INPUT: &str = "OVERLAY_PASS_VIEW";
    pub const IN_VIEW: &str = "OVERLAY_IN_VIEW";
}
pub(crate) struct OverlayNode {
    query: QueryState<&'static ViewTarget, With<CameraOverlay>>,
    render_pipeline_id: CachedRenderPipelineId,
}
impl OverlayNode {
    pub(crate) fn new(world: &mut World) -> Self {
        let overlay_pipeline = (*world.resource::<OverlayPipeline>()).clone();
        let render_pipeline = world
            .resource_mut::<PipelineCache>()
            .queue_render_pipeline(overlay_pipeline.descriptor());

        Self {
            query: world.query_filtered(),
            render_pipeline_id: render_pipeline,
        }
    }
}

impl Node for OverlayNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(graph::IN_VIEW, SlotType::Entity)]
    }

    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let view_entity = graph.get_input_entity(graph::IN_VIEW)?;

        let target = if let Ok(result) = self.query.get_manual(world, view_entity) {
            result
        } else {
            return Ok(());
        };

        let target = ViewTarget {
            view: target.view.clone(),
            sampled_target: None,
        };
        let pass_descriptor = RenderPassDescriptor {
            label: Some("overlay"),
            color_attachments: &[Some(target.get_color_attachment(Operations {
                load: LoadOp::Load,
                store: true,
            }))],
            depth_stencil_attachment: None,
        };

        let render_pass = render_context
            .command_encoder
            .begin_render_pass(&pass_descriptor);

        let mut tracked = TrackedRenderPass::new(render_pass);

        let render_pipeline = world
            .resource::<PipelineCache>()
            .get_render_pipeline(self.render_pipeline_id)
            .unwrap();

        let buffer = world.resource::<FrametimeOverlayBuffer>();

        tracked.set_render_pipeline(render_pipeline);
        tracked.set_bind_group(0, &buffer.bind_group, &[]);

        tracked.draw(0..3, 0..1);

        Ok(())
    }
}
