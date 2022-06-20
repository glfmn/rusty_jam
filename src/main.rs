use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::WorldInspectorPlugin;

mod camera;
mod map;
mod material;

use map::MapPlugin;
use material::{RenderPlugin, UnlitMaterial};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(watch_for_changes)
        .add_plugin(RenderPlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(camera::CameraPlugin)
        .add_plugin(MapPlugin)
        // Systems that create Egui widgets should be run during the
        // `CoreStage::Update` stage, or after the `EguiSystem::BeginFrame`
        // system (which belongs to the `CoreStage::PreUpdate` stage).
        .add_startup_system(test_map)
        .run();
}

fn test_map(
    mut commands: Commands,
    mut materials: ResMut<Assets<UnlitMaterial>>,
) {
    use map::{Direction, Location, TileBundle, WallBundle};

    // Add handle for blank material
    let material = materials.add(UnlitMaterial::default());

    // Spawn the ground
    commands.spawn_bundle(TileBundle::new(
        Location { x: 0, y: 0 },
        material.clone(),
    ));
    commands.spawn_bundle(TileBundle::new(
        Location { x: 1, y: 0 },
        material.clone(),
    ));
    commands.spawn_bundle(TileBundle::new(
        Location { x: 0, y: 1 },
        material.clone(),
    ));
    commands.spawn_bundle(TileBundle::new(
        Location { x: 0, y: -1 },
        material.clone(),
    ));
    commands.spawn_bundle(TileBundle::new(
        Location { x: -1, y: 0 },
        material.clone(),
    ));

    // Spawn walls
    commands.spawn_bundle(WallBundle::new(
        Location { x: -1, y: 0 },
        Direction::NegativeX,
        material.clone(),
    ));
    commands.spawn_bundle(WallBundle::new(
        Location { x: 1, y: 0 },
        Direction::PositiveX,
        material.clone(),
    ));
    commands.spawn_bundle(WallBundle::new(
        Location { x: 0, y: 1 },
        Direction::PositiveY,
        material.clone(),
    ));
    commands.spawn_bundle(WallBundle::new(
        Location { x: 0, y: -1 },
        Direction::NegativeY,
        material.clone(),
    ));
}

fn watch_for_changes(asset_server: ResMut<AssetServer>) {
    info!("Watching for changes");
    asset_server.watch_for_changes().unwrap();
}
