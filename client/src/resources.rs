use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Resource, Default)]
pub struct NetworkMapping {
    pub client: HashMap<Entity, Entity>,
    pub server: HashMap<Entity, Entity>,
}

impl NetworkMapping {
    pub fn add(&mut self, client_entity: &Entity, server_entity: &Entity) {
        self.client.insert(*client_entity, *server_entity);
        self.server.insert(*server_entity, *client_entity);
    }
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
