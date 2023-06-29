use bevy::prelude::*;
use lib::components::Slime;

#[derive(Resource, Default)]
pub struct SlimeAnimations(pub Vec<Handle<AnimationClip>>);

pub fn slime_anims(
    animations: Res<SlimeAnimations>,
    mut animation_players: Query<(&Parent, &mut AnimationPlayer)>,
    player_parent: Query<(Entity, &Parent, &Children)>,
    slime_query: Query<Entity, With<Slime>>,
) {
    for (parent, mut player) in animation_players.iter_mut() {
        let player_parent_get = parent.get();
        for (player_parent_entity, parent_player_parent, _) in player_parent.iter() {
            if player_parent_get == player_parent_entity {
                let entity_animate = parent_player_parent.get();
                for e in slime_query.iter() {
                    if e == entity_animate {
                        player.play(animations.0[1].clone_weak()).repeat();
                    }
                }
            }
        }
    }
}
