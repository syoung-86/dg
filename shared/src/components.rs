use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct ControlledEntity;

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ServerMessages {
    PlayerConnected { id: u64 },
    PlayerDisconnected { id: u64 },
}

#[derive(Serialize, Deserialize, Component)]
pub struct Client;
