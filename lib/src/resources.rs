use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Default,Serialize, Deserialize, Eq, PartialEq, Debug, Resource)]
pub struct Tick{
    tick: u64,
}
