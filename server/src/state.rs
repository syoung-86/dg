use bevy::prelude::*;
use lib::components::{Player, Tile};
use seldom_state::prelude::*;
#[derive(Clone, Component, Reflect)]
#[component(storage = "SparseSet")]
pub struct Idle;

#[derive(Clone, Component, Reflect)]
#[component(storage = "SparseSet")]
pub struct Running;

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
    type Param<'w, 's> = Query<'w, 's, (Entity, &'static Player), Changed<Tile>>;

    // this function checks if the given entity should trigger
    // it runs once per frame for each entity that is in a state that can transition
    // on this trigger
    fn trigger(&self, self_entity: Entity, player: &Self::Param<'_, '_>) -> Result<f32, f32> {
        if let Some((moving_entity, _)) = player.iter().next() {
            if self_entity == moving_entity {
                //println!("Running");
                Ok(0.0)
            } else {
                //println!("Walk");
                Err(1.0)
            }
        } else {
            //println!("Walk");
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

pub fn send_state(running: Query<Entity, Added<Running>>, idle: Query<(Entity, Added<Idle>)>) {
    for e in running.iter() {
        //println!("running");
    }

    for e in idle.iter() {
        //println!("idle");
    }
}
