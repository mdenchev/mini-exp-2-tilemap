#![allow(clippy::too_many_arguments)]
use std::time::Duration;

use crate::tiled::*;
use benimator::{AnimationPlugin, SpriteSheetAnimation};
use bevy::{prelude::*, utils::HashMap};
use bevy_ecs_tilemap::prelude::*;
use bevy_rapier2d::{
    physics::{NoUserData, RapierPhysicsPlugin, ColliderBundle, ColliderPositionSync, RigidBodyBundle},
    render::{RapierRenderPlugin, ColliderDebugRender}, prelude::{ColliderShape, RigidBodyMassPropsFlags},
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

fn startup(mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut textures: ResMut<Assets<TextureAtlas>>,
    mut animations: ResMut<Assets<SpriteSheetAnimation>>,
) {
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
        shape: ColliderShape::cuboid(8.0, 8.0),
        position: Vec3::new(300.0, 50.0, 5.0).into(),
        ..Default::default()
    };
    let animation_handle = animations.add(SpriteSheetAnimation::from_range(
        0..=4,
        Duration::from_millis(400),
    ));

    commands
        .spawn()
        .insert_bundle(collider)
        .insert(ColliderDebugRender::with_id(2))
        .insert(ColliderPositionSync::Discrete)
        .insert_bundle(
            RigidBodyBundle {
            mass_properties: (RigidBodyMassPropsFlags::ROTATION_LOCKED).into(),
            ..Default::default()
        })
        .insert(Player(300.0))
        .with_children(|parent| {
            parent
                .spawn_bundle(SpriteSheetBundle {
                    texture_atlas: textures.add(TextureAtlas::from_grid(asset_server.load("cow-sheet.png"), Vec2::new(16.0, 16.0), 20, 1)),
                    transform: Transform::from_scale(Vec3::splat(SCALE)),
                    ..Default::default()
                })
                // Insert the asset handle of the animation
                .insert(animation_handle)
                // Start the animation immediately. Remove this component in order to pause the animation.
                .insert(benimator::Play);

        });


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
    let count = render_tags.iter_mut().count();
    if count > 0 {
        dbg!(count);
    }
    for (ent, mut trans) in render_tags.iter_mut() {
        trans.translation.z = 0.3;
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
        .add_plugin(AnimationPlugin)
        .add_plugin(TilemapPlugin)
        .add_plugin(TiledMapPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        //.add_plugin(RapierRenderPlugin)
        .add_startup_system(startup.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .add_system(camera::movement.system())
        .add_system_to_stage(CoreStage::PostUpdate, camera::char_input.system())
        .add_system(lift_debug.system())
        .run();
}
