use bevy::{ecs::entity::EntityMap, prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Default, Serialize, Deserialize, Eq, PartialEq, Debug, Resource)]
pub struct Tick {
    pub tick: u64,
}
