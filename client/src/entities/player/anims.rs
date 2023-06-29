use bevy::prelude::*;
use lib::{
    components::{CombatState, Running},
    resources::Tick,
};

use crate::Animations;

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
                                if *end_tick >= tick.tick - 3 {
                                    player.play(animations.0[4].clone_weak());
                                    //println!("Punching");
                                } else {
                                    commands.entity(e).insert(CombatState::Idle);
                                    println!("inserted idle");
                                }
                                //continue;
                            }
                            CombatState::Idle => {
                                if running.is_some() {
                                    player.play(animations.0[3].clone_weak()).repeat();
                                } else {
                                    player.play(animations.0[2].clone_weak()).repeat();
                                }
                            } // 0 combat idle
                              // 2 idle idle
                              // 5 block
                              // 6 block idle
                              // 7 walk
                        }
                        //}
                    }
                }
            }
        }
    }
}
