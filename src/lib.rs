// mod material;
mod overlay_node;
mod plugin;

use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::*,
    reflect::TypeUuid,
    window::WindowResized,
};
use overlay_node::FrametimeOverlayBuffer;
pub use plugin::{CameraOverlay, OverlayPlugin};

pub const OVERLAY_SHADER_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 1236245567947772696);

#[derive(Resource)]
pub struct FrametimeDisplayDescriptor {
    /// The width of the display in pixels
    pub width: f32,
    /// The height of the display in pixels
    pub height: f32,
    /// The minimum target value for a single frametime, used to determine the size and colors of the bars.
    /// It's generally recommended to pick a value that is about double the target framerate.
    /// So if you want to target 60 fps, then set it to `1.0/120.0` which is the frametime for 120 fps
    ///
    /// Defaults to `1. / 240.` or 240 fps
    pub dt_min: f32,
    /// The maximum target value for a single frametime, used to determine the size and colors of the bars
    ///
    /// Defaults to `1. / 15.` or 15 fps
    pub dt_max: f32,
    /// The position of the display
    pub position: Position,
}

pub enum Position {
    TopLeft,
    TopRight,
}

impl Default for FrametimeDisplayDescriptor {
    fn default() -> Self {
        Self {
            width: 400.,
            height: 100.,
            dt_min: 1. / 240.,
            dt_max: 1. / 15.,
            position: Position::TopLeft,
        }
    }
}

pub struct FrametimeDisplayPlugin;

impl Plugin for FrametimeDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(FrameTimeDiagnosticsPlugin::default())
            .init_resource::<FrametimeDisplayDescriptor>()
            .add_system(update_frametimes)
            .add_system(resize);
    }
}

#[derive(Component)]
struct FrametimeDisplay;

fn resize(
    mut resize_events: EventReader<WindowResized>,
    mut query: Query<&mut Transform, With<FrametimeDisplay>>,
    desc: Res<FrametimeDisplayDescriptor>,
) {
    for ev in resize_events.iter() {
        for mut transform in query.iter_mut() {
            transform.translation.x = match desc.position {
                Position::TopLeft => -(ev.width / 2.0) + (desc.width / 2.0),
                Position::TopRight => (ev.width / 2.0) - (desc.width / 2.0),
            };
            transform.translation.y = (ev.height / 2.0) - (desc.height / 2.0);
        }
    }
}

fn update_frametimes(diagnostics: Res<Diagnostics>, mut buffer: ResMut<FrametimeOverlayBuffer>) {
    if let Some(frame_time_diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FRAME_TIME) {
        let dt = frame_time_diagnostic.value().unwrap();
        buffer.frametimes_buffer.get_mut().push(dt as f32);
    }
}
