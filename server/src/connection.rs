use bevy::prelude::*;
use bevy_renet::{
    renet::{RenetConnectionConfig, RenetServer, ServerAuthentication, ServerConfig, ServerEvent},
    RenetServerPlugin,
};
use lib::components::{Client, ComponentType, EntityType, ServerMessages};
use lib::PROTOCOL_ID;
use lib::{
    channels::{ClientChannel, ServerChannel},
    components::{Player, Scope, Tile},
};
use rand::Rng;
use std::{net::UdpSocket, time::SystemTime};

use crate::{
    events::{ChunkRequest, ClientSetup},
    resources::ServerLobby,
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
    mut events: ResMut<Events<ServerEvent>>,
    mut request_event: EventWriter<ChunkRequest>,
    mut server: ResMut<RenetServer>,
) {
    for event in events.drain() {
        match event {
            ServerEvent::ClientConnected(id, _) => {
                println!("client connected {}", id);
                let mut rng = rand::thread_rng();
                let x: u32 = rng.gen_range(0..10);
                let player = commands
                    .spawn((EntityType::Player(Player { id }), Tile { cell: (x, 0, 4) }))
                    .id();
                let new_client = Client {
                    id,
                    scope: Scope::get(Tile { cell: (4, 0, 4) }),
                    controlled_entity: player,
                };
                commands.spawn(new_client);
                request_event.send(ChunkRequest(id));
                server_lobby.clients.insert(id, new_client);
                let message = ServerMessages::PlayerConnected { id };
                let message = bincode::serialize(&message).unwrap();
                server.broadcast_message(ServerChannel::ServerMessages, message);
            }

            ServerEvent::ClientDisconnected(id) => {
                println!("client disconnected {}", id);
                if let Some((_, client_entity)) = server_lobby.clients.remove_entry(&id) {
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