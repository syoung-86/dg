use bevy::prelude::*;
use components::{LeftClick, Tile};
use serde::{Deserialize, Serialize};

pub mod channels;
pub mod components;
pub mod resources;
pub const PROTOCOL_ID: u64 = 7;

#[derive(SystemSet, Debug, Hash, Eq, PartialEq, Clone)]
pub enum TickSet {
    Connection,
    ReceiveReliable,
    ReceiveUnreliable,
    SendChunk,
    SendUnreliable,
    SendReliable,
    Clear,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct ClickEvent {
    pub target: Entity,
    pub left_click: LeftClick,
    pub destination: Tile,
}

impl ClickEvent {
    pub fn new(target: Entity, left_click: LeftClick, destination: Tile) -> Self {
        Self {
            target,
            left_click,
            destination,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct UpdateComponentEvent<C: Component>(pub Entity, pub C);

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct OpenEvent {
    pub entity: Entity,
}

#[derive(Serialize, Deserialize)]
pub enum ServerEvents {
    OpenEvent(OpenEvent),
}
