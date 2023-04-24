use bevy::prelude::{Commands, DespawnRecursiveExt, Entity, EventWriter, Res, ResMut};
use bevy_renet::renet::RenetClient;
use lib::{
    channels::ServerChannel,
    components::{ComponentType, EntityType, SpawnEvent, Tile, UpdateEvent},
    resources::Tick,
};

use crate::resources::NetworkMapping;

pub fn load_message(
    mut client: ResMut<RenetClient>,
    mut spawn_event: EventWriter<SpawnEvent>,
    mut network_mapping: ResMut<NetworkMapping>,
    mut commands: Commands,
) {
    if let Some(message) = client.receive_message(ServerChannel::Load) {
        println!("received load message");
        let load_message: Vec<(Entity, EntityType, Tile)> = bincode::deserialize(&message).unwrap();
        for (server_entity, entity_type, tile) in load_message {
            if let None = network_mapping.server.get(&server_entity) {
                let entity = commands.spawn_empty().id();
                network_mapping.add(&entity, &server_entity);
                spawn_event.send(SpawnEvent {
                    entity,
                    entity_type,
                    tile,
                });
            }
        }
    }
}

pub fn spawn_message(
    mut client: ResMut<RenetClient>,
    mut spawn_event: EventWriter<SpawnEvent>,
    mut network_mapping: ResMut<NetworkMapping>,
    mut commands: Commands,
) {
    if let Some(message) = client.receive_message(ServerChannel::Spawn) {
        let spawn_message: SpawnEvent = bincode::deserialize(&message).unwrap();
        if let None = network_mapping.server.get(&spawn_message.entity) {
            let entity = commands.spawn_empty().id();
            network_mapping.add(&entity, &spawn_message.entity);
            spawn_event.send(SpawnEvent {
                entity,
                entity_type: spawn_message.entity_type,
                tile: spawn_message.tile,
            });
        }
    }
}
pub fn update_message(
    mut client: ResMut<RenetClient>,
    mut update_event: EventWriter<UpdateEvent>,
    network_mapping: Res<NetworkMapping>,
) {
    if let Some(message) = client.receive_message(ServerChannel::Update) {
        println!("Received Update Message!");
        let (server_entity, component): (Entity, ComponentType) =
            bincode::deserialize(&message).unwrap();
        if let Some(entity) = network_mapping.server.get(&server_entity) {
            update_event.send(UpdateEvent {
                entity: *entity,
                component,
            });
        }
    }
}
pub fn despawn_message(
    mut client: ResMut<RenetClient>,
    mut network_mapping: ResMut<NetworkMapping>,
    mut commands: Commands,
) {
    if let Some(message) = client.receive_message(ServerChannel::Despawn) {
        let despawn_entity: Entity = bincode::deserialize(&message).unwrap();
        if let Some(entity) = network_mapping.server.remove(&despawn_entity) {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn tick(mut client: ResMut<RenetClient>, mut tick: ResMut<Tick>) {
    if let Some(message) = client.receive_message(ServerChannel::Tick) {
        let new_tick: Tick = bincode::deserialize(&message).unwrap();
        tick.tick = new_tick.tick;
    }
}
