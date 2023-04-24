use bevy::prelude::*;
use bevy_renet::renet::RenetServer;
use lib::{components::SpawnEvent, channels::ServerChannel};

pub fn spawn(mut server: ResMut<RenetServer>, mut spawn_event: EventReader<SpawnEvent>){
    for event in spawn_event.iter(){
                let message = bincode::serialize(&event).unwrap();
                server.broadcast_message(ServerChannel::Spawn, message);
    }
}
