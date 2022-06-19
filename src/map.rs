use crate::material::{UnlitMaterial, UnlitMaterialBundle};
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use iyes_loopless::prelude::*;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.register_inspectable::<Location>()
            .add_system_set(
                ConditionSet::new().with_system(location_controller).into(),
            )
            .init_resource::<TileMesh>();
    }
}

const TILE_MESH_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Mesh::TYPE_UUID, 0x857e0e2d7312f367);

const TILE_SIZE: f32 = 0.2;

pub struct TileMesh {
    /// This probably won't be used, but we need at least one strong handle
    /// to the mesh to persist in order to prevent it from being unexpectedly
    /// dropped.
    #[allow(unused)]
    pub handle: Handle<Mesh>,
}

impl FromWorld for TileMesh {
    fn from_world(world: &mut World) -> Self {
        Self {
            handle: world.resource_mut::<Assets<Mesh>>().set(
                TILE_MESH_HANDLE.typed::<Mesh>(),
                Mesh::from(shape::Plane { size: TILE_SIZE }),
            ),
        }
    }
}

#[derive(Bundle)]
pub struct TileBundle {
    pub grid_pos: Location,
    #[bundle]
    pub render: UnlitMaterialBundle,
}

impl TileBundle {
    /// Create a tile at the given location with the provided material
    pub fn new(grid_pos: Location, material: Handle<UnlitMaterial>) -> Self {
        Self {
            grid_pos,
            render: UnlitMaterialBundle {
                material,
                transform: Transform::from_translation(grid_pos.into()),
                global_transform: GlobalTransform::from_translation(
                    grid_pos.into(),
                ),
                mesh: TILE_MESH_HANDLE.typed::<Mesh>(),
                ..Default::default()
            },
        }
    }
}

#[derive(Component, Inspectable, PartialEq, Eq, Hash, Copy, Clone)]
pub struct Location {
    pub x: i32,
    pub y: i32,
}

impl From<Location> for Vec3 {
    fn from(l: Location) -> Self {
        // Center of the tile should be at the location
        let offset = TILE_SIZE * 0.5;
        Self::new(
            l.x as f32 * TILE_SIZE + offset,
            0.0,
            l.y as f32 * TILE_SIZE + offset,
        )
    }
}

/// When location is changed, change the transform to match
fn location_controller(
    mut query: Query<(&Location, &mut Transform), Changed<Location>>,
) {
    for (loc, mut transform) in query.iter_mut() {
        transform.translation = (*loc).into();
    }
}
