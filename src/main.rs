use bevy::prelude::*;
use bevy_single_variable_function_mesh::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PbrBundle {
        mesh: meshes.add(SingleVariableFunctionMesh {
            f1: squircle, // or circle
            f2_x_end: 0.0,
            ..default()
        }),
        material: materials.add(StandardMaterial::default()),
        transform: Transform::from_xyz(-2.5, 1.0, 0.0),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(SingleVariableFunctionMesh {
            f1: squircle, // or circle
            f2_x_end: 0.2,
            ..default()
        }),
        material: materials.add(StandardMaterial::default()),
        transform: Transform::from_xyz(0.0, 1.0, 0.0),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(SingleVariableFunctionMesh {
            f1: squircle, // or circle
            ..default()
        }),
        material: materials.add(StandardMaterial::default()),
        transform: Transform::from_xyz(2.5, 1.0, 0.0),
        ..default()
    });

    // Light.
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // Camera.
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::ZERO),
        ..default()
    });
}
