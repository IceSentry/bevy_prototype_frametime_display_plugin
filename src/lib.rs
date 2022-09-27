mod overlay_node;
mod pipeline;

use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    ecs::query::QueryItem,
    prelude::*,
    render::{
        camera::CameraRenderGraph,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_graph::{RenderGraph, SlotInfo, SlotType},
        render_resource::{
            BindGroup, BindGroupDescriptor, BindGroupEntry, ShaderType, StorageBuffer,
            UniformBuffer,
        },
        renderer::{RenderDevice, RenderQueue},
        view::VisibleEntities,
        Extract, RenderApp, RenderStage,
    },
};

use overlay_node::{graph, OverlayNode};
use pipeline::OverlayPipeline;

/// The amount of frametimes kept in the buffer to be rendered in the display
/// Since the bars aren't all of the same size, this is the maximum value possible
// TODO make this configurable
pub const FRAMETIME_BUFFER_LEN: usize = 100;

// TODO use a struct containing each pair of dt and color
// TODO support runtime config
#[derive(Debug, Clone, Resource)]
pub struct OverlayConfig {
    /// The list of delta times where the colors will change in order of smallest to biggest.
    ///
    /// Defaults to 1/240, 1/60, 1/30, 1/15
    pub dts: Vec4,
    /// The amount of frametimes kept in the buffer to be rendered in the overlay
    /// Since the bars aren't all of the same size, this is the maximum value possible
    pub buffer_len: usize,
    /// The colors used in the overlay.
    ///
    /// Defaults to green, yellow, orange, red
    pub colors: Mat4,
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            #[rustfmt::skip]
            dts: Vec4::new(
                1. / 240.,
                1. / 60.,
                1. / 30.,
                1. / 15.
            ),
            buffer_len: 100,
            colors: Mat4::from_cols_array_2d(&[
                Color::GREEN.as_linear_rgba_f32(),
                Color::YELLOW.as_linear_rgba_f32(),
                Color::ORANGE.as_linear_rgba_f32(),
                Color::RED.as_linear_rgba_f32(),
            ]),
        }
    }
}

#[derive(Default)]
pub struct OverlayPlugin;
impl Plugin for OverlayPlugin {
    fn build(&self, app: &mut App) {
        if app
            .world
            .resource::<Diagnostics>()
            .get(FrameTimeDiagnosticsPlugin::FRAME_TIME)
            .is_none()
        {
            app.add_plugin(FrameTimeDiagnosticsPlugin::default());
        }

        app.add_plugin(ExtractComponentPlugin::<CameraOverlay>::default())
            .add_startup_system(move |mut commands: Commands| {
                commands.spawn(CameraOverlayBundle::default());
            });

        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };

        render_app
            .init_resource::<OverlayConfig>()
            .init_resource::<Frametimes>()
            .init_resource::<OverlayPipeline>()
            .init_resource::<OverlayBuffer>()
            .add_system_to_stage(RenderStage::Extract, extract_overlay_camera_phases)
            .add_system_to_stage(RenderStage::Extract, update_frametimes)
            .add_system_to_stage(RenderStage::Prepare, prepare_frametime_overlay_buffer);

        let pass_node_overlay = OverlayNode::new(&mut render_app.world);
        let mut graph = render_app.world.resource_mut::<RenderGraph>();

        let mut overlay_graph = RenderGraph::default();
        overlay_graph.add_node(graph::NODE, pass_node_overlay);
        let input_node_id =
            overlay_graph.set_input(vec![SlotInfo::new(graph::NODE_INPUT, SlotType::Entity)]);
        overlay_graph
            .add_slot_edge(
                input_node_id,
                graph::NODE_INPUT,
                graph::NODE,
                graph::IN_VIEW,
            )
            .unwrap();
        graph.add_sub_graph(graph::NAME, overlay_graph);
    }
}

#[derive(Debug, Clone, ShaderType, Default)]
pub struct OverlayConfigBuffer {
    dt_min: f32,
    dt_max: f32,
    dt_min_log2: f32,
    dt_max_log2: f32,
    max_width: f32,
    len: i32,
    colors: Mat4,
    dts: Vec4,
}

impl OverlayConfigBuffer {
    fn new(dts: Vec4, buffer_len: usize, colors: Mat4) -> Self {
        Self {
            dt_min: dts[0],
            dt_max: dts[3],
            dt_min_log2: dts[0].log2(),
            dt_max_log2: dts[3].log2(),
            max_width: buffer_len as f32,
            len: buffer_len as i32,
            colors,
            dts,
        }
    }
}

#[derive(Resource)]
pub struct OverlayBuffer {
    pub config_buffer: UniformBuffer<OverlayConfigBuffer>,
    pub frametimes_buffer: StorageBuffer<Frametimes>,
    pub bind_group: BindGroup,
}

impl FromWorld for OverlayBuffer {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let render_queue = world.resource::<RenderQueue>();
        let config = world.resource::<OverlayConfig>();
        let frametimes = world.resource::<Frametimes>();
        let pipeline = world.resource::<OverlayPipeline>();

        let mut config_buffer = UniformBuffer::default();
        config_buffer.set(OverlayConfigBuffer::new(
            config.dts,
            config.buffer_len,
            config.colors,
        ));
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

        OverlayBuffer {
            config_buffer,
            frametimes_buffer,
            bind_group,
        }
    }
}

#[derive(Debug, Clone, ShaderType, Resource)]
pub struct Frametimes {
    pub fps: f32,
    pub values: [f32; FRAMETIME_BUFFER_LEN],
}

impl Default for Frametimes {
    fn default() -> Self {
        Self {
            fps: 0.0,
            values: [0.0; FRAMETIME_BUFFER_LEN],
        }
    }
}

impl Frametimes {
    pub fn push(&mut self, value: f32) {
        self.values.rotate_left(1);
        self.values[FRAMETIME_BUFFER_LEN - 1] = value;
    }
}

fn extract_overlay_camera_phases(
    mut commands: Commands,
    cameras_overlay: Extract<Query<(Entity, &Camera), With<CameraOverlay>>>,
) {
    for (entity, camera) in cameras_overlay.iter() {
        if camera.is_active {
            commands.get_or_spawn(entity);
        }
    }
}

fn update_frametimes(diagnostics: Extract<Res<Diagnostics>>, mut frametimes: ResMut<Frametimes>) {
    if let Some(frame_time_diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FRAME_TIME) {
        if let Some(dt) = frame_time_diagnostic.value() {
            frametimes.push(dt as f32 / 1000.0);
        }
    }

    if let Some(fps_diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(fps) = fps_diagnostic.value() {
            frametimes.fps = fps as f32;
        }
    }
}

fn prepare_frametime_overlay_buffer(
    mut buffer: ResMut<OverlayBuffer>,
    frametimes: Res<Frametimes>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    if frametimes.is_changed() {
        buffer.frametimes_buffer.set(frametimes.clone());
        buffer
            .frametimes_buffer
            .write_buffer(&render_device, &render_queue);
    }
}

#[derive(Component, Clone, Copy, Default)]
pub struct CameraOverlay;
impl ExtractComponent for CameraOverlay {
    type Query = &'static Self;
    type Filter = With<Camera>;

    fn extract_component(item: QueryItem<Self::Query>) -> Self {
        *item
    }
}

#[derive(Bundle)]
pub struct CameraOverlayBundle {
    pub camera: Camera,
    pub camera_render_graph: CameraRenderGraph,
    pub projection: OrthographicProjection,
    pub visible_entities: VisibleEntities,
    pub global_transform: GlobalTransform,
    pub camera_overlay: CameraOverlay,
}

impl Default for CameraOverlayBundle {
    fn default() -> Self {
        Self {
            camera: Camera {
                priority: isize::MAX,
                ..default()
            },
            camera_render_graph: CameraRenderGraph::new(graph::NAME),
            visible_entities: default(),
            projection: default(),
            global_transform: default(),
            camera_overlay: default(),
        }
    }
}
