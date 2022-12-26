use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};

/// A 2D or 3D mesh generated from a single-variable function.
pub struct SingleVariableFunctionMesh {
    pub f: fn(f32) -> f32,
    pub x_start: f32,
    pub x_end: f32,
    pub relative_depth: f32,
    pub vertices_per_side: usize,
    pub vertices_thickness: usize,
}

impl Default for SingleVariableFunctionMesh {
    fn default() -> Self {
        SingleVariableFunctionMesh {
            f: squircle,
            x_start: -1.0,
            x_end: 1.0,
            relative_depth: 0.0,
            vertices_per_side: 30,
            vertices_thickness: 1,
        }
    }
}

fn squircle(x: f32) -> f32 {
    (1.0 - (x).abs().powf(4.0)).powf(0.25)
}

impl From<SingleVariableFunctionMesh> for Mesh {
    fn from(mathfunction: SingleVariableFunctionMesh) -> Self {
        let ring = calculate_ring_of_vertices(
            mathfunction.f,
            mathfunction.x_start,
            mathfunction.x_end,
            mathfunction.vertices_per_side,
        );
        let ring2 = calculate_ring_of_vertices(
            mathfunction.f,
            mathfunction.x_start,
            mathfunction.x_end,
            mathfunction.vertices_thickness,
        );
        let amount = ring.len();
        let mut vertices: Vec<([f32; 3], [f32; 3], [f32; 2])> = Vec::with_capacity(amount + 1);
        let mut indeces: Vec<u32> = Vec::with_capacity(amount);
        let height = mathfunction.relative_depth;

        vertices.push(([0.0, -height, 0.0], [1.0, 1.0, 1.0], [0.0, 0.0]));
        for segment in 0..mathfunction.vertices_thickness {
            for i in 0..amount {
                let vorzeichen = {
                    if ring[i][1] >= 0.0 {
                        1.0
                    } else {
                        -1.0
                    }
                };
                let vorzeichen2 = {
                    if ring[i][0] >= 0.0 {
                        1.0
                    } else {
                        -1.0
                    }
                };
                let x = ring[i][0] + vorzeichen2 * ring2[segment][1] * mathfunction.relative_depth;
                let y = ring2[segment][0] / (1.0 / mathfunction.relative_depth);
                let z = ring[i][1] + vorzeichen * ring2[segment][1] * mathfunction.relative_depth;
                vertices.push(([x, y, z], [1.0, 1.0, 1.0], [0.0, 0.0]));
            }
        }
        vertices.push(([0.0, height, 0.0], [1.0, 1.0, 1.0], [0.0, 0.0]));

        for segment in 0..mathfunction.vertices_thickness {
            if segment == 0 {
                for i in 0..amount {
					if mathfunction.vertices_thickness != 1 {
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
                        tr = (segment * amount + 0 + 1) as u32;
                        br = ((segment - 1) * amount + 0 + 1) as u32;
                    }
                    indeces.append(&mut vec![br, tr, tl]);
                    indeces.append(&mut vec![bl, br, tl]);
                }
            }
            if segment == mathfunction.vertices_thickness - 1 {
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

fn calculate_ring_of_vertices(
    f: fn(f32) -> f32,
    x_start: f32,
    x_end: f32,
    vertices: usize,
) -> Vec<[f32; 2]> {
    assert!(x_start < x_end);
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
    let mut return_vec: Vec<[f32; 2]> = Vec::with_capacity(vertices);
    for e in &vec {
        return_vec.push([e.x, e.y]);
    }
    vec.reverse();
    vec.remove(vertices - 1);
    vec.remove(0);
    for e in &vec {
        return_vec.push([e.x, -e.y]);
    }
    return_vec
}
