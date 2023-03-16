use std::time::Duration;

use bevy::prelude::*;
use bevy_renet::RenetServerPlugin;
use connection::{client_handler, new_renet_server};
use events::{ChunkRequest, ClientSetup};
use plugins::ConfigPlugin;
use resources::ServerLobby;
use send::send_chunk;
use shared::TickSet;
use world::{client_setup, create_tiles};

pub mod connection;
pub mod events;
pub mod plugins;
pub mod resources;
pub mod send;
pub mod world;

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugin(ConfigPlugin);

    app.insert_resource(FixedTime::new(Duration::from_millis(100)));
    app.insert_resource(new_renet_server());
    app.init_resource::<ServerLobby>();
    app.init_resource::<Events<ChunkRequest>>();
    app.init_resource::<Events<ClientSetup>>();
    app.add_systems((client_handler, client_setup).in_schedule(CoreSchedule::FixedUpdate));
    app.add_system(
        send_chunk
            .in_set(TickSet::SendChunk)
            .in_schedule(CoreSchedule::FixedUpdate),
    );

    app.add_systems((
        clear_event::<ClientSetup>
            .in_schedule(CoreSchedule::FixedUpdate)
            .in_set(TickSet::Clear),
        clear_event::<ChunkRequest>
            .in_schedule(CoreSchedule::FixedUpdate)
            .in_set(TickSet::Clear),
    ));
    app.add_startup_system(create_tiles);
    app.add_event::<ClientSetup>();
    app.run();
}

fn clear_event<T: 'static + Send + Sync + std::fmt::Debug>(mut events: ResMut<Events<T>>) {
    for event in events.drain() {
        println!("even clear: {:?}", event);
    }
    events.clear();
}
