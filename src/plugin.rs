use crate::material::FrametimeMaterial;
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    math::vec2,
    prelude::*,
    sprite::{Material2dPlugin, MaterialMesh2dBundle},
    window::WindowResized,
};

// TODO make this configurable
const FRAMETIME_LEN: usize = 200;
const DISPLAY_WIDTH: f32 = 400.0;
const DISPLAY_HEIGHT: f32 = 100.0;

pub struct FrametimeDisplayPlugin;

impl Plugin for FrametimeDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_plugin(Material2dPlugin::<FrametimeMaterial>::default())
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
) {
    let window = windows.get_primary().expect("failed to get window");
    commands
        .spawn()
        .insert_bundle(MaterialMesh2dBundle {
            mesh: meshes
                .add(shape::Quad::new(vec2(DISPLAY_WIDTH, DISPLAY_HEIGHT)).into())
                .into(),
            transform: Transform::from_xyz(
                (window.width() / 2.0) - (DISPLAY_WIDTH / 2.0),
                (window.height() / 2.0) - (DISPLAY_HEIGHT / 2.0),
                500.0,
            ),
            material: f_materials.add(FrametimeMaterial::default()),
            ..default()
        })
        .insert(FrametimeDisplay);
}

fn resize(
    mut resize_events: EventReader<WindowResized>,
    mut query: Query<&mut Transform, With<FrametimeDisplay>>,
) {
    for ev in resize_events.iter() {
        for mut transform in query.iter_mut() {
            transform.translation.x = (ev.width / 2.0) - (DISPLAY_WIDTH / 2.0);
            transform.translation.y = (ev.height / 2.0) - (DISPLAY_HEIGHT / 2.0);
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
                material.frametimes.rotate_left(1);
                let dt = frame_time_diagnostic.value();
                material.frametimes[FRAMETIME_LEN - 1] = dt.unwrap_or(0.0) as f32;
            }
        }
    }
}
