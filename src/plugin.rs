use bevy::{
    asset::load_internal_asset,
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    ecs::query::QueryItem,
    prelude::*,
    render::{
        camera::CameraRenderGraph,
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        render_graph::{RenderGraph, SlotInfo, SlotType},
        render_resource::ShaderType,
        renderer::{RenderDevice, RenderQueue},
        view::VisibleEntities,
        Extract, RenderApp, RenderStage,
    },
};

use crate::{
    overlay_node::{graph, FrametimeOverlayBuffer, OverlayNode, OverlayPipeline},
    OVERLAY_SHADER_HANDLE,
};

const DEFAULT_DT_MIN: f32 = 1. / 240.;
const DEFAULT_DT_MAX: f32 = 1. / 15.;

/// The amount of frametimes kept in the buffer to be rendered in the display
/// Since the bars aren't all of the same size, this is the maximum value possible
// TODO make this configurable
pub const FRAMETIME_BUFFER_LEN: usize = 100;

#[derive(Debug, Clone, ShaderType, Resource)]
pub struct FrametimeOverlayConfig {
    pub dt_min: f32,
    pub dt_max: f32,
    pub dt_min_log2: f32,
    pub dt_max_log2: f32,
    pub max_width: f32,
    pub len: i32,
    pub colors: Mat4,
    pub dts: Vec4,
}

impl Default for FrametimeOverlayConfig {
    fn default() -> Self {
        Self {
            dt_min: DEFAULT_DT_MIN,
            dt_max: DEFAULT_DT_MAX,
            dt_min_log2: DEFAULT_DT_MIN.log2(),
            dt_max_log2: DEFAULT_DT_MAX.log2(),
            // There's probably a better value for this
            max_width: FRAMETIME_BUFFER_LEN as f32,
            len: FRAMETIME_BUFFER_LEN as i32,
            colors: Mat4::from_cols_array_2d(&[
                Color::BLUE.as_linear_rgba_f32(),
                Color::GREEN.as_linear_rgba_f32(),
                Color::YELLOW.as_linear_rgba_f32(),
                Color::RED.as_linear_rgba_f32(),
            ]),
            #[rustfmt::skip]
            dts: Vec4::new(
                DEFAULT_DT_MIN,
                1. / 60.,
                1. / 30.,
                DEFAULT_DT_MAX,
            ),
        }
    }
}

#[derive(Debug, Clone, ShaderType, Resource)]
pub struct Frametimes {
    pub values: [f32; FRAMETIME_BUFFER_LEN],
}

impl Default for Frametimes {
    fn default() -> Self {
        Self {
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

#[derive(Default)]
pub struct OverlayPlugin;
impl Plugin for OverlayPlugin {
    fn build(&self, app: &mut App) {
        if app
            .world
            .resource::<Diagnostics>()
            .get(FrameTimeDiagnosticsPlugin::FPS)
            .is_none()
        {
            app.add_plugin(FrameTimeDiagnosticsPlugin::default());
        }

        load_internal_asset!(
            app,
            OVERLAY_SHADER_HANDLE,
            "frametime_display.wgsl",
            Shader::from_wgsl
        );

        app.add_plugin(ExtractComponentPlugin::<CameraOverlay>::default())
            .add_startup_system(move |mut commands: Commands| {
                commands.spawn(CameraOverlayBundle::default());
            });

        let render_app = match app.get_sub_app_mut(RenderApp) {
            Ok(render_app) => render_app,
            Err(_) => return,
        };

        render_app
            .init_resource::<FrametimeOverlayConfig>()
            .init_resource::<Frametimes>()
            .init_resource::<OverlayPipeline>()
            .init_resource::<FrametimeOverlayBuffer>()
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
}

fn prepare_frametime_overlay_buffer(
    mut buffer: ResMut<FrametimeOverlayBuffer>,
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
