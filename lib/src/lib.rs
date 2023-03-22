use bevy::prelude::*;

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
