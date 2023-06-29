use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use lib::components::{Health, HealthBar};

pub fn update_health_bar(
    bar: Query<Entity, With<HealthBar>>,
    player: Query<(&Health, &Children), Changed<Health>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (hp, children) in player.iter() {
        for &child in children.iter() {
            if let Ok(entity) = bar.get(child) {
                println!("health {:?}", hp);
                let mut transform = Transform::from_xyz(0., 3., 0.);
                let scale_hp: f32 = hp.hp as f32 / 100.0;
                transform.scale = Vec3::new(1.0, scale_hp, 1.0);
                transform.rotate_z(FRAC_PI_2);
                commands.entity(entity).insert(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Capsule {
                        radius: 0.03,
                        rings: 10,
                        ..default()
                    })),
                    material: materials.add(Color::rgb(0.0, 1.0, 0.0).into()),
                    transform,
                    ..Default::default()
                });
            }
        }
    }
}
