use bevy::prelude::*;

#[derive(Debug)]
pub struct ClientSetup(pub u64);
#[derive(Debug)]
pub struct ChunkRequest(pub u64);

pub fn clear_event<T: 'static + Send + Sync + std::fmt::Debug>(mut events: ResMut<Events<T>>) {
    for _event in events.drain() {
        //println!("even clear: {:?}", event);
    }
    events.clear();
}
