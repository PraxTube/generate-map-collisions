#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use std::time::Duration;

use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::time::common_conditions::once_after_delay;
use bevy::utils::HashSet;
use bevy::window::{PresentMode, Window, WindowMode};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

const TILE_SIZE: f32 = 16.0;

#[derive(Resource)]
struct Grid {
    size: IVec2,
    positions: Vec<IVec2>,
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
        .add_systems(Startup, setup)
        .add_systems(Update, (add_cells,))
        .add_systems(
            Update,
            spawn_colliders.run_if(once_after_delay(Duration::from_secs_f32(0.5))),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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

fn index_to_vertices(index: u8) -> Vec<UVec2> {
    match index {
        0 => Vec::new(),
        1 => vec![UVec2::X, UVec2::Y],
        2 => vec![UVec2::new(2, 1), UVec2::X],
        3 => vec![UVec2::new(2, 1), UVec2::Y],
        4 => vec![UVec2::new(1, 2), UVec2::new(2, 1)],
        5 => vec![UVec2::new(1, 2), UVec2::Y, UVec2::X, UVec2::new(2, 1)],
        6 => vec![UVec2::new(1, 2), UVec2::X],
        7 => vec![UVec2::new(1, 2), UVec2::Y],
        8 => vec![UVec2::Y, UVec2::new(1, 2)],
        9 => vec![UVec2::X, UVec2::new(1, 2)],
        10 => vec![UVec2::new(2, 1), UVec2::new(1, 2), UVec2::Y, UVec2::X],
        11 => vec![UVec2::new(2, 1), UVec2::new(1, 2)],
        12 => vec![UVec2::Y, UVec2::new(2, 1)],
        13 => vec![UVec2::X, UVec2::new(2, 1)],
        14 => vec![UVec2::Y, UVec2::X],
        15 => Vec::new(),
        _ => {
            warn!("should never happen");
            Vec::new()
        }
    }
}

fn spawn_colliders(mut commands: Commands, grid: Res<Grid>) {
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
                info!("{}", index_matrix[i][j]);
            }
        }
    }

    let mut vertices = HashSet::new();
    let mut indices = HashSet::new();

    for i in 0..index_matrix.len() {
        for j in 0..index_matrix.len() {
            for raw_vertex in index_to_vertices(index_matrix[i][j]) {
                let v = raw_vertex + 2 * UVec2::new(i as u32, j as u32);
                vertices.insert(v);
                indices.insert(indices.len() + 1);
            }
        }
    }

    let mut collider_vertices = Vec::new();
    // let mut collider_indices = Vec::new();

    for uvert in vertices {
        let v = Vec2::new(uvert.x as f32, uvert.y as f32) / 2.0 * TILE_SIZE;
        info!("{}", v);
        collider_vertices.push(v);
    }

    commands.spawn((
        // Collider::polyline(collider_vertices, None),
        Collider::convex_hull(&collider_vertices).unwrap(),
        SpatialBundle::default(),
    ));

    // for index in coll
    //
    // let collider = Collider::convex_decomposition(&collider_vertices, indices);

    // for pos in &grid.positions {
    //     let collider = Collider::cuboid(8.0, 8.0);
    //     let translation = Vec3::new(pos.x as f32, pos.y as f32, 0.0) * TILE_SIZE;
    //     commands.spawn((
    //         collider,
    //         SpatialBundle::from_transform(Transform::from_translation(translation)),
    //     ));
    // }
}
