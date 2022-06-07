use crate::material::FrametimeMaterial;
use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    math::vec2,
    prelude::*,
    sprite::{Material2dPlugin, MaterialMesh2dBundle},
};

// TODO make this configurable
const FRAMETIME_LEN: usize = 200;

pub struct FrametimeDisplayPlugin;

impl Plugin for FrametimeDisplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(FrameTimeDiagnosticsPlugin::default())
            .add_plugin(Material2dPlugin::<FrametimeMaterial>::default())
            .add_startup_system(setup)
            .add_system(update_frametimes);
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut f_materials: ResMut<Assets<FrametimeMaterial>>,
) {
    commands.spawn().insert_bundle(MaterialMesh2dBundle {
        mesh: meshes
            .add(shape::Quad::new(vec2(400.0, 100.0)).into())
            .into(),
        // TODO move to corner and handle resizing
        transform: Transform::from_xyz(0.0, 0.0, 500.0),
        material: f_materials.add(FrametimeMaterial::default()),
        ..default()
    });
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
