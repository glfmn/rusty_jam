use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::WorldInspectorPlugin;

mod camera;
mod map;
mod material;

use map::MapPlugin;
use material::RenderPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(watch_for_changes)
        .add_plugin(RenderPlugin)
        .add_plugin(EguiPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(camera::CameraPlugin)
        .add_plugin(MapPlugin)
        .run();
}

fn watch_for_changes(asset_server: ResMut<AssetServer>) {
    info!("Watching for changes");
    asset_server.watch_for_changes().unwrap();
}
