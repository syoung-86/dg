use bevy::prelude::*;
use shared::channels::ServerChannel;
use crate::events::ChunkRequest;

pub fn send_chunk(mut requests: EventReader<ChunkRequest>) {
    for request in requests.iter() {
    
    }
}

