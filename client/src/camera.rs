use bevy_mod_picking::prelude::*;
use std::f32::consts::PI;

use bevy::prelude::*;
use lib::components::ControlledEntity;
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController},
    LookTransform, LookTransformBundle, Smoother,
};
pub fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle::default()).insert((
        OrbitCameraBundle::new(
            OrbitCameraController {
                mouse_rotate_sensitivity: Vec2::new(0.5, 0.5),
                mouse_translate_sensitivity: Vec2::splat(0.0),
                ..default()
            },
            Vec3::new(0.0, 16.0, -4.5),
            Vec3::new(0., 0., 0.),
            Vec3::Y,
        ),
        RaycastPickCamera::default(),
    ));
}

pub fn camera_follow(
    mut camera_query: Query<&mut LookTransform, (With<Camera>, Without<ControlledEntity>)>,
    player_query: Query<&Transform, (With<ControlledEntity>, Changed<Transform>)>,
) {
    let mut cam_transform = camera_query.single_mut();
    if let Ok(player_transform) = player_query.get_single() {
        cam_transform.eye.x =
            cam_transform.eye.x + (player_transform.translation.x - cam_transform.target.x);
        cam_transform.eye.z =
            cam_transform.eye.z + (player_transform.translation.z - cam_transform.target.z);
        cam_transform.target = player_transform.translation;
    }
}
