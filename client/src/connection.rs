use bevy::prelude::*;
use bevy_renet::renet::RenetConnectionConfig;
use bevy_renet::renet::{ClientAuthentication, RenetClient};
use shared::channels::{ClientChannel, ServerChannel};
use shared::components::ServerMessages;
use shared::PROTOCOL_ID;
use std::{net::UdpSocket, time::SystemTime};

use crate::resources::{ClientInfo, ClientLobby};

pub fn client_connection_config() -> RenetConnectionConfig {
    RenetConnectionConfig {
        send_channels_config: ClientChannel::channels_config(),
        receive_channels_config: ServerChannel::channels_config(),
        ..Default::default()
    }
}
pub fn new_renet_client() -> RenetClient {
    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let connection_config = client_connection_config();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;
    println!("client_id: {:?}", client_id);
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };
    RenetClient::new(current_time, socket, connection_config, authentication).unwrap()
}

pub fn server_messages(mut client: ResMut<RenetClient>, mut lobby: ResMut<ClientLobby>) {
    while let Some(message) = client.receive_message(ServerChannel::ServerMessages) {
        let server_message: ServerMessages = bincode::deserialize(&message).unwrap();

        match server_message {
            ServerMessages::PlayerConnected { id } => {
                println!("Player {} connected", id);
                lobby.clients.insert(id, ClientInfo::default());
            }

            ServerMessages::PlayerDisconnected { id } => {
                lobby.clients.remove(&id);
                println!("Player {} disconnected", id);
            }
        }
    }
}
