use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_inspector_egui::WorldInspectorPlugin;

mod camera;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(camera::CameraPlugin)
        // Systems that create Egui widgets should be run during the
        // `CoreStage::Update` stage, or after the `EguiSystem::BeginFrame`
        // system (which belongs to the `CoreStage::PreUpdate` stage).
        .add_startup_system(test_plane)
        .run();
}

fn test_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Spawn the ground
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 1.0 })),
        material: materials.add(StandardMaterial {
            base_color: Color::WHITE,
            ..Default::default()
        }),
        ..Default::default()
    });
}
