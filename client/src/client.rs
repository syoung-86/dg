use std::{net::UdpSocket, time::SystemTime};

use bevy::prelude::*;
use bevy_replicon::{
    prelude::*,
    renet::{ClientAuthentication, RenetConnectionConfig},
};
use lib::components::Tile;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(ReplicationPlugins.build().disable::<ServerPlugin>());
    app.add_startup_system(spawn_tile);
    app.add_startup_system(new_client);
    app.replicate::<Tile>();
    app.register_type::<(u32, u32, u32)>();
    app.add_system(print_tile);

    app.run();
}

pub fn spawn_tile(mut commands: Commands) {
    commands.spawn((Tile::default(), Replication));
}

pub fn print_tile(query: Query<(Entity, &Tile)>) {
    for tile in &query {
        println!("Tile: {:?}", tile);
    }
}


const PROTOCOL_ID: u64 = 0;
pub fn new_client(mut commands: Commands, network_channels: Res<NetworkChannels>) {
    let send_channels_config = network_channels.client_channels();
    let receive_channels_config = network_channels.server_channels();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;
    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    let connection_config = RenetConnectionConfig {
        send_channels_config,
        receive_channels_config,
        ..Default::default()
    };

    let client = RenetClient::new(current_time, socket, connection_config, authentication).unwrap();
    commands.insert_resource(client);
}
