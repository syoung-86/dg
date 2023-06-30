use bevy::prelude::*;
use core::any::type_name;
use leafwing_input_manager::prelude::*;
use lib::components::{Action, Idle, Player, PlayerCommand, Running, Tile};
use seldom_state::prelude::*;
pub fn auto_attack(
    //query: Query<&ActionState<Action>, (With<Player>, With<Target>)>,
    query: Query<&ActionState<Action>>,
    mut player_command: EventWriter<PlayerCommand>,
) {
    if let Ok(action_state) = query.get_single() {
        if action_state.just_pressed(Action::AutoAttack) {
            println!("auto attack!");
            player_command.send(PlayerCommand::AutoAttack);
        }
    }
}

#[derive(Default, Clone, Copy, Component, Reflect)]
#[reflect(Component)]
pub struct Punching;

#[derive(Debug, Deref, DerefMut, Reflect)]
pub struct JustPressedTrigger<A: Actionlike>(pub A);

impl<A: Actionlike + Reflect> BoolTrigger for JustPressedTrigger<A> {
    type Param<'w, 's> = Query<'w, 's, &'static ActionState<A>>;

    fn trigger(
        &self,
        entity: Entity,
        actors: &bevy::prelude::Query<
            '_,
            '_,
            &'static leafwing_input_manager::action_state::ActionState<A>,
        >,
    ) -> bool {
        let Self(action) = self;
        actors
            .get(entity)
            .unwrap_or_else(|_| {
                panic!(
                    "entity {entity:?} with `JustPressedTrigger<{0}>` is missing `ActionState<{0}>`",
                    type_name::<A>()
                )
            })
            .just_pressed(action.clone())
    }
}
#[derive(Clone, Copy, FromReflect, Reflect)]
pub struct Moving;

impl Trigger for Moving {
    // put the parameters that your trigger needs here
    // for concision, you may use `bevy_ecs::system::system_param::lifetimeless` variants of system
    // params, like so:
    // type param<'w, 's> = (squery<&'static transform>, sres<time>);
    // triggers are immutable; you may not access system params mutably
    // do not query for the `statemachine` component in this type. this, unfortunately, will panic.
    // `time` is included here to demonstrate how to get multiple system params
    type Ok = f32;
    type Err = f32;
    //type param<'w, 's> = (query<'w, 's, &'static pathmap>, res<'w, tick>);
    type Param<'w, 's> = Query<'w, 's, (Entity, &'static Player), Changed<Transform>>;

    // this function checks if the given entity should trigger
    // it runs once per frame for each entity that is in a state that can transition
    // on this trigger
    fn trigger(&self, self_entity: Entity, player: &Self::Param<'_, '_>) -> Result<f32, f32> {
        if let Some((moving_entity, _)) = player.iter().next() {
            if self_entity == moving_entity {
                Ok(0.0)
            } else {
                Err(1.0)
            }
        } else {
            Err(1.0)
        }
    }
}
#[derive(Bundle)]
pub struct PlayerBundle {
    tile: Tile,
    state: StateMachine,
}

impl PlayerBundle {
    pub fn new(tile: &Tile) -> Self {
        PlayerBundle {
            tile: *tile,
            state: StateMachine::new(Idle)
                .trans::<Idle>(Moving, Running)
                .insert_on_enter::<Running>(Running)
                .remove_on_exit::<Running, Running>()
                .trans::<Running>(NotTrigger(Moving), Idle)
                .insert_on_enter::<Idle>(Idle)
                .remove_on_exit::<Idle, Idle>(),
        }
    }
}
