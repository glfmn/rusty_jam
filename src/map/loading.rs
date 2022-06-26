use std::collections::HashMap;
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
    id: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct MapWall {
    pos: (i32, i32),
    direction: Direction,
    id: Option<u32>,
}

#[derive(Debug)]
struct SpriteSheet {
    /// Dimension of a sprite from the sprite sheet (in pixels)
    grid_dimensions: (u32, u32),
    sprite_sheet: Handle<Image>,
}

impl SpriteSheet {
    /// Get the (width, height) of the image in pixels
    fn dimensions(&self, images: &Assets<Image>) -> (u32, u32) {
        images
            .get(&self.sprite_sheet)
            .map(|i| i.size())
            .map(|size| (size.x as u32, size.y as u32))
            .unwrap_or(self.grid_dimensions)
    }

    fn material_allocator(&self, images: &Assets<Image>) -> MaterialAllocator {
        MaterialAllocator {
            texture: self.sprite_sheet.clone(),
            grid_dimensions: self.grid_dimensions,
            dimensions: self.dimensions(images),
            cache: HashMap::new(),
        }
    }
}

struct MaterialAllocator {
    texture: Handle<Image>,
    dimensions: (u32, u32),
    grid_dimensions: (u32, u32),
    cache: HashMap<u32, Handle<UnlitMaterial>>,
}

impl MaterialAllocator {
    fn get_material(
        &mut self,
        index: u32,
        materials: &mut Assets<UnlitMaterial>,
    ) -> Handle<UnlitMaterial> {
        let rect = &self.index(index);
        self.cache
            .entry(index)
            .or_insert_with(|| {
                materials.add(UnlitMaterial::new(self.texture.clone(), *rect))
            })
            .clone()
    }

    /// Extract the rect from the provided index
    ///
    /// Rects are aligned to the grid defined by `grid_dimensions` and go from
    /// left-to-right, top-to-bottom (low to high, first in x then in y).
    fn index(&self, index: u32) -> Rect<f32> {
        let (width, height) = self.dimensions;
        // First get the number of rows and columns
        let (rows, cols) = (
            width / self.grid_dimensions.0,
            height / self.grid_dimensions.1,
        );

        // Convert linear index into row and column of the sprite sheet
        // 0 (0,0) 1 (1,0) 2 (2,0) 3 (3,0) 4 (4,0)
        // 5 (0,1) 6 (1,1) 7 (2,1) 8 (3,1) 9 (4,1)
        let (x, y) = ((index % rows) as f32, (index / rows) as f32);

        // Width and height of a single tile in UV coordinates
        let (w, h) = (
            self.grid_dimensions.0 as f32 / width as f32,
            self.grid_dimensions.1 as f32 / height as f32,
        );

        // Create a rectangle spanning 1 grid cell in UV coordinates
        // It is possible for the values to oustide [0, 1], let the
        // shader/pipeline/sampler handle this.
        Rect {
            // Min
            top: y * h,
            left: x * w,
            // Max
            bottom: y * h + h,
            right: x * w + w,
        }
    }
}

/// Map defined as an asset
#[derive(Debug, TypeUuid)]
#[uuid = "b6f944ca-0812-42d0-8466-84c39ed401a1"]
pub struct Map {
    /// Name of the map
    name: String,
    /// Texture sheet for floor tiles
    tile_sprites: SpriteSheet,
    /// Texture sheet for wall tiles
    wall_sprites: SpriteSheet,
    /// List of all floor tiles
    tiles: Vec<MapTile>,
    /// List of all wall tiles
    walls: Vec<MapWall>,
}

#[derive(Debug, Deserialize)]
struct MapFile {
    name: String,
    tile_sprites: SpriteSheetFile,
    wall_sprites: SpriteSheetFile,
    tiles: Vec<MapTile>,
    walls: Vec<MapWall>,
}

#[derive(Debug, Deserialize)]
struct SpriteSheetFile {
    sprite_sheet: PathBuf,
    grid_dimensions: (u32, u32),
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
            // First deserialize the contents of our file
            let map_file = serde_yaml::from_slice::<MapFile>(bytes)?;

            // We get the path to a sprite texture, but we want a handle to the
            // image directly on our asset so we load the asset here first.
            // This is cleaner and allows everything to be loaded much sooner.
            let tile_sprites = SpriteSheet {
                sprite_sheet: load_context.get_handle(
                    map_file.tile_sprites.sprite_sheet.to_str().unwrap(),
                ),
                grid_dimensions: map_file.tile_sprites.grid_dimensions,
            };
            let wall_sprites = SpriteSheet {
                sprite_sheet: load_context.get_handle(
                    map_file.wall_sprites.sprite_sheet.to_str().unwrap(),
                ),
                grid_dimensions: map_file.wall_sprites.grid_dimensions,
            };

            // Now we can create the map, copying the rest of the fields
            let map = Map {
                name: map_file.name,
                tile_sprites,
                wall_sprites,
                tiles: map_file.tiles,
                walls: map_file.walls,
            };

            // Finally, register the dependencies and produce the loaded asset
            let asset = LoadedAsset::new(map)
                .with_dependency(map_file.tile_sprites.sprite_sheet.into())
                .with_dependency(map_file.wall_sprites.sprite_sheet.into());
            load_context.set_default_asset(asset);

            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["map", ".map.yaml"]
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
    images: Res<Assets<Image>>,
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

                let mut tile_materials =
                    map.tile_sprites.material_allocator(&*images);
                let mut wall_materials =
                    map.wall_sprites.material_allocator(&*images);

                commands.entity(entity).despawn_descendants();
                commands
                    .entity(entity)
                    .insert(handle.clone())
                    .with_children(|parent| {
                        for tile in map.tiles.iter() {
                            parent.spawn_bundle(TileBundle::new(
                                tile.pos.into(),
                                tile_materials.get_material(
                                    tile.id.unwrap_or(0),
                                    &mut *materials,
                                ),
                            ));
                        }

                        for wall in map.walls.iter() {
                            parent.spawn_bundle(WallBundle::new(
                                wall.pos.into(),
                                wall.direction,
                                wall_materials.get_material(
                                    wall.id.unwrap_or(0),
                                    &mut *materials,
                                ),
                            ));
                        }
                    });
            }
        }
    }
}
