use crate::events::ChunkRequest;
use bevy::prelude::*;
use shared::channels::ServerChannel;

pub fn send_chunk(mut requests: ResMut<Events<ChunkRequest>>) {
    for request in requests.drain() {}

    requests.clear();
}
