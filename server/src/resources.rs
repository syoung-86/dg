use bevy::{prelude::{Entity, Resource}, utils::HashMap};

#[derive(Resource, Default)]
pub struct ServerLobby {
    pub clients: HashMap<u64, Entity>,
}
