use crate::material::{UnlitMaterial, UnlitMaterialBundle};
use bevy::reflect::TypeUuid;
use bevy::{prelude::*, render::mesh::Indices};
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use iyes_loopless::prelude::*;

/// Square tile side length
pub const TILE_SIZE: f32 = 0.33;
pub const WALL_HEIGHT: f32 = 0.45;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.register_inspectable::<Location>()
            .register_inspectable::<Direction>()
            .add_system_set(
                ConditionSet::new()
                    .with_system(location_controller)
                    .with_system(direction_controller)
                    .into(),
            )
            .init_resource::<TileMesh>()
            .init_resource::<WallMesh>();
    }
}

#[derive(Component, Inspectable, PartialEq, Eq, Hash, Copy, Clone)]
pub struct Location {
    pub x: i32,
    pub y: i32,
}

impl From<Location> for Vec3 {
    fn from(l: Location) -> Self {
        Self::new(l.x as f32 * TILE_SIZE, 0.0, l.y as f32 * TILE_SIZE)
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

/// Direction on the (x,y) plane
#[derive(Copy, Clone, Component, Inspectable)]
pub enum Direction {
    PositiveX,
    NegativeY,
    NegativeX,
    PositiveY,
}

impl From<Direction> for Quat {
    fn from(dir: Direction) -> Self {
        let angle: f32 = match dir {
            Direction::PositiveX => 0.0,
            Direction::NegativeY => 90.0,
            Direction::NegativeX => 180.0,
            Direction::PositiveY => 270.0,
        };
        Self::from_axis_angle(Vec3::Y, angle.to_radians())
    }
}

fn direction_controller(
    mut query: Query<(&Direction, &mut Transform), Changed<Direction>>,
) {
    for (dir, mut transform) in query.iter_mut() {
        transform.rotation = (*dir).into();
    }
}

const TILE_MESH_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Mesh::TYPE_UUID, 0x857e0e2d7312f367);

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

/// Simple vertical wall
#[derive(Bundle)]
pub struct WallBundle {
    location: Location,
    direction: Direction,
    #[bundle]
    render: UnlitMaterialBundle,
}

impl WallBundle {
    pub fn new(
        location: Location,
        direction: Direction,
        material: Handle<UnlitMaterial>,
    ) -> Self {
        let grid_pos: Vec3 = location.into();
        Self {
            location,
            direction,
            render: UnlitMaterialBundle {
                material,
                transform: Transform::from_translation(grid_pos.into()),
                global_transform: GlobalTransform::from_translation(
                    grid_pos.into(),
                ),
                mesh: WALL_MESH_HANDLE.typed::<Mesh>(),
                ..Default::default()
            },
        }
    }
}

const WALL_MESH_HANDLE: HandleUntyped =
    HandleUntyped::weak_from_u64(Mesh::TYPE_UUID, 0x903dba46d6b08058);

pub struct WallMesh {
    /// This probably won't be used, but we need at least one strong handle
    /// to the mesh to persist in order to prevent it from being unexpectedly
    /// dropped.
    #[allow(unused)]
    pub handle: Handle<Mesh>,
}

impl FromWorld for WallMesh {
    fn from_world(world: &mut World) -> Self {
        // The local coordinates (model space) are relative to the center of the
        // tile on the ground plane.  Tiles are square and all share the same
        // width.  The ground plane is the (x,z) plane in bevy (ugh).

        // Since Direction::PositiveX is rotation zero, the plane wall plane
        // should be a subset of the (y,z) plane
        let x = TILE_SIZE * 0.5;
        let z = x;

        let mut mesh =
            Mesh::new(bevy::render::mesh::PrimitiveTopology::TriangleStrip);

        // The vertical extent (y coordinate) is from 0 to WALL_HEIGHT
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![
                [x, 0.0, z],
                [x, WALL_HEIGHT, z],
                [x, 0.0, -z],
                [x, WALL_HEIGHT, -z],
            ],
        );

        mesh.insert_attribute(
            Mesh::ATTRIBUTE_UV_0,
            vec![[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]],
        );

        let normal = [-1.0, 0.0, 0.0];
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            vec![normal, normal, normal, normal],
        );

        mesh.set_indices(Some(Indices::U16(vec![0, 1, 2, 3])));
        Self {
            handle: world
                .resource_mut::<Assets<Mesh>>()
                .set(WALL_MESH_HANDLE.typed::<Mesh>(), mesh),
        }
    }
}
