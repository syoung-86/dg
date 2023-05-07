use bevy::{prelude::*, utils::HashSet};
use bevy_renet::{
    renet::{RenetConnectionConfig, RenetServer, ServerAuthentication, ServerConfig, ServerEvent},
};
use lib::components::{Client, EntityType, Health, ServerMessages, SpawnEvent};
use lib::PROTOCOL_ID;
use lib::{
    channels::{ClientChannel, ServerChannel},
    components::{Player, Scope, Tile},
};
use rand::Rng;
use seldom_state::prelude::*;
use std::{net::UdpSocket, time::SystemTime};

use crate::{
    events::{ChunkRequest},
    resources::ServerLobby,
    state::{Idle, Moving, Running},
};

pub fn new_renet_server() -> RenetServer {
    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind(server_addr).unwrap();
    let connection_config = server_connection_config();
    let server_config =
        ServerConfig::new(64, PROTOCOL_ID, server_addr, ServerAuthentication::Unsecure);
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    RenetServer::new(current_time, server_config, connection_config, socket).unwrap()
}

pub fn server_connection_config() -> RenetConnectionConfig {
    RenetConnectionConfig {
        send_channels_config: ServerChannel::channels_config(),
        receive_channels_config: ClientChannel::channels_config(),
        ..Default::default()
    }
}

pub fn client_handler(
    mut server_lobby: ResMut<ServerLobby>,
    mut commands: Commands,
    //mut events: ResMut<Events<ServerEvent>>,
    mut events: EventReader<ServerEvent>,
    mut request_event: EventWriter<ChunkRequest>,
    mut server: ResMut<RenetServer>,
) {
    for event in events.iter() {
        match event {
            ServerEvent::ClientConnected(id, _) => {
                println!("client connected {}", id);
                let mut rng = rand::thread_rng();
                let x: u32 = rng.gen_range(0..10);
                let player = commands
                    .spawn((
                        EntityType::Player(Player { id: *id }),
                        Tile { cell: (x, 0, 0) },
                        StateMachine::new(Idle)
                            .trans::<Idle>(Moving, Running)
                            .insert_on_enter::<Running>(Running)
                            .remove_on_exit::<Running, Running>()
                            .trans::<Running>(NotTrigger(Moving), Idle)
                            .insert_on_enter::<Idle>(Idle)
                            .remove_on_exit::<Idle, Idle>(),
                        Player { id: *id },
                        Health { hp: 50 },
                    ))
                    .id();
                let new_client = Client {
                    id: *id,
                    scope: Scope::get(Tile { cell: (0, 0, 0) }),
                    scoped_entities: HashSet::new(),
                    controlled_entity: player,
                };
                commands.spawn(new_client);
                request_event.send(ChunkRequest(*id));
                let new_client = Client {
                    id: *id,
                    scope: Scope::get(Tile { cell: (0, 0, 0) }),
                    scoped_entities: HashSet::new(),
                    controlled_entity: player,
                };
                server_lobby.clients.insert(*id, new_client);
                let message = ServerMessages::PlayerConnected { id: *id };
                let message = bincode::serialize(&message).unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, message);
                let message = SpawnEvent {
                    entity: player,
                    entity_type: EntityType::Player(Player { id: *id }),
                    tile: Tile { cell: (x, 0, 0) },
                };
                let message = bincode::serialize(&message).unwrap();
                server.broadcast_message(ServerChannel::Spawn, message);
            }

            ServerEvent::ClientDisconnected(id) => {
                println!("client disconnected {}", id);
                if let Some((_, client_entity)) = server_lobby.clients.remove_entry(id) {
                    commands
                        .entity(client_entity.controlled_entity)
                        .despawn_recursive();
                    let message = bincode::serialize(&id).unwrap();
                    server.broadcast_message(ServerChannel::Despawn, message);
                }
            }
        }
    }
}

pub fn spawn_player(
    mut server: ResMut<RenetServer>,
    new_player: Query<(Entity, &EntityType, &Tile), Added<EntityType>>,
) {
    for (entity, player, tile) in &new_player {
        let message: (Entity, EntityType, Tile) = (entity, *player, *tile);
        let message = bincode::serialize(&message).unwrap();
        server.broadcast_message(ServerChannel::Spawn, message);
        println!("sent spawn message for new player");
    }
}
