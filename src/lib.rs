use bevy::math::Vec3;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

/// A 2D or 3D mesh (`bevy::render::mesh::Mesh`) generated from a single-variable function
/// `f(f32) -> f32`.
#[derive(Debug, Clone, Copy)]
pub struct SingleVariableFunctionMesh {
    /// The function to be used as the upper half of the generated polygon.
    /// The function will be mirrored to the x-axis to generate the lower half of the polygon.
    /// If the mesh is 3D (`relative_height` > 0.0), the function will also be applied to the
    /// side vertices. Default = squircle.
    pub f: fn(f32) -> f32,
    /// `f` starts here. Together with `x_end`, this determines the size of the mesh.
    /// Must be lower than `x_end`. Default = -1.0.
    pub x_start: f32,
    /// `f` ends here. Together with `x_start`, this determines the size of the mesh.
    /// Must be bigger than `x_start`. Default = 1.0.
    pub x_end: f32,
    /// The amount of vertices that are used for each upper half of the polygon.
    /// Should be at least 3. There will be (n - 2) * (2 * n - 2) + 2 vertices in total
    /// if `relative_height` > 0.0.  Default = 30.0.
    pub vertices: usize,
    /// If `relative_height` is 0.0, then the mesh is a 2D poglygon without height. If 1.0,
    /// the mesh is fully 3D without any bigger flat surface. Default = 0.1.
    pub relative_height: f32,
}

impl Default for SingleVariableFunctionMesh {
    fn default() -> Self {
        SingleVariableFunctionMesh {
            f: squircle,
            x_start: -1.0,
            x_end: 1.0,
            vertices: 30,
            relative_height: 0.1,
        }
    }
}

impl From<SingleVariableFunctionMesh> for Mesh {
    fn from(mathfunction: SingleVariableFunctionMesh) -> Self {
        debug_assert!(0.0 <= mathfunction.relative_height && mathfunction.relative_height <= 1.0);
        debug_assert!(mathfunction.x_start < mathfunction.x_end);
        let (ring, maximum) = calculate_ring_of_vertices(
            mathfunction.f,
            mathfunction.x_start,
            mathfunction.x_end,
            mathfunction.vertices,
        );
        let width_maximum = mathfunction.x_end - mathfunction.x_start;
        let amount = ring.len();
        let mut amount_layers = (ring.len() - 2) / 2;
        if mathfunction.relative_height == 0.0 {
            amount_layers = 1;
        }

        // Create vertices.
        let mut vertices: Vec<([f32; 3], [f32; 3], [f32; 2])> =
            Vec::with_capacity(amount * amount_layers + 2);
        vertices.push((
            [
                0.0,
                mathfunction.x_start * mathfunction.relative_height,
                0.0,
            ],
            [0.0, -1.0, 0.0],
            [0.5, 0.5],
        ));
        if mathfunction.relative_height >= 0.0 {
            for i in 0..amount_layers {
                for j in 0..amount {
                    let (mut x, mut z) = (ring[j].x, ring[j].y);
                    (x, z) = (
                        x.signum()
                            * (x.abs()
                                * (ring[i].y * mathfunction.relative_height
                                    + maximum * (1.0 - mathfunction.relative_height))),
                        z.signum()
                            * (z.abs()
                                * (ring[i].y * mathfunction.relative_height
                                    + maximum * (1.0 - mathfunction.relative_height))),
                    );
                    let y = ring[i].x * mathfunction.relative_height;

                    let mut normal_horizontally =
                        Vec3::new(-ring[j].slope_in_percentage.tan(), 0.0, 1.0).normalize();
                    if j >= amount / 2 {
                        normal_horizontally[2] = -normal_horizontally[2];
                    }
                    let normal_vertical =
                        Vec3::new(1.0, -ring[i].slope_in_percentage.tan(), 1.0).normalize();
                    let mut normals = [
                        normal_horizontally[0] / 3.0 * 2.0,
                        normal_vertical[1],
                        normal_horizontally[2] / 3.0 * 2.0,
                    ];
                    if amount_layers == 1 {
                        normals = [0.0, 1.0, 0.0];
                    }

                    let realtive_texture_size = 1.0 - (y / maximum.abs());
                    let uv_x =
                        (x * realtive_texture_size + mathfunction.x_start.abs()) / width_maximum;
                    let uv_y = (z * realtive_texture_size + maximum) / (maximum * 2.0);
                    vertices.push(([x, y, z], normals, [uv_x, uv_y]));
                }
            }
        }
        vertices.push((
            [0.0, mathfunction.x_end * mathfunction.relative_height, 0.0],
            [0.0, 1.0, 0.0],
            [0.5, 0.5],
        ));

        // Create faces (indeces).
        let mut indeces: Vec<u32> = Vec::with_capacity((amount * amount_layers + 2) * 3);
        for i in 0..amount {
            if amount_layers > 1 {
                indeces.append(&mut vec![
                    ((i + 1) % amount + 1).try_into().unwrap(),
                    (i + 1).try_into().unwrap(),
                    0,
                ]);
            }
            indeces.append(&mut vec![
                ((amount_layers - 1) * amount + i + 1).try_into().unwrap(),
                ((amount_layers - 1) * amount + (i + 1) % amount + 1)
                    .try_into()
                    .unwrap(),
                (amount_layers * amount + 1).try_into().unwrap(),
            ]);
        }
        for segment in 1..amount_layers {
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

        let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
        let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
        let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();
        let mut mesh = Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        );
        mesh.insert_indices(Indices::U32(indeces));
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
) -> (Vec<Position>, f32) {
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
    let mut maximum = 0.0;
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
        if f(new_x) > maximum {
            maximum = f(new_x);
        }
    }
    let mut lower_half = vec.clone();
    if f(lower_half[0].x) != 0.0 {
        lower_half.remove(0);
    }
    lower_half.reverse();
    if f(lower_half[0].x) != 0.0 {
        lower_half.remove(0);
    }
    for vertex in &lower_half {
        vec.push(Position {
            x: vertex.x,
            y: -vertex.y,
            slope_in_percentage: vertex.slope_in_percentage,
        });
    }
    (vec, maximum)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(_x: f32) -> f32 {
        1.0
    }

    #[test]
    fn test_amount_of_vertices() {
        let square_2d_mesh: Mesh = SingleVariableFunctionMesh {
            f: square,
            relative_height: 0.0,
            x_start: -1.0,
            x_end: 1.0,
            vertices: 20,
        }
        .into();
        let circle_2d_mesh: Mesh = SingleVariableFunctionMesh {
            f: circle,
            relative_height: 0.0,
            x_start: -1.0,
            x_end: 1.0,
            vertices: 20,
        }
        .into();
        assert_eq!(
            square_2d_mesh.count_vertices(),
            circle_2d_mesh.count_vertices() - 2
        );
    }
}
