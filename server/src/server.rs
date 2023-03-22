use std::time::Duration;

use bevy::prelude::*;
use bevy_renet::{renet::RenetServer, RenetServerPlugin};
use connection::{client_handler, new_renet_server};
use events::{ChunkRequest, ClientSetup};
use lib::{
    channels::ServerChannel,
    components::{Client, EntityType, TilePos},
    resources::Tick,
    TickSet,
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
    app.add_system(tick.in_schedule(CoreSchedule::FixedUpdate));
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

    app.add_systems(
        (RenetServerPlugin::get_clear_event_systems().in_set(TickSet::Clear))
            .in_schedule(CoreSchedule::FixedUpdate),
    );
    app.add_startup_system(create_tiles);
    app.add_event::<ClientSetup>();
    app.run();
}
#[bevycheck::system]
pub fn tick(mut tick: ResMut<Tick>) {
    tick += 1;
}
pub fn send_chunk(
    query: Query<(Entity, &EntityType, &TilePos)>,
    mut requests: ResMut<Events<ChunkRequest>>,
    clients: Query<&Client>,
    mut server: ResMut<RenetServer>,
) {
    for request in requests.drain() {
        for client in clients.iter() {
            if client.id == request.0 {
                let scope: Vec<(Entity, EntityType, TilePos)> = query
                    .iter()
                    .filter(|(_entity, _entity_type, pos)| client.scope.check(**pos))
                    .map(|(entity, ty, pos)| (entity, ty.clone(), *pos))
                    .collect();
                let message = bincode::serialize(&scope).unwrap();
                server.send_message(client.id, ServerChannel::Load, message);
            }
        }
    }
}
