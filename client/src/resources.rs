use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Resource, Default)]
pub struct NetworkMapping {
    pub client_to_server: HashMap<Entity, Entity>,
}

#[derive(Default)]
pub struct ClientInfo {
    pub client_entity: Option<Entity>,
    pub server_entity: Option<Entity>,
    pub controlled_entity: Option<Entity>,
}

#[derive(Resource, Default)]
pub struct ClientLobby {
    pub clients: HashMap<u64, ClientInfo>,
}
