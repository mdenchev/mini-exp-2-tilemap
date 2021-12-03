#![allow(clippy::too_many_arguments)]
use crate::tiled::*;
use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod camera;
mod tiled;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    camera_bundle.orthographic_projection.scale = 0.5;
    commands.spawn_bundle(camera_bundle);

    let handle: Handle<TiledMap> = asset_server.load("level1.tmx");

    let map_entity = commands.spawn().id();

    commands.entity(map_entity).insert_bundle(TiledMapBundle {
        tiled_map: handle,
        map: Map::new(0u16, map_entity),
        transform: Transform {
            translation: Vec3::new(-200.0, -200.0 , 0.0),
            ..Default::default()
        },
        ..Default::default()
    });
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    App::build()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Tiled map editor example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_plugin(TiledMapPlugin)
        .add_startup_system(startup.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .add_system(camera::movement.system())
        .run();
}
