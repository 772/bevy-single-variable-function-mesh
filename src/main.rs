use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::mesh::PrimitiveTopology;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_startup_system(setup)
        .add_system(rotate)
        .run();
}

#[derive(Component)]
struct Shape;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(MathFunction2D::default().into()),
            material: materials.add(StandardMaterial {
                base_color: Color::hsl(0.3, 0.5, 0.55),
                ..default()
            }),
            transform: Transform::from_xyz(0.0, 1.0, 0.0),
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
        mesh: meshes.add(shape::Plane { size: 40. }.into()),
        material: materials.add(Color::WHITE.into()),
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

/// A 2D or 3D mesh generated from a single-variable function.
pub struct MathFunction2D {
    pub size: f32,
    pub height: f32,
    pub segments: usize,
    pub vertices_per_side: usize,
    pub f: fn(f32) -> f32,
    pub x_start: f32,
    pub x_end: f32,
}

impl Default for MathFunction2D {
    fn default() -> Self {
        MathFunction2D {
            size: 1.0,
            height: 0.5,
            segments: 1,
            vertices_per_side: 50,
            f: squircle,
            x_start: -1.0,
            x_end: 1.0,
        }
    }
}

fn squircle(x: f32) -> f32 {
    (1.0 - (x).abs().powf(4.0)).powf(0.25)
}

impl From<MathFunction2D> for Mesh {
    fn from(mathfunction: MathFunction2D) -> Self {
        let ring = get_ring_from_math_function(
            mathfunction.x_start,
            mathfunction.x_end,
            mathfunction.f,
            mathfunction.vertices_per_side,
        );
        let amount = ring.len();
        let mut vertices: Vec<([f32; 3], [f32; 3], [f32; 2])> = Vec::with_capacity(amount + 1);
        let mut indeces: Vec<u32> = Vec::with_capacity(amount);

        vertices.push(([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [0.0, 0.0]));
        for segment in 0..mathfunction.segments {
            for i in 0..amount {
                let x = ring[i][0];
                let y = segment as f32 * mathfunction.height / mathfunction.segments as f32;
                let z = ring[i][1];
                vertices.push(([x, y, z], [1.0, 1.0, 1.0], [0.0, 0.0]));
            }
        }
        vertices.push(([0.0, 1.0, 0.0], [1.0, 1.0, 1.0], [0.0, 0.0]));

        for segment in 0..mathfunction.segments {
            if segment == 0 {
                for i in 0..amount {
                    if i == amount - 1 {
                        indeces.append(&mut vec![
                            0,
                            (segment * amount + i + 1).try_into().unwrap(),
                            1,
                        ]);
                    } else {
                        indeces.append(&mut vec![
                            0,
                            (segment * amount + i + 1).try_into().unwrap(),
                            (segment * amount + i + 2).try_into().unwrap(),
                        ]);
                    }
                }
            } else {
                /*let tl = 0;
                let tr = 0;
                let bl = 0;
                let br = 0;*/
            }
            if segment == mathfunction.segments - 1 {
                for i in 0..amount {
                    if i == amount - 1 {
                        indeces.append(&mut vec![
                            (segment * amount + 2).try_into().unwrap(),
                            (segment * amount + i + 1).try_into().unwrap(),
                            (segment * amount + 1).try_into().unwrap(),
                        ]);
                    } else {
                        indeces.append(&mut vec![
                            (segment * amount + 2).try_into().unwrap(),
                            (segment * amount + i + 1).try_into().unwrap(),
                            (segment * amount + i + 2).try_into().unwrap(),
                        ]);
                    }
                }
            }
        }

        let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
        let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
        let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(Indices::U32(indeces)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}

#[derive(Copy, Clone, Debug)]
struct Position {
    x: f32,
    y: f32,
    slope_in_percentage: f32,
}

fn get_ring_from_math_function(
    x_start: f32,
    x_end: f32,
    f: fn(f32) -> f32,
    vertices_per_side: usize,
) -> Vec<[f32; 2]> {
    assert!(x_start < x_end);
    assert!(vertices_per_side > 2);
    let delta = 0.000001;
    let start = Position {
        x: x_start,
        y: f(x_start),
        slope_in_percentage: ((f(x_start + delta) - f(x_start)) / (delta)).atan(),
    };
    let end = Position {
        x: x_end,
        y: f(x_end),
        slope_in_percentage: ((f(x_end) - f(x_end - delta)) / (delta)).atan(),
    };
    let mut vec: Vec<Position> = Vec::with_capacity(vertices_per_side);
    vec.push(start);
    vec.push(end);
    for _ in 2..vertices_per_side {
        let (mut index, mut max_absolute_difference) = (1, 0.0);
        for j in 1..vec.len() {
            let new_x = vec[j - 1].x + (vec[j].x - vec[j - 1].x) / 2.0;
            let new_m = ((f(new_x + delta) - f(new_x)) / (delta)).atan();
            let absolute_difference = (new_m - vec[j].slope_in_percentage).abs()
                + (new_m - vec[j - 1].slope_in_percentage).abs();
            if absolute_difference >= max_absolute_difference {
                index = j;
                max_absolute_difference = absolute_difference;
            }
        }
        let new_x = vec[index - 1].x + (vec[index].x - vec[index - 1].x) / 2.0;
        vec.insert(
            index,
            Position {
                x: new_x,
                y: f(new_x),
                slope_in_percentage: ((f(new_x + delta) - f(new_x)) / (delta)).atan(),
            },
        );
    }
    let mut return_vec: Vec<[f32; 2]> = Vec::with_capacity(vertices_per_side);
    for e in &vec {
        return_vec.push([e.x, e.y]);
    }
    vec.reverse();
    vec.remove(vertices_per_side - 1);
    vec.remove(0);
    for e in &vec {
        return_vec.push([e.x, -e.y]);
    }
    return_vec
}
