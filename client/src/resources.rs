use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Resource)]
pub struct NetworkMapping {
    client_to_server: HashMap<Entity, Entity>,
}

pub struct ClientInfo {
    client_entity: Entity,
    server_entity: Entity,
    controlled_entity: Entity,
}

#[derive(Resource)]
pub struct ClientLobby {
    players: HashMap<u64, ClientInfo>,
}
