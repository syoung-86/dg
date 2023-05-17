use bevy::prelude::*;
use bevy_renet::renet::RenetServer;
use lib::{
    channels::ServerChannel,
    components::{
        Client, ComponentType, EntityType, Health, Player, Scope, SpawnEvent, SyncEvent, Tile,
        UpdateEvent, Dummy,
    },
};

use crate::events::ChunkRequest;

macro_rules! update_component {
    ($fn_name:ident, $type_name:ident) => {
        pub fn $fn_name(
            clients: Query<&Client>,
            components: Query<(Entity, &$type_name), Changed<$type_name>>,
            mut update_event: EventWriter<SyncEvent>,
        ) {
            for client in clients.iter() {
                for (entity, component) in components.iter() {
                    if client.scoped_entities.contains(&entity) {
                        let event = UpdateEvent {
                            entity,
                            component: ComponentType::$type_name(*component),
                        };
                        //println!("sync event: {:?}", event);
                        update_event.send(SyncEvent::Update(client.id, event));
                    }
                }
            }
        }
    };
}
update_component!(update_health, Health);
update_component!(update_tile, Tile);

pub fn send_chunk(
    query: Query<(Entity, &EntityType, &Tile)>,
    mut requests: ResMut<Events<ChunkRequest>>,
    clients: Query<&Client>,
    mut server: ResMut<RenetServer>,
) {
    for request in requests.drain() {
        for client in clients.iter() {
            //println!("send load message");
            if client.id == request.0 {
                let scope: Vec<(Entity, EntityType, Tile)> = query
                    .iter()
                    .filter(|(_entity, _entity_type, pos)| client.scope.check(pos))
                    .map(|(entity, entity_type, pos)| (entity, *entity_type, *pos))
                    .collect();
                let message = bincode::serialize(&scope).unwrap();
                server.send_message(client.id, ServerChannel::Load, message);
            }
        }
    }
}

pub fn create_scope(
    mut clients: Query<&mut Client, Added<Client>>,
    entities: Query<(Entity, &Tile)>,
) {
    for mut client in clients.iter_mut() {
        for (e, t) in entities.iter() {
            if client.scope.check(t) && !client.scoped_entities.contains(&e) {
                client.scoped_entities.insert(e);
                //println!("added e into client: {:?}", client.id);
            }
        }
    }
}

pub fn entered_left_scope(
    mut clients: Query<&mut Client>,
    entities: Query<(Entity, &Tile, &EntityType)>,
    mut server: ResMut<RenetServer>,
    players: Query<(Entity, &Tile), (Changed<Tile>, With<Player>)>,
) {
    for mut client in clients.iter_mut() {
        for (e, t) in players.iter() {
            if client.controlled_entity == e {
                client.scope = Scope::get(*t);
                //println!("updated scope");
            }
        }
        for (entity, tile, entity_type) in entities.iter() {
            if client.scoped_entities.contains(&entity) {
                if !client.scope.check(tile) {
                    client.scoped_entities.remove(&entity);
                    //DESPAWN
                    let message = bincode::serialize(&entity).unwrap();
                    server.send_message(client.id, ServerChannel::Despawn, message)
                }
            } else if client.scope.check(tile) {
                //println!("scope spawn");
                client.scoped_entities.insert(entity);
                let message = SpawnEvent {
                    entity,
                    entity_type: *entity_type,
                    tile: *tile,
                };
                let message = bincode::serialize(&message).unwrap();
                server.send_message(client.id, ServerChannel::Spawn, message);
            }
        }
    }
}

pub fn send_updates(mut update_event: EventReader<SyncEvent>, mut server: ResMut<RenetServer>) {
    for sync_event in update_event.iter() {
        match sync_event {
            SyncEvent::Update(client_id, event) => {
                let message = bincode::serialize(&event).unwrap();
                server.send_message(*client_id, ServerChannel::Update, message);
            }
            _ => (),
        }
    }
}
