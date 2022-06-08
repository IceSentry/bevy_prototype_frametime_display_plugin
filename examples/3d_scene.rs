use bevy::{asset::AssetServerSettings, prelude::*};
use bevy_prototype_frametime_display_plugin::{
    FrametimeDisplayDescriptor, FrametimeDisplayPlugin, Position,
};

fn main() {
    App::new()
        .insert_resource(AssetServerSettings {
            watch_for_changes: true,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        // If you need to configure it, simply insert a FrametimeDisplayDescriptor
        // and change any config
        .insert_resource(FrametimeDisplayDescriptor {
            width: 200.0,
            height: 50.0,
            position: Position::TopRight,
            ..default()
        })
        // Insert the plugin on the app
        .add_plugin(FrametimeDisplayPlugin)
        .add_startup_system(setup_cameras)
        .add_startup_system(setup_3d_scene)
        .run();
}

fn setup_cameras(mut commands: Commands) {
    // The plugin uses a 2d mesh to render so we need to spawn a 2d camera to render
    commands.spawn_bundle(Camera2dBundle {
        camera: Camera {
            // Since this example uses a Camera3d for the scene, we need to set the priority
            // that is higher than the default of 0
            priority: 1,
            ..default()
        },
        camera_2d: Camera2d {
            // Since we layer multiple cameras this one needs to not clear anything
            // otherwise it would clear the 3d camera
            clear_color: bevy::core_pipeline::clear_color::ClearColorConfig::None,
        },
        ..default()
    });
}

// This is simply the scene from the 3d_scene example of bevy
fn setup_3d_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // ground
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });

    // cube
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });

    // light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // camera
    commands.spawn_bundle(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
