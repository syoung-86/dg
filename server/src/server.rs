use std::time::Duration;

use bevy::prelude::*;
use bevy_renet::{renet::RenetServer, RenetServerPlugin};
use connection::{client_handler, new_renet_server};
use events::{ChunkRequest, ClientSetup};
use lib::{
    channels::{ClientChannel, ServerChannel},
    components::{Client, EntityType, Tile},
    resources::Tick,
    ClickEvent, TickSet,
};
use plugins::{ClearEventPlugin, ConfigPlugin};
use resources::ServerLobby;
use world::create_tiles;

pub mod connection;
pub mod events;
pub mod plugins;
pub mod resources;
pub mod world;

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugin(ConfigPlugin);
    //app.add_plugin(ClearEventPlugin);

    app.insert_resource(FixedTime::new(Duration::from_millis(100)));
    app.insert_resource(Tick::default());
    app.insert_resource(new_renet_server());
    app.init_resource::<ServerLobby>();
    app.init_resource::<Events<ChunkRequest>>();
    app.init_resource::<Events<ClientSetup>>();
    app.add_systems(
        (tick, send_tick)
            .chain()
            .in_schedule(CoreSchedule::FixedUpdate),
    );
    //app.add_system(send_tick.in_schedule(CoreSchedule::FixedUpdate));
    app.add_system(
        client_handler
            .in_set(TickSet::Connection)
            .in_schedule(CoreSchedule::FixedUpdate),
    );
    app.add_system(
        send_chunk
            .in_set(TickSet::SendChunk)
            .in_schedule(CoreSchedule::FixedUpdate),
    );
    app.add_system(receive_clicks)
        .init_schedule(CoreSchedule::FixedUpdate);
    app.add_systems(
        (RenetServerPlugin::get_clear_event_systems().in_set(TickSet::Clear))
            .in_schedule(CoreSchedule::FixedUpdate),
    );
    app.add_startup_system(create_tiles);
    app.add_event::<ClientSetup>();
    app.run();
}

pub fn receive_clicks(mut lobby: Res<ServerLobby>, mut server: ResMut<RenetServer>) {
    for (client_id, _) in lobby.clients.iter() {
        if let Some(message) = server.receive_message(*client_id, ClientChannel::Click) {
            let click: ClickEvent = bincode::deserialize(&message).unwrap();
            println!("click event: {:?}", click);
        }
    }
}

//#[bevycheck::system]
pub fn tick(mut tick: ResMut<Tick>) {
    tick.tick += 1;
}
pub fn send_tick(mut server: ResMut<RenetServer>, tick: Res<Tick>) {
    let tick = Tick { tick: tick.tick };
    let message = bincode::serialize(&tick).unwrap();
    server.broadcast_message(ServerChannel::Tick, message)
}
pub fn send_chunk(
    query: Query<(Entity, &EntityType, &Tile)>,
    mut requests: ResMut<Events<ChunkRequest>>,
    clients: Query<&Client>,
    mut server: ResMut<RenetServer>,
) {
    for request in requests.drain() {
        for client in clients.iter() {
            if client.id == request.0 {
                let scope: Vec<(Entity, EntityType, Tile)> = query
                    .iter()
                    .filter(|(_entity, _entity_type, pos)| client.scope.check(**pos))
                    .map(|(entity, entity_type, pos)| (entity, entity_type.clone(), *pos))
                    .collect();
                let message = bincode::serialize(&scope).unwrap();
                server.send_message(client.id, ServerChannel::Load, message);
            }
        }
    }
}
