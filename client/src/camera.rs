use std::f32::consts::{PI, TAU};
use bevy_mod_picking::PickingCameraBundle;

use bevy::prelude::*;
use lib::components::ControlledEntity;
use smooth_bevy_cameras::{
    controllers::{
        orbit::{OrbitCameraBundle, OrbitCameraController},
        unreal::{UnrealCameraBundle, UnrealCameraController},
    },
    LookTransform, LookTransformBundle, Smoother,
};
pub fn setup_camera(mut commands: Commands) {
    let mut cam_transform = Transform::from_xyz(0., 8., 0.0);
    cam_transform.rotate_x(PI / 2.);
    commands.spawn(LookTransformBundle {
        transform: LookTransform {
            eye: Vec3::new(0.0, 16., -4.5),
            target: Vec3::new(0.0, 0.0, 0.0),
            up: Vec3::Y,
        },
        smoother: Smoother::new(0.99),
    })
    .insert((
    Camera3dBundle {
    transform: cam_transform,
    ..default()
    }, PickingCameraBundle::default(),
    //PickRaycastSource::default(),
    ));
    //commands
        //.spawn(Camera3dBundle::default())
        //.insert(UnrealCameraBundle::new(
            //UnrealCameraController::default(),
            //Vec3::new(-2.0, 5.0, 5.0),
            //Vec3::new(0., 0., 0.),
            //Vec3::Y,
        //));
    //commands
    //.spawn(Camera3dBundle::default())
    //.insert(OrbitCameraBundle::new(
    //OrbitCameraController {
    //mouse_rotate_sensitivity: Vec2::new(10., 10.),
    //..default()
    //},
    //Vec3::new(-2.0, 5.0, 5.0),
    //Vec3::new(0., 0., 0.),
    //Vec3::Y,
    //));
}

pub fn camera_follow(
    mut camera_query: Query<&mut LookTransform, (With<Camera>, Without<ControlledEntity>)>,
    player_query: Query<&Transform, With<ControlledEntity>>,
) {
    let mut cam_transform = camera_query.single_mut();
    if let Ok(player_transform) = player_query.get_single() {
        cam_transform.eye.x = player_transform.translation.x;
        cam_transform.eye.z = player_transform.translation.z + 2.5;
        cam_transform.target = player_transform.translation;
    }
}
