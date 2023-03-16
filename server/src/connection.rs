use bevy::prelude::*;
use bevy_renet::{
    renet::{RenetConnectionConfig, RenetServer, ServerAuthentication, ServerConfig, ServerEvent},
    RenetServerPlugin,
};
use shared::channels::{ClientChannel, ServerChannel};
use shared::components::Client;
use shared::PROTOCOL_ID;
use std::{net::UdpSocket, time::SystemTime};

use crate::{events::ClientSetup, resources::ServerLobby};

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
    mut new_client_event: EventWriter<ClientSetup>,
) {
    for event in events.drain() {
        match event {
            ServerEvent::ClientConnected(id, _) => {
                println!("client connected {}", id);
                let new_client = commands.spawn(Client { id, ..default() }).id();
                server_lobby.clients.insert(id, new_client);
                new_client_event.send(ClientSetup(id));
                println!("sendevent");
            }

            ServerEvent::ClientDisconnected(id) => {
                println!("client disconnected {}", id);
                if let Some((_, client_entity)) = server_lobby.clients.remove_entry(&id) {
                    commands.entity(client_entity).despawn();
                }
            }
        }
    }
    events.clear();
}
