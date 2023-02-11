use bevy::math::f32::Quat;
use bevy::prelude::*;
use bevy_single_variable_function_mesh::SingleVariableFunctionMesh;
use rand::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_startup_system(setup)
        .add_system(rotate)
        .run();
}

#[derive(Component)]
struct Shape;

pub fn big_squircle(x: f32) -> f32 {
    (1.0 - x.powf(2.0)).powf(0.25) - x * 0.1 - 0.1
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // load a texture and retrieve its aspect ratio
    let texture_handle = asset_server.load("holz.png");
    let texture_puni = asset_server.load("puni.png");
    let texture_red = asset_server.load("red.png");
    let texture_king = asset_server.load("king.png");
    //let texture_handle2 = asset_server.load("bildchen3.png");

    // this material renders the texture normally
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(texture_handle.clone()),
        ..default()
    });

    let blue = materials.add(StandardMaterial {
        //base_color: Color::rgb(0.686, 0.486, 0.945),
        base_color_texture: Some(texture_puni.clone()),
        perceptual_roughness: 0.8,
        //normal_map_texture: Some(texture_handle2.clone()),
        ..default()
    });
    let red = materials.add(StandardMaterial {
        base_color_texture: Some(texture_red.clone()),
        perceptual_roughness: 0.8,
        ..default()
    });
    let king = materials.add(StandardMaterial {
        base_color_texture: Some(texture_king.clone()),
        perceptual_roughness: 0.8,
        ..default()
    });

    let mut mesh1: Mesh = SingleVariableFunctionMesh {
        f: big_squircle,
        relative_height: 0.0,
        x_start: -1.0,
        x_end: 0.999,
        ..default()
    }
    .into();
    mesh1.generate_tangents().unwrap();

    let mut mesh2: Mesh = SingleVariableFunctionMesh {
        f: big_squircle,
        relative_height: 0.0,
        x_start: -1.0,
        x_end: 0.999,
        ..default()
    }
    .into();
    mesh2.generate_tangents().unwrap();

    let mut gen = rand::thread_rng();
    for _ in 0..18 {
        let x: f32 = gen.gen_range(0..1600) as f32 / 100.0;
        let z: f32 = gen.gen_range(0..2000) as f32 / 100.0;
        let height: f32 = gen.gen_range(0..100) as f32 / 100.0;
        let rotation: f32 = gen.gen_range(0..100) as f32 / 100.0;
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(mesh1.clone()),
                material: blue.clone(),
                transform: Transform {
                    translation: [-8.0 + x, 1.0, 3.0 - z].into(),
                    scale: [1.0, 0.5 + height, 1.0].into(),
                    rotation: Quat::from_xyzw(0.0, -0.5 + rotation, 0.0, 1.0),
                },
                ..default()
            },
            Shape,
        ));
    }
    for _ in 0..4 {
        let x: f32 = gen.gen_range(0..1600) as f32 / 100.0;
        let z: f32 = gen.gen_range(0..2000) as f32 / 100.0;
        let height: f32 = gen.gen_range(0..100) as f32 / 100.0;
        let rotation: f32 = gen.gen_range(0..100) as f32 / 100.0;
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(mesh1.clone()),
                material: red.clone(),
                transform: Transform {
                    translation: [-8.0 + x, 1.0, 3.0 - z].into(),
                    scale: [1.0, 0.5 + height, 1.0].into(),
                    rotation: Quat::from_xyzw(0.0, -0.5 + rotation, 0.0, 1.0),
                },
                ..default()
            },
            Shape,
        ));
    }
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(mesh1.clone()),
            material: king.clone(),
            transform: Transform {
                translation: [0.0, 1.0, 3.5].into(),
                scale: [2.0, 1.4, 2.0].into(),
                rotation: Quat::from_xyzw(0.0, 0.0, 0.0, 1.0),
            },
            ..default()
        },
        Shape,
    ));

    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 9000.0,
            range: 100.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(8.0, 16.0, 8.0),
        ..default()
    });

    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane { size: 60. }.into()),
        transform: Transform::from_xyz(0.0, 0.0, -10.0),
        material: material_handle,
        ..default()
    });

    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 6., 12.0).looking_at(Vec3::new(0., 1., 0.), Vec3::Y),
        ..default()
    });
}

fn rotate(mut query: Query<&mut Transform, With<Shape>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_x(time.delta_seconds() / 2.);
    }
}
