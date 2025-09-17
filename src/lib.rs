use bevy::math::Vec3;
use bevy::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::asset::RenderAssetUsages;

/// A 2D or 3D mesh (`bevy::render::mesh::Mesh`) generated from a single-variable function
/// `f(f32) -> f32`.
#[derive(Debug, Clone, Copy)]
pub struct SingleVariableFunctionMesh {
    /// The function to be used as the upper half of the generated polygon.
    /// The function will be mirrored to the x-axis to generate the lower half of the polygon.
    /// If the mesh is 3D, the function will also be applied to the
    /// side vertices. Default = squircle.
    pub f1: fn(f32) -> f32,
    /// `f1` starts here. Together with `x_end`, this determines the size of the mesh.
    /// Must be lower than `x_end`. Default = -1.0.
    pub f1_x_start: f32,
    /// `f1` ends here. Together with `x_start`, this determines the size of the mesh.
    /// Must be bigger than `x_start`. Default = 1.0.
    pub f1_x_end: f32,
    /// The amount of vertices that are used for each upper half of the polygon.
    /// Should be at least 3. There will be (n - 2) * (2 * n - 2) + 2 vertices in total
    /// if 3D.  Default = 30.0.
    pub f1_vertices: usize,
    pub f2: fn(f32) -> f32,
    pub f2_x_start: f32,
    pub f2_x_end: f32,
    pub f2_vertices: usize,
}

impl Default for SingleVariableFunctionMesh {
    fn default() -> Self {
        SingleVariableFunctionMesh {
            f1: |_x: f32| -> f32 { 1.0 },
            f1_x_start: -1.0,
            f1_x_end: 1.0,
            f1_vertices: 18,
            f2: |_x: f32| -> f32 { 1.0 },
            f2_x_start: -1.0,
            f2_x_end: 1.0,
            f2_vertices: 18,
        }
    }
}

impl From<SingleVariableFunctionMesh> for Mesh {
    fn from(mathfunction: SingleVariableFunctionMesh) -> Self {
        debug_assert!(mathfunction.f1_x_start <= mathfunction.f1_x_end);
        debug_assert!(mathfunction.f2_x_start <= mathfunction.f2_x_end);
        let (ring_horizontal, maximum1) = calculate_ring_of_vertices(
            mathfunction.f1,
            mathfunction.f1_x_start,
            mathfunction.f1_x_end,
            mathfunction.f1_vertices,
            true,
        );
        let (ring_vertical, _) = calculate_ring_of_vertices(
            mathfunction.f2,
            mathfunction.f2_x_start,
            mathfunction.f2_x_end,
            mathfunction.f2_vertices,
            false,
        );
        let width_maximum = mathfunction.f1_x_end - mathfunction.f1_x_start;
        let amount = ring_horizontal.len();
        let mut amount_layers = mathfunction.f2_vertices - 1;
        if mathfunction.f2_x_start == mathfunction.f2_x_end {
            amount_layers = 1;
        }

        let mut vertices: Vec<([f32; 3], [f32; 3], [f32; 2])> =
            Vec::with_capacity(amount * amount_layers + 2);
        vertices.push((
            [0.0, mathfunction.f2_x_start, 0.0],
            [0.0, -1.0, 0.0],
            [0.5, 0.5],
        ));
        for i in ring_vertical.iter().take(amount_layers) {
            for (k, j) in ring_horizontal.iter().enumerate().take(amount) {
                // Place vertices.
                let (mut x, mut z) = (j.x, j.y);
                if amount_layers > 1 {
                    (x, z) = (x.signum() * (x.abs() * i.y), z.signum() * (z.abs() * i.y));
                }
                let y = i.x;

                // Create normals.
                let mut normal_horizontally =
                    Vec3::new(-j.slope_in_percentage.tan(), 0.0, 1.0).normalize();

                if k >= amount / 2 {
                    normal_horizontally[2] = -normal_horizontally[2];
                }
                let normal_vertical = Vec3::new(1.0, -i.slope_in_percentage.tan(), 1.0).normalize();
                let mut normals = [
                    normal_horizontally[0] / 3.0 * 2.0,
                    normal_vertical[1],
                    normal_horizontally[2] / 3.0 * 2.0,
                ];
                if amount_layers == 1 {
                    normals = [0.0, 1.0, 0.0];
                }
                let uv_x = (x + mathfunction.f1_x_start.abs()) / width_maximum;
                let uv_y = (z + maximum1) / (maximum1 * 2.0);
                vertices.push(([x, y, z], normals, [uv_x, uv_y]));
            }
        }
        vertices.push((
            [0.0, mathfunction.f2_x_end, 0.0],
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
    generate_lower_half: bool,
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
    if !generate_lower_half {
        return (vec, maximum);
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

    fn circle(x: f32) -> f32 {
        (1.0 - x.powf(2.0)).powf(0.5)
    }

    fn square(_x: f32) -> f32 {
        1.0
    }

    #[test]
    fn test_amount_of_vertices() {
        let square_2d_mesh: Mesh = SingleVariableFunctionMesh {
            f1: square,
            f1_x_start: -1.0,
            f1_x_end: 1.0,
            f1_vertices: 20,
            f2: square,
            f2_x_start: 0.0,
            f2_x_end: 0.0,
            f2_vertices: 20,
        }
        .into();
        let circle_2d_mesh: Mesh = SingleVariableFunctionMesh {
            f1: circle,
            f1_x_start: -1.0,
            f1_x_end: 1.0,
            f1_vertices: 20,
            f2: circle,
            f2_x_start: 0.0,
            f2_x_end: 0.0,
            f2_vertices: 20,
        }
        .into();
        assert_eq!(
            square_2d_mesh.count_vertices(),
            circle_2d_mesh.count_vertices() - 2
        );
    }
}
