use bevy::math::Vec3;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};

/// A 2D or 3D mesh (`bevy::render::mesh::Mesh`) generated from a single-variable function
/// `f(f32) -> f32`.
#[derive(Debug, Clone, Copy)]
pub struct SingleVariableFunctionMesh {
    /// The function to be used as the upper half of the generated polygon.
    /// The function will be mirrored to the x-axis to generate the lower half of the polygon.
    /// If the mesh is 3D (`relative_height` > 0.0), the function will also be applied to the
    /// side vertices.
    pub f: fn(f32) -> f32,
    /// `f` starts here. Together with `x_end`, this determines the size of the mesh.
    /// Must be lower than `x_end`.
    pub x_start: f32,
    /// `f` ends here. Together with `x_start`, this determines the size of the mesh.
    /// Must be bigger than `x_start`.
    pub x_end: f32,
    /// The amount of vertices that are used for each upper half of the polygon.
    /// Should be at least 3.
    pub vertices: usize,
    /// If `relative_height` is 0.0, then the mesh is 2D poglygon without height. If 1.0, the mesh
    /// is fully 3D without any bigger flat surface.
    pub relative_height: f32,
}

impl Default for SingleVariableFunctionMesh {
    fn default() -> Self {
        SingleVariableFunctionMesh {
            f: squircle,
            x_start: -1.0,
            x_end: 1.0,
            vertices: 60,
            relative_height: 0.1,
        }
    }
}

impl From<SingleVariableFunctionMesh> for Mesh {
    fn from(mut mathfunction: SingleVariableFunctionMesh) -> Self {
        debug_assert!(0.0 <= mathfunction.relative_height && mathfunction.relative_height <= 1.0);
        debug_assert!(mathfunction.x_start < mathfunction.x_end);
        let ring = calculate_ring_of_vertices(
            mathfunction.f,
            mathfunction.x_start,
            mathfunction.x_end,
            mathfunction.vertices,
        );
        let amount = ring.len();
        let amount_layers = ring.len() / 2;
        let mut vertices: Vec<([f32; 3], [f32; 3], [f32; 2])> =
            Vec::with_capacity(amount * amount_layers + 2);
        let mut indeces: Vec<u32> = Vec::with_capacity((amount * amount_layers + 2) * 3);
        if mathfunction.relative_height == 1.0 {
            mathfunction.relative_height = 0.99; // TODO.
        }

        vertices.push((
            [0.0, -mathfunction.relative_height, 0.0],
            [0.0, -1.0, 0.0],
            [0.0, 0.0],
        ));
        for i in 0..amount_layers {
            for j in 0..amount {
                let (mut x, mut z) = (
                    ring[j].x * (1.0 - mathfunction.relative_height),
                    ring[j].y * (1.0 - mathfunction.relative_height),
                );
                let distance = (x.powf(2.0) + z.powf(2.0)).powf(0.5);
                let new_distance = ring[i].y * mathfunction.relative_height;
                (x, z) = (
                    x / distance * (distance + new_distance),
                    z / distance * (distance + new_distance),
                );
                let y = ring[i].x * mathfunction.relative_height;
                let mut normal1 =
                    Vec3::new(-ring[j].slope_in_percentage.tan(), 0.0, 1.0).normalize();
                let mut normal2 =
                    Vec3::new(1.0, -ring[i].slope_in_percentage.tan().abs(), 1.0).normalize();
                if j >= amount / 2 {
                    normal1[2] = -normal1[2];
                }
                if i >= amount_layers / 2 {
                    normal2[1] = -normal2[1];
                }
                if amount_layers > 1 {
                    normal1[0] = normal1[0] / 3.0 * 2.0;
                    normal1[1] = normal2[1];
                    normal1[2] = normal1[2] / 3.0 * 2.0;
                }
                vertices.push(([x, y, z], normal1.into(), [0.0, 0.0]));
            }
        }
        vertices.push((
            [0.0, 1.0 - mathfunction.relative_height, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0],
        ));

        for segment in 0..mathfunction.vertices {
            if segment == 0 {
                for i in 0..amount {
                    if mathfunction.vertices != 1 {
                        if i == amount - 1 {
                            indeces.append(&mut vec![1, (i + 1).try_into().unwrap(), 0]);
                        } else {
                            indeces.append(&mut vec![
                                (i + 2).try_into().unwrap(),
                                (i + 1).try_into().unwrap(),
                                0,
                            ]);
                        }
                    }
                }
            } else {
                for i in 0..amount {
                    let tl = (segment * amount + i + 1) as u32;
                    let mut tr = (segment * amount + i + 2) as u32;
                    let bl = ((segment - 1) * amount + i + 1) as u32;
                    let mut br = ((segment - 1) * amount + i + 2) as u32;
                    if i == amount - 1 {
                        tr = (segment * amount + 1) as u32;
                        br = ((segment - 1) * amount + 1) as u32;
                    }
                    indeces.append(&mut vec![br, tr, tl]);
                    indeces.append(&mut vec![bl, br, tl]);
                }
            }
            if segment == mathfunction.vertices - 1 {
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

/// An example for a single-variable function.
#[allow(dead_code)]
pub fn squircle(x: f32) -> f32 {
    (1.0 - (x).abs().powf(4.0)).powf(0.25)
}

/// An example for a single-variable function.
#[allow(dead_code)]
pub fn circle(x: f32) -> f32 {
    (1.0 - x.powf(2.0)).powf(0.5)
}

#[derive(Copy, Clone, Debug)]
struct Position {
    x: f32,
    y: f32,
    slope_in_percentage: f32,
}

fn calculate_ring_of_vertices(
    f: fn(f32) -> f32,
    x_start: f32,
    x_end: f32,
    vertices: usize,
) -> Vec<Position> {
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
    let mut vec: Vec<Position> = Vec::with_capacity(vertices);
    vec.push(start);
    vec.push(end);
    for _ in 2..vertices {
        let (mut index, mut max_slope_difference, mut max_x_difference) = (1, 0.0, 0.0);
        for j in 1..vec.len() {
            let new_x = vec[j - 1].x + (vec[j].x - vec[j - 1].x) / 2.0;
            let new_m = ((f(new_x + delta) - f(new_x)) / (delta)).atan();
            let x_difference = vec[j].x - vec[j - 1].x;
            let slope_difference = (new_m - vec[j].slope_in_percentage).abs()
                + (new_m - vec[j - 1].slope_in_percentage).abs();
            if slope_difference > max_slope_difference
                || (slope_difference == max_slope_difference && x_difference > max_x_difference)
            {
                (index, max_slope_difference, max_x_difference) =
                    (j, slope_difference, x_difference);
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
    let mut lower_half = vec.clone();
    lower_half.reverse();
    for vertex in &lower_half {
        vec.push(Position {
            x: vertex.x,
            y: -vertex.y,
            slope_in_percentage: vertex.slope_in_percentage,
        });
    }
    vec
}
