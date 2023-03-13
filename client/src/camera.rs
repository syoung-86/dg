use bevy::prelude::*;
use shared::components::ControlledEntity;
use smooth_bevy_cameras::{LookTransform, LookTransformBundle, Smoother};
pub fn setup_camera(mut commands: Commands) {
    commands
        .spawn(LookTransformBundle {
            transform: LookTransform {
                eye: Vec3::new(0.0, 16., -4.5),
                target: Vec3::new(0.0, 0.0, 0.0),
                up: Vec3::Y,
            },
            smoother: Smoother::new(0.99),
        })
        .insert((
            Camera3dBundle {
                transform: Transform::from_xyz(0., 8.0, 0.0)
                    .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
                ..default()
            },
            //PickRaycastSource::default(),
        ));
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
