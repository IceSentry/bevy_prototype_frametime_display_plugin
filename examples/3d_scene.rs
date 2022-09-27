use bevy::{asset::AssetServerSettings, prelude::*};
use bevy_prototype_frametime_display_plugin::{CameraOverlay, OverlayPlugin};

fn main() {
    App::new()
        .insert_resource(AssetServerSettings {
            watch_for_changes: true,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        // If you need to configure it, simply insert a FrametimeDisplayDescriptor
        // and change any config
        // .insert_resource(FrametimeDisplayDescriptor {
        //     width: 200.0,
        //     height: 50.0,
        //     position: Position::TopRight,
        //     ..default()
        // })
        // Insert the plugin on the app
        .add_plugin(OverlayPlugin)
        .add_startup_system(setup_3d_scene)
        .run();
}

// This is simply the scene from the 3d_scene example of bevy
fn setup_3d_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // ground
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });

    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
        material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        CameraOverlay,
    ));
}
