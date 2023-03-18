use bevy::prelude::*;
use bevy_renet::{
    renet::{RenetConnectionConfig, RenetServer, ServerAuthentication, ServerConfig, ServerEvent},
    RenetServerPlugin,
};
use shared::components::Client;
use shared::PROTOCOL_ID;
use shared::{
    channels::{ClientChannel, ServerChannel},
    components::{Player, Scope, TilePos},
};
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
) {
    for event in events.drain() {
        match event {
            ServerEvent::ClientConnected(id, _) => {
                println!("client connected {}", id);
                let new_client = commands
                    .spawn((
                        Player,
                        TilePos { cell: (4, 0, 0) },
                        Client {
                            id,
                            scope: Scope::get(TilePos { cell: (4, 0, 4) }),
                        },
                    ))
                    .id();
                request_event.send(ChunkRequest(id));
                server_lobby.clients.insert(id, new_client);
            }

            ServerEvent::ClientDisconnected(id) => {
                println!("client disconnected {}", id);
                if let Some((_, client_entity)) = server_lobby.clients.remove_entry(&id) {
                    commands.entity(client_entity).despawn();
                }
            }
        }
    }
}
