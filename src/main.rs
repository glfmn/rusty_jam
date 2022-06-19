use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::WorldInspectorPlugin;

mod camera;
mod map;
mod material;

use map::{Location, MapPlugin, TileBundle};
use material::{RenderPlugin, UnlitMaterial};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(RenderPlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(camera::CameraPlugin)
        .add_plugin(MapPlugin)
        // Systems that create Egui widgets should be run during the
        // `CoreStage::Update` stage, or after the `EguiSystem::BeginFrame`
        // system (which belongs to the `CoreStage::PreUpdate` stage).
        .add_startup_system(test_plane)
        .run();
}

fn test_plane(
    mut commands: Commands,
    mut materials: ResMut<Assets<UnlitMaterial>>,
) {
    // Spawn the ground
    let material = materials.add(UnlitMaterial::default());
    commands.spawn_bundle(TileBundle::new(Location { x: 0, y: 0 }, material));
}
