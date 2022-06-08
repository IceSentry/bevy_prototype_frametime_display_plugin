mod material;

use crate::material::FrametimeMaterial;
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    math::vec2,
    prelude::*,
    sprite::{Material2dPlugin, MaterialMesh2dBundle},
    window::WindowResized,
};
use material::FrametimeConfig;

/// The amount of frametimes kept in the buffer to be rendered in the display
/// Since the bars aren't all of the same size, this is the maximum value possible
// TODO make this configurable
const FRAMETIME_BUFFER_LEN: usize = 100;

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
            .add_plugin(Material2dPlugin::<FrametimeMaterial>::default())
            .init_resource::<FrametimeDisplayDescriptor>()
            .add_startup_system(setup)
            .add_system(update_frametimes)
            .add_system(resize);
    }
}

#[derive(Component)]
struct FrametimeDisplay;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut f_materials: ResMut<Assets<FrametimeMaterial>>,
    windows: Res<Windows>,
    desc: Res<FrametimeDisplayDescriptor>,
) {
    let window = windows.get_primary().expect("failed to get window");
    commands
        .spawn()
        .insert_bundle(MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Quad::new(vec2(desc.width, desc.height)).into())
                .into(),
            transform: match desc.position {
                Position::TopLeft => {
                    Transform::from_xyz(0.0, (window.height() / 2.0) - (desc.height / 2.0), 500.0)
                }
                Position::TopRight => Transform::from_xyz(
                    (window.width() / 2.0) - (desc.width / 2.0),
                    (window.height() / 2.0) - (desc.height / 2.0),
                    500.0,
                ),
            },
            material: f_materials.add(FrametimeMaterial {
                config: FrametimeConfig {
                    dt_min: desc.dt_min,
                    dt_max: desc.dt_max,
                    dt_min_log2: desc.dt_min.log2(),
                    dt_max_log2: desc.dt_max.log2(),
                    ..default()
                },
                ..default()
            }),
            ..default()
        })
        .insert(FrametimeDisplay);
}

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

fn update_frametimes(
    diagnostics: Res<Diagnostics>,
    mut materials: ResMut<Assets<FrametimeMaterial>>,
    mut materials_query: Query<&Handle<FrametimeMaterial>>,
) {
    if let Some(frame_time_diagnostic) = diagnostics.get(FrameTimeDiagnosticsPlugin::FRAME_TIME) {
        for material_handle in &mut materials_query {
            if let Some(material) = materials.get_mut(material_handle) {
                let dt = frame_time_diagnostic.value();
                material.frametimes.push(dt.unwrap_or(0.) as f32)
            }
        }
    }
}
