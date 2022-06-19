use bevy::prelude::*;
use bevy::render::camera::*;
use bevy_inspector_egui::{Inspectable, RegisterInspectable};
use iyes_loopless::prelude::*;

pub struct CameraPlugin;

/// Label applied to camera system
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
pub struct CameraSystem;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_inspectable::<ControllerBasis>()
            .register_type::<ControllerBasis>()
            .register_inspectable::<YawPitchControls>()
            .register_type::<YawPitchControls>()
            .register_inspectable::<IsometricCamera>()
            .register_type::<IsometricCamera>()
            .add_startup_system(setup_camera.label(CameraSystem))
            .add_system_set(
                ConditionSet::new()
                    .label(CameraSystem)
                    .with_system(YawPitchControls::system)
                    .into(),
            );
    }
}

#[derive(Bundle)]
struct IsometricCameraBundle {
    #[bundle]
    camera: OrthographicCameraBundle<Camera3d>,
    controller_basis: ControllerBasis,
    controls: YawPitchControls,
    marker: IsometricCamera,
}

impl IsometricCameraBundle {
    fn new() -> Self {
        Self {
            camera: OrthographicCameraBundle::new_3d(),
            controller_basis: ControllerBasis::default(),
            controls: YawPitchControls::default(),
            marker: IsometricCamera,
        }
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(IsometricCameraBundle::new());
}

#[derive(Component, Inspectable, Reflect)]
pub struct IsometricCamera;

/// Define the coordinate system a controller will use
#[derive(Component, Debug, Clone, Reflect, Inspectable)]
pub struct ControllerBasis {
    /// Up basis vector
    pub up: Vec3,
    /// Forward basis vector
    pub forward: Vec3,
}

impl Default for ControllerBasis {
    fn default() -> Self {
        Self {
            up: Vec3::Y,
            forward: Vec3::Z,
        }
    }
}

/// Set the entity's transform based on yaw, pitch, and distance from focus
#[derive(Component, Debug, Clone, Reflect, Inspectable)]
pub struct YawPitchControls {
    /// The "focus point" to orient around.
    pub focus: Vec3,
    /// The angle the camera is rotated around compared to the focus [-PI, PI]
    pub yaw: f32,
    /// Angle the camera looks down from (in radians)
    pub pitch: f32,
    /// Distance from the focus
    pub dist: f32,
}

impl Default for YawPitchControls {
    /// Spawn camera in ideal spot for isometric perspective
    fn default() -> Self {
        YawPitchControls {
            focus: Vec3::ZERO,
            pitch: f32::to_radians(45.0),
            yaw: f32::to_radians(45.0),
            dist: 1.0,
        }
    }
}

impl YawPitchControls {
    /// Update transform using yaw and pitch when controls or basis change
    fn system(
        mut query: Query<
            (&YawPitchControls, &ControllerBasis, &mut Transform),
            Or<(Changed<YawPitchControls>, Changed<ControllerBasis>)>,
        >,
    ) {
        for (controls, basis, mut transform) in query.iter_mut() {
            *transform = controls.transform(basis);
        }
    }

    /// Resolve the controls into the transform they represent
    pub fn transform(&self, basis: &ControllerBasis) -> Transform {
        // Determine the position relative to our focus based on the local basis
        // and then generate the transform looking back at the center.
        let basis = self.local_basis(basis);
        Transform::from_translation(self.focus + basis.forward * self.dist)
            .looking_at(self.focus, basis.up)
    }

    /// Transform the global space controller basis by yaw and pitch
    pub fn local_basis(&self, basis: &ControllerBasis) -> ControllerBasis {
        // Transform both of the basis vectors
        let orientation = self.rotator(basis);
        let forward = orientation * basis.forward;
        let up = orientation * basis.up;
        ControllerBasis { up, forward }
    }

    /// Rotation which maps global space to local space
    pub fn rotator(&self, basis: &ControllerBasis) -> Quat {
        // Create a rotation that will pitch up or down the desired amount
        let pitch_axis = basis.up.cross(basis.forward).normalize();
        let pitch = Quat::from_axis_angle(pitch_axis, -self.pitch);

        // Compose with yaw rotation
        self.yaw(basis) * pitch
    }

    /// Create a rotation that represents yaw around the global basis up
    pub fn yaw(&self, basis: &ControllerBasis) -> Quat {
        Quat::from_axis_angle(basis.up, self.yaw)
    }
}
