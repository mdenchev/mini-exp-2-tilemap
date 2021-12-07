#![allow(clippy::too_many_arguments)]
use crate::tiled::*;
use bevy::{prelude::*, utils::HashMap};
use bevy_ecs_tilemap::prelude::*;
use bevy_rapier2d::{
    physics::{NoUserData, RapierPhysicsPlugin, ColliderBundle, ColliderPositionSync, RigidBodyBundle},
    render::{RapierRenderPlugin, ColliderDebugRender}, prelude::ColliderShape,
};
//use bevy_spicy_aseprite::{AsepriteBundle, AsepriteAnimation, AsepritePlugin};

mod camera;
mod tiled;

//mod sprites {
//    use bevy_spicy_aseprite::aseprite;
//
//    aseprite!(pub Player, "assets/player.ase");
//    aseprite!(pub Cow, "assets/cow.ase");
//}

pub struct Player(f32);
const SCALE: f32 = 2.0;

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
            translation: Vec3::new(0.0, 0.0, 0.0),
            ..Default::default()
        },
        ..Default::default()
    });




    let collider = ColliderBundle {
        shape: ColliderShape::cuboid(16.0, 16.0),
        position: Vec3::new(300.0, 50.0, 5.0).into(),
        ..Default::default()
    };
    commands
        .spawn()
        .insert_bundle(collider)
        .insert(ColliderDebugRender::with_id(1))
        .insert(ColliderPositionSync::Discrete)
        .insert_bundle(RigidBodyBundle::default())
        .insert(Player(300.0));

    //commands
    //    .spawn_bundle(AsepriteBundle {
    //        aseprite: sprites::Cow::sprite(),
    //        animation: AsepriteAnimation::from(sprites::Cow::tags::SLEEP),
    //        transform: Transform {
    //            scale: Vec3::splat(SCALE),
    //            translation: Vec3::new(-300., -200., 0.),
    //            ..Default::default()
    //        },
    //        ..Default::default()
    //    })
    //    .insert_bundle(collider);
}

fn lift_debug(mut commands: Commands, mut render_tags: Query<(Entity, &mut Transform), (Added<ColliderDebugRender>)>) {
    for (ent, mut trans) in render_tags.iter_mut() {
        trans.translation.z = 3.0;
    }
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
        //.add_plugin(AsepritePlugin)
        .add_plugin(TilemapPlugin)
        .add_plugin(TiledMapPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierRenderPlugin)
        .add_startup_system(startup.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .add_system(camera::movement.system())
        .add_system(camera::char_input.system())
        .add_system(lift_debug.system())
        .run();
}
