use bevy::{
    prelude::*,
    render::{
        render_graph::{Node, NodeRunError, RenderGraphContext, SlotInfo, SlotType},
        render_phase::TrackedRenderPass,
        render_resource::{
            CachedRenderPipelineId, LoadOp, Operations, PipelineCache, RenderPassDescriptor,
        },
        renderer::RenderContext,
        view::ViewTarget,
    },
};

use crate::{pipeline::OverlayPipeline, CameraOverlay, OverlayBuffer};

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

        let buffer = world.resource::<OverlayBuffer>();

        tracked.set_render_pipeline(render_pipeline);
        tracked.set_bind_group(0, &buffer.bind_group, &[]);

        tracked.draw(0..3, 0..1);

        Ok(())
    }
}
