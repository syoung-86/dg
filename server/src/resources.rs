use bevy::{prelude::{Resource}, utils::HashMap};
use lib::components::Client;

#[derive(Resource, Default)]
pub struct ServerLobby {
    pub clients: HashMap<u64, Client>,
}
