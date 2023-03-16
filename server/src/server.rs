use std::time::Duration;

use bevy::prelude::*;
use connection::{client_handler, new_renet_server};
use events::{ChunkRequest, ClientSetup};
use plugins::{ClearEventPlugin, ConfigPlugin};
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
    //app.add_plugin(ClearEventPlugin);

    app.insert_resource(FixedTime::new(Duration::from_millis(100)));
    app.insert_resource(new_renet_server());
    app.init_resource::<ServerLobby>();
    app.init_resource::<Events<ChunkRequest>>();
    app.init_resource::<Events<ClientSetup>>();
    app.add_systems(
        (
            client_handler.in_set(TickSet::Connection),
            client_setup.in_set(TickSet::Connection),
        )
            .in_schedule(CoreSchedule::FixedUpdate),
    );
    app.add_system(
        send_chunk
            .in_set(TickSet::SendChunk)
            .in_schedule(CoreSchedule::FixedUpdate),
    );

    app.add_startup_system(create_tiles);
    app.add_event::<ClientSetup>();
    app.run();
}
