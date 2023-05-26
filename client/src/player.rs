use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;
use lib::{
    components::{CombatState, Health, HealthBar, Running, Target},
    resources::Tick,
};

use crate::{Animations, Punching};

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

pub fn setup_anims(
    animations: Res<Animations>,
    mut animation_players: Query<(&Parent, &mut AnimationPlayer)>,
    player_parent: Query<(Entity, &Parent, &Children)>,
    state: Query<(Entity, Option<&Running>, &CombatState)>,
    tick: Res<Tick>,
    mut commands: Commands,
) {
    for (parent, mut player) in animation_players.iter_mut() {
        let player_parent_get = parent.get();
        for (player_parent_entity, parent_player_parent, _) in player_parent.iter() {
            if player_parent_get == player_parent_entity {
                let entity_animate = parent_player_parent.get();
                for (e, running, combat_state) in state.iter() {
                    if entity_animate == e {
                        //if target.is_some() {
                        //}
                        //println!("combat_state: {:?}", combat_state);
                        match combat_state {
                            CombatState::Punching(end_tick) => {
                                if *end_tick >= tick.tick {
                                    player.play(animations.0[7].clone_weak());
                                    //println!("Punching");
                                } else {
                                    commands.entity(e).insert(CombatState::Idle);
                                    println!("inserted idle");
                                }
                                //continue;
                            }
                            CombatState::Idle => {
                                if running.is_some() {
                                    player.play(animations.0[9].clone_weak()).repeat();
                                } else {
                                    player.play(animations.0[3].clone_weak()).repeat();
                                }
                            }
                        }
                        //}
                    }
                }
            }
        }
    }
}
