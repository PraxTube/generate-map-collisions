#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use std::time::Duration;

use bevy::color::palettes::css::{PINK, RED};
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::time::common_conditions::once_after_delay;
use bevy::window::{PresentMode, Window, WindowMode};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

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

fn main() {
    App::new()
        .add_plugins((DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: PresentMode::Fifo,
                    mode: WindowMode::Windowed,
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

fn index_to_vertices(index: u8) -> Vec<Vec<UVec2>> {
    match index {
        0 => Vec::new(),
        1 => vec![vec![UVec2::X, UVec2::Y]],
        2 => vec![vec![UVec2::new(2, 1), UVec2::X]],
        3 => vec![vec![UVec2::new(2, 1), UVec2::Y]],
        4 => vec![vec![UVec2::new(1, 2), UVec2::new(2, 1)]],
        5 => vec![
            vec![UVec2::new(1, 2), UVec2::Y],
            vec![UVec2::X, UVec2::new(2, 1)],
        ],
        6 => vec![vec![UVec2::new(1, 2), UVec2::X]],
        7 => vec![vec![UVec2::new(1, 2), UVec2::Y]],
        8 => vec![vec![UVec2::Y, UVec2::new(1, 2)]],
        9 => vec![vec![UVec2::X, UVec2::new(1, 2)]],
        10 => vec![
            vec![UVec2::new(2, 1), UVec2::new(1, 2)],
            vec![UVec2::Y, UVec2::X],
        ],
        11 => vec![vec![UVec2::new(2, 1), UVec2::new(1, 2)]],
        12 => vec![vec![UVec2::Y, UVec2::new(2, 1)]],
        13 => vec![vec![UVec2::X, UVec2::new(2, 1)]],
        14 => vec![vec![UVec2::Y, UVec2::X]],
        15 => Vec::new(),
        _ => {
            error!("should never happen! Got bitmasks that are >15");
            Vec::new()
        }
    }
}

fn spawn_colliders(mut commands: Commands, grid: Res<Grid>, mut graph: ResMut<Graph>) {
    let mut matrix = vec![vec![0; grid.size.y as usize]; grid.size.y as usize];
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
            if index_matrix[i][j] != 0 {
                // info!("{}", index_matrix[i][j]);
            }
        }
    }

    let mut vertices: Vec<Vec<UVec2>> = Vec::new();

    for i in 0..index_matrix.len() {
        for j in 0..index_matrix.len() {
            for vertex_pair in index_to_vertices(index_matrix[i][j]) {
                let v_pair = vertex_pair
                    .iter()
                    .map(|v| *v + 2 * UVec2::new(i as u32, j as u32))
                    .collect();
                vertices.push(v_pair);
            }
        }
    }

    while vertices.len() > 1 {
        // info!("verts: {:?}", vertices);
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
    // we just remove the last vertex which and it now "loops" to the first one.
    assert!(vertices[0][0] == vertices[0][n]);
    vertices[0].remove(n);

    info!("DOOOONE");
    // info!("{:?}", vertices);

    let mut collider_vertices = Vec::new();

    for uvert in &vertices[0] {
        let v = Vec2::new(uvert.x as f32, uvert.y as f32) / 2.0 * TILE_SIZE;
        // info!("{}", v);
        collider_vertices.push(v);
    }

    info!("spawning colliders now...");

    commands.spawn((
        // Collider::polyline(collider_vertices, None),
        Collider::convex_hull(&collider_vertices).unwrap(),
        ColliderDebugColor(RED.into()),
        SpatialBundle::default(),
    ));

    info!("Spawned Naiive");

    let mut collider_indices = Vec::new();

    for i in 0..collider_vertices.len() - 1 {
        collider_indices.push([i as u32, i as u32 + 1]);
    }
    collider_indices.push([collider_vertices.len() as u32 - 1, 0]);

    commands.spawn((
        Collider::convex_decomposition(&collider_vertices, &collider_indices),
        SpatialBundle::default(),
    ));

    graph.v = collider_vertices.clone();
    graph.e = collider_indices.clone();

    // info!("{:?}", collider_vertices);
    // info!("{:?}", collider_indices);
    info!("Spawned complex");
}

fn draw_gizmos(mut gizmos: Gizmos, graph: Res<Graph>) {
    if graph.v.is_empty() || graph.e.is_empty() {
        return;
    }

    for [i, j] in &graph.e {
        gizmos.line_2d(graph.v[*i as usize], graph.v[*j as usize], PINK);
    }
}
