#![allow(clippy::too_many_arguments, clippy::type_complexity)]

use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::window::{PresentMode, Window, WindowMode};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

const TILE_SIZE: f32 = 16.0;

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
        .add_systems(Startup, setup)
        .add_systems(Update, (print_grid_coords, place_colliders))
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

fn print_grid_coords(q_grid_coords: Query<&GridCoords, Added<IntGridCell>>) {
    for grid_coords in &q_grid_coords {
        info!("{:?}", grid_coords);
    }
}

fn place_colliders(mut commands: Commands, q_grid_coords: Query<&GridCoords, Added<IntGridCell>>) {
    let collider = Collider::convex_hull(&[
        Vec2::X * TILE_SIZE,
        Vec2::Y * TILE_SIZE,
        Vec2::NEG_Y * TILE_SIZE,
        Vec2::NEG_X * TILE_SIZE,
    ])
    .unwrap();
    for grid_coords in &q_grid_coords {
        let v = Vec2::new(grid_coords.x as f32, grid_coords.y as f32) * TILE_SIZE;
        commands.spawn((
            collider.clone(),
            SpatialBundle::from_transform(Transform::from_translation(v.extend(0.0))),
        ));
    }
}
