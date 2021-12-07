use bevy_ecs_tilemap::prelude::*;
use bevy_rapier2d::physics::{ColliderBundle, ColliderPositionSync};
use bevy_rapier2d::prelude::ColliderShape;
use bevy_rapier2d::render::ColliderDebugRender;
use std::path::Path;
use std::{collections::HashMap, io::BufReader};

use bevy::reflect::TypeUuid;
use bevy::{
    asset::{AssetLoader, AssetPath, BoxedFuture, LoadContext, LoadedAsset},
    prelude::*,
};

#[derive(Default)]
pub struct MapIdToTiledMap(HashMap<u16, Handle<TiledMap>>);

#[derive(Default)]
pub struct TiledMapPlugin;

impl Plugin for TiledMapPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_asset::<TiledMap>()
            .add_asset_loader(TiledLoader)
            .init_resource::<MapIdToTiledMap>()
            .add_system(process_loaded_tile_maps.system().label("load"))
            .add_system(process_collisions_for_loaded_tile_maps.system().after("load"));
    }
}

#[derive(TypeUuid)]
#[uuid = "e51081d0-6168-4881-a1c6-4249b2000d7f"]
pub struct TiledMap {
    pub map: tiled::map::Map,
    pub tilesets: HashMap<u32, Handle<Texture>>,
}

#[derive(Default, Bundle)]
pub struct TiledMapBundle {
    pub tiled_map: Handle<TiledMap>,
    pub map: Map,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

pub struct TiledLoader;

impl AssetLoader for TiledLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let root_dir = load_context.path().parent().unwrap();
            let map = tiled::parse_with_path(BufReader::new(bytes), Path::new("assets/set2.tsx"))?;

            let mut dependencies = Vec::new();
            for tileset in &map.tilesets {
                for i in tileset.first_gid..(tileset.first_gid + tileset.tilecount.unwrap_or(1)) {
                    let tile_path = root_dir.join(tileset.images.first().unwrap().source.as_str());
                    dependencies.push((i, AssetPath::new(tile_path, None)));
                }
            }

            let loaded_asset = LoadedAsset::new(TiledMap {
                map,
                tilesets: dependencies
                    .iter()
                    .map(|dep| {
                        let texture: Handle<Texture> = load_context.get_handle(dep.1.clone());
                        (dep.0, texture)
                    })
                    .collect(),
            });
            load_context.set_default_asset(
                loaded_asset.with_dependencies(dependencies.iter().map(|x| x.1.clone()).collect()),
            );
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["tmx"];
        EXTENSIONS
    }
}

pub fn process_loaded_tile_maps(
    mut commands: Commands,
    mut map_id_to_tiled_map: ResMut<MapIdToTiledMap>,
    mut map_events: EventReader<AssetEvent<TiledMap>>,
    maps: Res<Assets<TiledMap>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(Entity, &Handle<TiledMap>, &mut Map)>,
    new_maps: Query<&Handle<TiledMap>, Added<Handle<TiledMap>>>,
    layer_query: Query<&Layer>,
    chunk_query: Query<&Chunk>,
) {
    let mut changed_maps = Vec::<Handle<TiledMap>>::default();
    for event in map_events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                log::info!("Map added!");
                changed_maps.push(handle.clone());
            }
            AssetEvent::Modified { handle } => {
                log::info!("Map changed!");
                changed_maps.push(handle.clone());
            }
            AssetEvent::Removed { handle } => {
                log::info!("Map removed!");
                // if mesh was modified and removed in the same update, ignore the modification
                // events are ordered so future modification events are ok
                changed_maps = changed_maps
                    .into_iter()
                    .filter(|changed_handle| changed_handle == handle)
                    .collect();
            }
        }
    }

    // If we have new map entities add them to the changed_maps list.
    for new_map_handle in new_maps.iter() {
        changed_maps.push(new_map_handle.clone());
    }

    for changed_map in changed_maps.iter() {
        for (_, map_handle, mut map) in query.iter_mut() {
            // only deal with currently changed map
            if map_handle != changed_map {
                continue;
            }
            map_id_to_tiled_map
                .0
                .insert(map.id, map_handle.clone_weak());
            if let Some(tiled_map) = maps.get(map_handle) {
                // Despawn all tiles/chunks/layers.
                for (layer_id, layer_entity) in map.get_layers() {
                    if let Ok(layer) = layer_query.get(layer_entity) {
                        for x in 0..layer.get_layer_size_in_tiles().0 {
                            for y in 0..layer.get_layer_size_in_tiles().1 {
                                let tile_pos = TilePos(x, y);
                                let chunk_pos = ChunkPos(
                                    tile_pos.0 / layer.settings.chunk_size.0,
                                    tile_pos.1 / layer.settings.chunk_size.1,
                                );
                                if let Some(chunk_entity) = layer.get_chunk(chunk_pos) {
                                    if let Ok(chunk) = chunk_query.get(chunk_entity) {
                                        let chunk_tile_pos = chunk.to_chunk_pos(tile_pos);
                                        if let Some(tile) = chunk.get_tile_entity(chunk_tile_pos) {
                                            commands.entity(tile).despawn_recursive();
                                        }
                                    }

                                    commands.entity(chunk_entity).despawn_recursive();
                                }
                            }
                        }
                    }
                    map.remove_layer(&mut commands, layer_id);
                }

                for tileset in tiled_map.map.tilesets.iter() {
                    // Once materials have been created/added we need to then create the layers.
                    for layer in tiled_map.map.layers.values() {
                        let tile_width = tileset.tile_width as f32;
                        let tile_height = tileset.tile_height as f32;

                        let _tile_space = tileset.spacing as f32; // TODO: re-add tile spacing.. :p

                        let offset_x = layer.offset_x;
                        let offset_y = layer.offset_y;

                        let mut map_settings = LayerSettings::new(
                            MapSize(
                                (tiled_map.map.width as f32 / 64.0).ceil() as u32,
                                (tiled_map.map.height as f32 / 64.0).ceil() as u32,
                            ),
                            ChunkSize(64, 64),
                            TileSize(tile_width, tile_height),
                            TextureSize(
                                tileset.images[0].width as f32,
                                tileset.images[0].height as f32,
                            ), // TODO: support multiple tileset images?
                        );
                        map_settings.grid_size = Vec2::new(
                            tiled_map.map.tile_width as f32,
                            tiled_map.map.tile_height as f32,
                        );
                        map_settings.set_layer_id(layer.layer_index as u16);

                        map_settings.mesh_type = match tiled_map.map.orientation {
                            tiled::map::Orientation::Hexagonal => {
                                TilemapMeshType::Hexagon(HexType::Row) // TODO: Support hex for real.
                            }
                            tiled::map::Orientation::Isometric => {
                                TilemapMeshType::Isometric(IsoType::Diamond)
                            }
                            tiled::map::Orientation::Staggered => {
                                TilemapMeshType::Isometric(IsoType::Staggered)
                            }
                            tiled::map::Orientation::Orthogonal => TilemapMeshType::Square,
                        };

                        let material = materials.add(ColorMaterial::texture(
                            tiled_map.tilesets.get(&tileset.first_gid).unwrap().clone(),
                        ));
                        let layer_entity = LayerBuilder::<TileBundle>::new_batch(
                            &mut commands,
                            map_settings,
                            &mut meshes,
                            material,
                            0u16,
                            layer.layer_index as u16,
                            None,
                            move |mut tile_pos| {
                                if tile_pos.0 >= tiled_map.map.width
                                    || tile_pos.1 >= tiled_map.map.height
                                {
                                    return None;
                                }

                                if tiled_map.map.orientation == tiled::map::Orientation::Orthogonal
                                {
                                    tile_pos.1 = (tiled_map.map.height - 1) as u32 - tile_pos.1;
                                }

                                let x = tile_pos.0 as usize;
                                let y = tile_pos.1 as usize;

                                let map_tile = match &layer.tiles {
                                    tiled::layers::LayerData::Finite(tiles) => &tiles[y][x],
                                    _ => panic!("Infinite maps not supported"),
                                };

                                if map_tile.gid < tileset.first_gid
                                    || map_tile.gid
                                        >= tileset.first_gid + tileset.tilecount.unwrap()
                                {
                                    return None;
                                }

                                let tile_id = map_tile.gid - tileset.first_gid;
                                if let Some(tile_info) = tileset.tiles.get(&tile_id) {
                                    if let Some(object_group) = &tile_info.objectgroup {
                                        for object in object_group.objects.iter() {
                                            match &object.shape {
                                                tiled::objects::ObjectShape::Rect {
                                                    width,
                                                    height,
                                                } => {
                                                    // Add collider
                                                }
                                                tiled::objects::ObjectShape::Ellipse {
                                                    width,
                                                    height,
                                                } => todo!(),
                                                tiled::objects::ObjectShape::Polyline {
                                                    points,
                                                } => todo!(),
                                                tiled::objects::ObjectShape::Polygon { points } => {
                                                    todo!()
                                                }
                                                tiled::objects::ObjectShape::Point(_, _) => todo!(),
                                            }
                                        }
                                    }
                                }

                                let tile = Tile {
                                    texture_index: tile_id as u16,
                                    flip_x: map_tile.flip_h,
                                    flip_y: map_tile.flip_v,
                                    flip_d: map_tile.flip_d,
                                    ..Default::default()
                                };

                                Some(TileBundle {
                                    tile,
                                    ..Default::default()
                                })
                            },
                        );

                        commands.entity(layer_entity).insert(Transform::from_xyz(
                            offset_x,
                            -offset_y,
                            map_settings.layer_id as f32,
                        ));
                        map.add_layer(&mut commands, layer.layer_index as u16, layer_entity);
                    }
                }

                // for object_group in tiled_map.map.object_groups.iter() {
                // dbg!(object_group);
                // }
                //
                // for tileset in tiled_map.map.tilesets.iter() {
                // dbg!(tile);
                // panic!();
                // }
                // }
            }
        }
    }
}

pub fn process_collisions_for_loaded_tile_maps(
    mut commands: Commands,
    map_id_to_tiled_map: Res<MapIdToTiledMap>,
    maps: Res<Assets<TiledMap>>,
    added_tiles_query: Query<(Entity, &TilePos, &TileParent), Added<Tile>>,
    chunks_query: Query<(&Chunk, &GlobalTransform)>,
    layer_query: Query<(&Layer, &GlobalTransform)>,
) {
    let count = added_tiles_query.iter().count();
    if count > 0 {
        dbg!(count);
    } else {
        return
    }

    for (tile_ent, &tile_pos, tile_parent) in added_tiles_query.iter() {
        let x = tile_pos.0 as usize;
        let y = tile_pos.1 as usize;
        let map_id = tile_parent.map_id;
        let tiled_map = maps.get(&map_id_to_tiled_map.0[&map_id]).unwrap();
        let layer_id = tile_parent.layer_id;
        let chunk_ent = tile_parent.chunk;
        let (chunk, chunk_transform) = chunks_query.get(chunk_ent).unwrap();
        let tiled_layer = &tiled_map.map.layers[&(layer_id as u32)];
        // TODO make work with multiple tilesets
        let tileset = tiled_map
            .map
            .tilesets
            .get(0)
            .expect("Expecting one tileset");

        let map_tile = match &tiled_layer.tiles {
            tiled::layers::LayerData::Finite(tiles) => &tiles[y][x],
            _ => panic!("Infinite maps not supported"),
        };

        if map_tile.gid < tileset.first_gid
            || map_tile.gid >= tileset.first_gid + tileset.tilecount.unwrap()
        {
            continue
        }

        let tiled_height = tiled_map.map.height;
        let map_height = tiled_height * tiled_map.map.tile_height;
        let tile_translation = Vec2::new(
            (tile_pos.0 as f32 + 0.5) * chunk.settings.tile_size.0 - (chunk.position.0 * chunk.settings.chunk_size.0) as f32 + chunk_transform.translation.x,
            -1.0 * (tile_pos.1 as f32 + 0.5) * chunk.settings.tile_size.1 + map_height as f32,
        );
        //dbg!(layer_query.iter().next().unwrap().1);
        //dbg!(tile_pos);

        //dbg!(tile_pos, map_tile.gid, tileset.first_gid);
        let tile_id = map_tile.gid - tileset.first_gid;
        if let Some(tile_info) = tileset.tiles.get(&tile_id) {
            if let Some(object_group) = &tile_info.objectgroup {
                for object in object_group.objects.iter() {
                    match &object.shape {
                        &tiled::objects::ObjectShape::Rect { width, height } => {
                            //if tile_pos.0 < 50 && tile_pos.1 < 50 {
                                //dbg!(tile_id, width, height, tile_translation, 1);
                                //dbg!(tile_pos);
                                // Add collider
                                let collider = ColliderBundle {
                                    shape: ColliderShape::cuboid(width/2.0, height/2.0),
                                    position: tile_translation.into(),
                                    ..Default::default()
                                };
                                commands
                                    .spawn_bundle(collider)
                                    .insert(ColliderDebugRender::default())
                                    .insert(ColliderPositionSync::Discrete);
                            //}
                        }
                        tiled::objects::ObjectShape::Ellipse { width, height } => todo!(),
                        tiled::objects::ObjectShape::Polyline { points } => todo!(),
                        tiled::objects::ObjectShape::Polygon { points } => todo!(),
                        tiled::objects::ObjectShape::Point(_, _) => todo!(),
                    };
                }
            }
        }
    }
}
