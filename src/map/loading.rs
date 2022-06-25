use std::path::PathBuf;

use bevy::asset::{AssetEvent, AssetLoader, LoadContext, LoadedAsset};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::utils::BoxedFuture;
use serde::Deserialize;

use crate::map::{Direction, TileBundle, WallBundle};
use crate::material::UnlitMaterial;

#[derive(Debug, Deserialize)]
struct MapTile {
    pos: (i32, i32),
}

#[derive(Debug, Deserialize)]
struct MapWall {
    pos: (i32, i32),
    direction: Direction,
}

/// Map defined as an asset
#[derive(Debug, TypeUuid)]
#[uuid = "b6f944ca-0812-42d0-8466-84c39ed401a1"]
pub struct Map {
    name: String,
    tile_texture: Handle<Image>,
    wall_texture: Handle<Image>,
    tiles: Vec<MapTile>,
    walls: Vec<MapWall>,
}

#[derive(Debug, Deserialize)]
struct MapFile {
    name: String,
    tile_texture: PathBuf,
    wall_texture: PathBuf,
    tiles: Vec<MapTile>,
    walls: Vec<MapWall>,
}

/// Asset loader which defines how to load our map file from disk
#[derive(Default)]
pub(super) struct MapLoader;

impl AssetLoader for MapLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move {
            let map_file = serde_yaml::from_slice::<MapFile>(bytes)?;
            let tile_texture = load_context
                .get_handle(map_file.tile_texture.to_str().unwrap());
            let wall_texture = load_context
                .get_handle(map_file.wall_texture.to_str().unwrap());
            let map = Map {
                name: map_file.name,
                tile_texture,
                wall_texture,
                tiles: map_file.tiles,
                walls: map_file.walls,
            };
            let asset = LoadedAsset::new(map)
                .with_dependency(map_file.tile_texture.into())
                .with_dependency(map_file.wall_texture.into());
            load_context.set_default_asset(asset);
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["map"]
    }
}

#[derive(Debug)]
pub enum MapEvent {
    Update { handle: Handle<Map> },
}

pub fn detect_changes(
    active_map: Res<super::ActiveMap>,
    maps: Res<Assets<Map>>,
    mut asset_events: EventReader<AssetEvent<Map>>,
    mut map_events: EventWriter<MapEvent>,
) {
    for event in asset_events.iter() {
        let handle = match event {
            AssetEvent::Created { handle } => {
                if let Some(map) = maps.get(handle) {
                    info!("Loaded map: \"{}\"", map.name);
                }
                handle
            }
            AssetEvent::Modified { handle } => {
                if let Some(map) = maps.get(handle) {
                    debug!("Updated map: \"{}\"", map.name);
                }
                handle
            }
            AssetEvent::Removed { handle } => {
                if handle == &active_map.map {
                    panic!("Unloaded active map");
                }
                return;
            }
        };

        if handle == &active_map.map {
            map_events.send(MapEvent::Update {
                handle: handle.clone(),
            });
        }
    }
}

pub fn update_map(
    mut commands: Commands,
    map_query: Query<Entity, With<Handle<Map>>>,
    maps: Res<Assets<Map>>,
    mut map_events: EventReader<MapEvent>,
    mut materials: ResMut<Assets<UnlitMaterial>>,
) {
    for event in map_events.iter() {
        match event {
            MapEvent::Update { handle } => {
                let map = maps
                    .get(handle)
                    .expect("Map was created but is not in assets");

                info!("Updating map: \"{}\"", map.name);

                let entity = map_query.get_single().unwrap_or_else(|_| {
                    commands
                        .spawn()
                        .insert(Transform::default())
                        .insert(GlobalTransform::default())
                        .id()
                });

                let tile_material =
                    materials.add(UnlitMaterial::new(map.tile_texture.clone()));

                let wall_material =
                    materials.add(UnlitMaterial::new(map.wall_texture.clone()));

                commands.entity(entity).despawn_descendants();
                commands
                    .entity(entity)
                    .insert(handle.clone())
                    .with_children(|parent| {
                        for tile in map.tiles.iter() {
                            parent.spawn_bundle(TileBundle::new(
                                tile.pos.into(),
                                tile_material.clone(),
                            ));
                        }

                        for wall in map.walls.iter() {
                            parent.spawn_bundle(WallBundle::new(
                                wall.pos.into(),
                                wall.direction,
                                wall_material.clone(),
                            ));
                        }
                    });
            }
        }
    }
}
