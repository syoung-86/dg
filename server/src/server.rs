use std::time::Duration;

use bevy::prelude::*;
use bevy_renet::{renet::RenetServer, RenetServerPlugin};
use connection::{client_handler, new_renet_server};
use events::{ChunkRequest, ClientSetup};
use lib::{
    channels::ServerChannel,
    components::{
        Client, Dummy, EntityType, Health, LeftClick, Player, Scope, SpawnEvent, SyncEvent, Tile,
    },
    resources::Tick,
    TickSet,
};
use plugins::{ClearEventPlugin, ConfigPlugin};
use receive::{left_click, message};
use resources::ServerLobby;
use seldom_state::prelude::*;
use send::spawn;
use sync::{
    create_scope, entered_left_scope, send_chunk, send_updates, update_health, update_tile,
};
use world::create_tiles;

pub mod connection;
pub mod events;
pub mod plugins;
pub mod receive;
pub mod resources;
pub mod send;
pub mod state;
pub mod sync;
pub mod world;

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugin(ConfigPlugin);
    app.add_plugin(ClearEventPlugin);
    app.add_plugin(StateMachinePlugin);

    app.insert_resource(FixedTime::new(Duration::from_millis(100)));
    app.insert_resource(Tick::default());
    app.insert_resource(new_renet_server());
    app.init_resource::<ServerLobby>();
    app.init_resource::<Events<ChunkRequest>>();
    app.init_resource::<Events<ClientSetup>>();
    app.init_resource::<Events<LeftClickEvent>>();
    app.init_resource::<Events<SpawnEvent>>();
    app.init_resource::<Events<SyncEvent>>();
    app.add_systems(
        (tick, send_tick)
            .chain()
            .in_schedule(CoreSchedule::FixedUpdate),
    );
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
        (
            //spawn,
            create_scope,
            entered_left_scope,
            message,
            left_click,
            //change_health,
            update_tile,
            update_health,
            send_updates,
        )
            .chain()
            .in_schedule(CoreSchedule::FixedUpdate),
    );
    app.add_systems(
        (RenetServerPlugin::get_clear_event_systems().in_set(TickSet::Clear))
            .in_schedule(CoreSchedule::FixedUpdate),
    );
    app.add_startup_system(create_tiles);
    app.add_startup_system(spawn_dummy);
    app.add_event::<ClientSetup>();
    app.run();
}

pub fn spawn_dummy(mut commands: Commands, mut spawn_event: EventWriter<SpawnEvent>) {
    let id = commands
        .spawn((EntityType::Dummy(Dummy), Health::new(99), Tile::new((1, 0, 1))))
        .id();
    spawn_event.send(SpawnEvent::new(
        id,
        EntityType::Dummy(Dummy),
        Tile::new((1, 0, 1)),
    ));
}
pub fn change_health(mut query: Query<&mut Health>, tick: Res<Tick>) {
    for mut hp in query.iter_mut() {
        if tick.tick % 10 == 0 {
            hp.hp += 1;
        }
    }
}
#[derive(Debug)]
pub struct LeftClickEvent {
    pub client_id: u64,
    pub left_click: LeftClick,
    pub tile: Tile,
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
