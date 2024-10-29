#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod decomposition;
mod point;

use std::time::Duration;

use bevy::color::palettes::css::{BLACK, VIOLET};
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::time::common_conditions::once_after_delay;
use bevy::window::{PresentMode, Window, WindowMode};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use decomposition::decompose_poly;

const TILE_SIZE: f32 = 16.0;

#[derive(Resource)]
struct Grid {
    size: IVec2,
    positions: Vec<IVec2>,
}

#[derive(Resource, Default)]
struct Graph {
    v: Vec<Vec2>,
    e: Vec<[u32; 2]>,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            size: IVec2::new(32, 32),
            positions: Vec::new(),
        }
    }
}

fn collinear(a: IVec2, b: IVec2, c: IVec2) -> bool {
    let dir_ab = b - a;
    let dir_bc = c - b;
    dir_ab == dir_bc
}

fn minimal_vertices(v: &Vec<IVec2>) -> Vec<IVec2> {
    let mut redundant_vert_indices = Vec::new();

    let n = v.len();
    if collinear(v[n - 1], v[0], v[1]) {
        redundant_vert_indices.push(0);
    }

    for i in 1..n {
        if collinear(v[i - 1], v[i], v[(i + 1) % n]) {
            redundant_vert_indices.push(i);
        }
    }
    redundant_vert_indices.reverse();

    let mut minimal_vertices = v.clone();
    for index in redundant_vert_indices {
        minimal_vertices.remove(index);
    }
    minimal_vertices
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::Fifo,
                    mode: WindowMode::Fullscreen,
                    fit_canvas_to_parent: true,
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest())
            .build(),))
        .add_plugins((
            LdtkPlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin {
                enabled: true,
                ..default()
            },
        ))
        .insert_resource(LevelSelection::index(0))
        .init_resource::<Grid>()
        .init_resource::<Graph>()
        .add_systems(Startup, setup)
        .add_systems(Update, (add_cells, draw_gizmos))
        .add_systems(
            Update,
            spawn_colliders.run_if(once_after_delay(Duration::from_secs_f32(0.5))),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut config_store: ResMut<GizmoConfigStore>,
) {
    let (config, _) = config_store.config_mut::<DefaultGizmoConfigGroup>();
    config.enabled = true;
    config.line_width = 5.0;

    let mut cam = Camera2dBundle::default();
    cam.projection.scaling_mode = ScalingMode::FixedVertical(300.0);
    cam.transform = Transform::from_translation(Vec3::new(128.0, 128.0, 0.0));
    commands.spawn(cam);

    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("map.ldtk"),
        ..Default::default()
    });
}

fn add_cells(mut grid: ResMut<Grid>, q_grid_coords: Query<&GridCoords, Added<IntGridCell>>) {
    for grid_coords in &q_grid_coords {
        grid.positions
            .push(IVec2::new(grid_coords.x, grid_coords.y));
    }
}

fn index_to_vertices(index: u8) -> Vec<Vec<IVec2>> {
    match index {
        0 => Vec::new(),
        1 => vec![vec![IVec2::X, IVec2::Y]],
        2 => vec![vec![IVec2::new(2, 1), IVec2::X]],
        3 => vec![vec![IVec2::new(2, 1), IVec2::Y]],
        4 => vec![vec![IVec2::new(1, 2), IVec2::new(2, 1)]],
        5 => vec![
            vec![IVec2::new(1, 2), IVec2::Y],
            vec![IVec2::X, IVec2::new(2, 1)],
        ],
        6 => vec![vec![IVec2::new(1, 2), IVec2::X]],
        7 => vec![vec![IVec2::new(1, 2), IVec2::Y]],
        8 => vec![vec![IVec2::Y, IVec2::new(1, 2)]],
        9 => vec![vec![IVec2::X, IVec2::new(1, 2)]],
        10 => vec![
            vec![IVec2::new(2, 1), IVec2::new(1, 2)],
            vec![IVec2::Y, IVec2::X],
        ],
        11 => vec![vec![IVec2::new(2, 1), IVec2::new(1, 2)]],
        12 => vec![vec![IVec2::Y, IVec2::new(2, 1)]],
        13 => vec![vec![IVec2::X, IVec2::new(2, 1)]],
        14 => vec![vec![IVec2::Y, IVec2::X]],
        15 => Vec::new(),
        _ => {
            error!("should never happen! Got bitmasks that are >15");
            Vec::new()
        }
    }
}

fn index_matrix(grid: &Grid) -> Vec<Vec<u8>> {
    let mut matrix = vec![vec![0; grid.size.y as usize]; grid.size.x as usize];
    for pos in &grid.positions {
        matrix[pos.x as usize][pos.y as usize] = 1;
    }

    let mut index_matrix = vec![vec![0; grid.size.y as usize]; grid.size.y as usize];

    for i in 0..matrix.len() - 1 {
        for j in 0..matrix[i].len() - 1 {
            index_matrix[i][j] = matrix[i][j] << 0
                | matrix[i + 1][j] << 1
                | matrix[i + 1][j + 1] << 2
                | matrix[i][j + 1] << 3;
        }
    }
    index_matrix
}

fn disjoint_vertices(grid: &Grid) -> Vec<Vec<IVec2>> {
    let index_matrix = index_matrix(grid);
    let mut vertices: Vec<Vec<IVec2>> = Vec::new();

    // Convert indices to vertices
    for i in 0..index_matrix.len() {
        for j in 0..index_matrix.len() {
            for vertex_pair in index_to_vertices(index_matrix[i][j]) {
                let v_pair = vertex_pair
                    .iter()
                    .map(|v| *v + 2 * IVec2::new(i as i32, j as i32))
                    .collect();
                vertices.push(v_pair);
            }
        }
    }
    vertices
}

fn connected_vertices(grid: &Grid) -> Vec<IVec2> {
    let mut vertices = disjoint_vertices(grid);
    // Merge disjoint graphs together until there is only one left
    // We do can do this by simply checking if the last and the first element of any two graphs
    // match. We know that this must be true for for all graphs with one other graph because the
    // whole collider must be closed and non-selfcrossing without loops.
    //
    // 0 --- 1 --- 2
    //              \
    //               \
    //                0           1 --- 0 --- 1
    //                 \         /
    //                  \       /
    //                   1 --- 0
    //
    // This for example, the first three vertices are already connected into one graph, then 2 and
    // 0 share one graph (edge in this case because I didn't draw it propely).
    while vertices.len() > 1 {
        let mut group_index = 0;
        for (i, vertex_group) in vertices.iter().enumerate() {
            if i == 0 {
                continue;
            }

            if vertices[0][vertices[0].len() - 1] == vertex_group[0] {
                group_index = i;
                break;
            }
        }

        assert!(group_index != 0);
        let mut new_group = vertices.remove(group_index);
        new_group.remove(0);
        vertices[0].append(&mut new_group);
    }
    let n = vertices[0].len() - 1;
    // First and last vertex should be equal, we now have a connected line, to bring it to a loop
    // we just remove the last vertex and it now "loops" to the first one.
    assert!(vertices[0][0] == vertices[0][n]);
    vertices[0].remove(n);
    vertices[0].clone()
}

fn vertices_and_indices(grid: &Grid) -> (Vec<Vec2>, Vec<[u32; 2]>) {
    let minimal_vertices = minimal_vertices(&connected_vertices(grid));

    let mut vertices = Vec::new();
    for uvert in &minimal_vertices {
        let v = Vec2::new(uvert.x as f32, uvert.y as f32) / 2.0 * TILE_SIZE;
        vertices.push(v);
    }

    let mut indices = Vec::new();
    for i in 0..vertices.len() - 1 {
        indices.push([i as u32, i as u32 + 1]);
    }
    indices.push([vertices.len() as u32 - 1, 0]);
    (vertices, indices)
}

fn disjoint_graphs(grid: &Res<Grid>) -> Vec<Vec<IVec2>> {
    let mut disjoint_graphs = Vec::new();
    let mut positions = grid.positions.clone();

    let mut graph = vec![vec![0; grid.size.y as usize]; grid.size.x as usize];
    for pos in &grid.positions {
        graph[pos.x as usize][pos.y as usize] = 1;
    }

    while !positions.is_empty() {
        let mut current_positions = Vec::new();
        let mut stack = vec![positions[0]];
        while let Some(n) = stack.pop() {
            // Out of bounds
            if n.x < 0 || n.y < 0 || n.x >= grid.size.x || n.y >= grid.size.y {
                continue;
            }
            let (Ok(x), Ok(y)): (Result<usize, _>, Result<usize, _>) =
                (n.x.try_into(), n.y.try_into())
            else {
                continue;
            };

            // We have hit a dead end (or a node we already visited)
            if graph[x][y] == 0 {
                continue;
            }
            graph[x][y] = 0;

            current_positions.push(n);
            // Delete the node from the positions, it should always be valid
            positions.swap_remove(
                positions
                    .iter()
                    .position(|x| *x == n)
                    .expect("node should be inside positions, something is fucky"),
            );

            stack.push(n + IVec2::X);
            stack.push(n + IVec2::Y);
            stack.push(n + IVec2::NEG_X);
            stack.push(n + IVec2::NEG_Y);
        }
        disjoint_graphs.push(current_positions);
    }
    disjoint_graphs
}

fn spawn_colliders(mut commands: Commands, grid: Res<Grid>) {
    for graph in disjoint_graphs(&grid) {
        let grid = Grid {
            size: grid.size,
            positions: graph,
        };
        let (vertices, _) = vertices_and_indices(&grid);

        let polygons = decompose_poly(&mut vertices.clone());
        for poly in &polygons {
            commands.spawn((
                Collider::compound(vec![(
                    Vec2::default(),
                    0.0,
                    Collider::convex_hull(poly).unwrap(),
                )]),
                ColliderDebugColor(VIOLET.into()),
                SpatialBundle::default(),
            ));
        }
    }
}

fn draw_gizmos(mut gizmos: Gizmos, graph: Res<Graph>) {
    if graph.v.is_empty() || graph.e.is_empty() {
        return;
    }

    for [i, j] in &graph.e {
        gizmos.circle_2d(graph.v[*i as usize], 5.0, BLACK);
        gizmos.line_2d(graph.v[*i as usize], graph.v[*j as usize], BLACK);
    }
}
