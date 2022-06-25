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
    texture: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct MapWall {
    pos: (i32, i32),
    direction: Direction,
    texture: Option<PathBuf>,
}

/// Map defined as an asset
#[derive(Debug, Deserialize, TypeUuid)]
#[uuid = "b6f944ca-0812-42d0-8466-84c39ed401a1"]
pub struct Map {
    name: String,
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
            let custom_asset = serde_yaml::from_slice::<Map>(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(custom_asset));
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
    asset_server: Res<AssetServer>,
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

                let default_material = materials.add(UnlitMaterial::default());

                commands.entity(entity).despawn_descendants();
                commands
                    .entity(entity)
                    .insert(handle.clone())
                    .with_children(|parent| {
                        for tile in map.tiles.iter() {
                            let material = tile
                                .texture
                                .as_ref()
                                .map(|path| asset_server.load(path.as_path()))
                                .map(|handle| {
                                    materials.add(UnlitMaterial::new(handle))
                                })
                                .unwrap_or_else(|| default_material.clone());

                            parent.spawn_bundle(TileBundle::new(
                                tile.pos.into(),
                                material,
                            ));
                        }

                        for wall in map.walls.iter() {
                            let material = wall
                                .texture
                                .as_ref()
                                .map(|path| asset_server.load(path.as_path()))
                                .map(|handle| {
                                    materials.add(UnlitMaterial::new(handle))
                                })
                                .unwrap_or_else(|| default_material.clone());

                            parent.spawn_bundle(WallBundle::new(
                                wall.pos.into(),
                                wall.direction,
                                material,
                            ));
                        }
                    });
            }
        }
    }
}
