use std::{net::UdpSocket, time::SystemTime};

use bevy::prelude::*;
use bevy_replicon::{
    prelude::*,
    renet::{RenetConnectionConfig, ServerAuthentication, ServerConfig, ServerEvent},
};
use lib::components::Tile;

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(
        ReplicationPlugins
            .build()
            .disable::<ClientPlugin>()
            .set(ServerPlugin { tick_rate: 10 }),
    );
    app.replicate::<Tile>();
    app.add_startup_system(spawn_tile);
    app.add_startup_system(new_server);
    app.add_system(change_tile.in_set(ServerSet::Tick));
    app.add_system(server_event_system);
    app.run();
}

pub fn spawn_tile(mut commands: Commands) {
    for _i in 1..10 {
        commands.spawn((Tile::default(), Replication));
    }
}

pub fn change_tile(mut query: Query<&mut Tile>) {
    for mut tile in &mut query {
        tile.cell.0 += 1;
        println!("tile: {:?}", tile);
    }
}

const PROTOCOL_ID: u64 = 0;
pub fn new_server(mut commands: Commands, network_channels: Res<NetworkChannels>) {
    let send_channels_config = network_channels.server_channels();
    let receive_channels_config = network_channels.client_channels();
    const MAX_CLIENTS: usize = 9;
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind(server_addr).unwrap();
    let server_config = ServerConfig::new(
        MAX_CLIENTS,
        PROTOCOL_ID,
        server_addr,
        ServerAuthentication::Unsecure,
    );

    let connection_config = RenetConnectionConfig {
        send_channels_config,
        receive_channels_config,
        ..Default::default()
    };

    let server = RenetServer::new(current_time, server_config, connection_config, socket).unwrap();

    commands.insert_resource(server);
}
fn server_event_system(mut server_event: EventReader<ServerEvent>) {
    for event in &mut server_event {
        match event {
            ServerEvent::ClientConnected(client_id, _) => {
                println!("client connected: {:?}", client_id);
            }
            ServerEvent::ClientDisconnected(_) => {}
        }
    }
}
