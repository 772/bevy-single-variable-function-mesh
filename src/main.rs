use bevy::prelude::*;
use bevy_single_variable_function_mesh::SingleVariableFunctionMesh;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .run();
}

fn circle(x: f32) -> f32 {
    (1.0 - x.powf(2.0)).powf(0.5)
}

fn squircle(x: f32) -> f32 {
    (1.0 - (x).abs().powf(4.0)).powf(0.25)
}

fn straight(_x: f32) -> f32 {
    1.0
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Flat ground.
    commands.spawn(PbrBundle {
        mesh: meshes.add(SingleVariableFunctionMesh {
            f1: |_x: f32| -> f32 { 10.0 },
            f1_x_start: -10.0,
            f1_x_end: 10.0,
            f2_x_start: 0.0,
            f2_x_end: 0.0,
            ..default()
        }),
        material: materials.add(StandardMaterial::default()),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });
    // Flat squircle.
    commands.spawn(PbrBundle {
        mesh: meshes.add(SingleVariableFunctionMesh {
            f1: squircle,
            f2_x_start: 0.0,
            f2_x_end: 0.0,
            ..default()
        }),
        material: materials.add(StandardMaterial::default()),
        transform: Transform::from_xyz(-4.0, 1.0, 0.0),
        ..default()
    });
    // A bit flat squircle.
    commands.spawn(PbrBundle {
        mesh: meshes.add(SingleVariableFunctionMesh {
            f1: squircle,
            f2: |x: f32| -> f32 { (1.0 - (x * 5.0).abs().powf(4.0)).powf(0.25) },
            f2_x_start: -0.2,
            f2_x_end: 0.2,
            ..default()
        }),
        material: materials.add(StandardMaterial::default()),
        transform: Transform::from_xyz(-2.0, 1.0, 0.0),
        ..default()
    });
    // Cylinder.
    commands.spawn(PbrBundle {
        mesh: meshes.add(SingleVariableFunctionMesh {
            f1: circle,
            f2: straight,
            ..default()
        }),
        material: materials.add(StandardMaterial::default()),
        transform: Transform::from_xyz(0.0, 1.0, 0.0),
        ..default()
    });
    // Ball.
    commands.spawn(PbrBundle {
        mesh: meshes.add(SingleVariableFunctionMesh {
            f1: circle,
            f2: circle,
            ..default()
        }),
        material: materials.add(StandardMaterial::default()),
        transform: Transform::from_xyz(2.0, 1.0, 0.0),
        ..default()
    });
    // Pyramid.
    commands.spawn(PbrBundle {
        mesh: meshes.add(SingleVariableFunctionMesh {
            f2: |x: f32| -> f32 { -0.5 * x + 0.5 },
            ..default()
        }),
        material: materials.add(StandardMaterial::default()),
        transform: Transform::from_xyz(4.0, 1.0, 0.0),
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
