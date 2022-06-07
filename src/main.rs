use bevy::{
    asset::AssetServerSettings,
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    math::vec2,
    prelude::*,
    sprite::{Material2dPlugin, MaterialMesh2dBundle},
};
use material::FrametimeMaterial;

mod material;
mod plugin_2d;

const FRAMETIME_LEN: usize = 200;

fn main() {
    App::new()
        .insert_resource(AssetServerSettings {
            watch_for_changes: true,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(Material2dPlugin::<FrametimeMaterial>::default())
        .add_startup_system(setup)
        .add_startup_system(setup_3d)
        .add_startup_system(setup_2d)
        .add_system(update_frametimes)
        .run();
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

    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        camera: Camera {
            priority: 0,
            ..default()
        },
        ..default()
    });

    commands.spawn_bundle(Camera2dBundle {
        camera: Camera {
            priority: 1,
            ..default()
        },
        camera_2d: Camera2d {
            clear_color: bevy::core_pipeline::clear_color::ClearColorConfig::None,
        },
        ..default()
    });
}

fn setup_2d(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
        transform: Transform::default().with_scale(Vec3::splat(128.)),
        material: materials.add(ColorMaterial::from(Color::PURPLE)),
        ..default()
    });
}

fn setup_3d(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
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
