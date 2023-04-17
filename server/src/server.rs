use std::time::Duration;

use bevy::prelude::*;
use bevy_renet::{renet::RenetServer, RenetServerPlugin};
use connection::{client_handler, new_renet_server, spawn_player};
use events::{ChunkRequest, ClientSetup};
use lib::{
    channels::{ClientChannel, ServerChannel},
    components::{Client, ComponentType, EntityType, LeftClick, Player, PlayerCommand, Tile},
    resources::Tick,
    ClickEvent, TickSet, UpdateComponentEvent,
};
use plugins::{ClearEventPlugin, ConfigPlugin};
use receive::{left_click, message};
use resources::ServerLobby;
use world::create_tiles;

pub mod connection;
pub mod events;
pub mod plugins;
pub mod receive;
pub mod resources;
pub mod world;

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugin(ConfigPlugin);
    app.add_plugin(ClearEventPlugin);

    app.insert_resource(FixedTime::new(Duration::from_millis(100)));
    app.insert_resource(Tick::default());
    app.insert_resource(new_renet_server());
    app.init_resource::<ServerLobby>();
    app.init_resource::<Events<ChunkRequest>>();
    app.init_resource::<Events<ClientSetup>>();
    app.init_resource::<Events<LeftClickEvent>>();
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
    //app.add_system(receive_clicks)
    //.init_schedule(CoreSchedule::FixedUpdate);
    app.add_systems(
        (message, left_click, replicate_players)
            .chain()
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

pub fn replicate_players(
    mut server: ResMut<RenetServer>,
    players: Query<(Entity, &Tile), (With<Player>, Changed<Tile>)>,
    //clients: Query<&ClientInfo>,
) {
    for client in server.clients_id().into_iter() {
        for (e, tile) in players.iter() {
            let update_component: (Entity, Vec<ComponentType>) =
                (e, vec![ComponentType::Tile(*tile)]);
            println!("update compoennt: {:?}", update_component);
            let message = bincode::serialize(&(update_component)).unwrap();
            server.send_message(client, ServerChannel::Update, message);
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
pub fn send_chunk(
    query: Query<(Entity, &EntityType, &Tile)>,
    mut requests: ResMut<Events<ChunkRequest>>,
    clients: Query<&Client>,
    mut server: ResMut<RenetServer>,
) {
    for request in requests.drain() {
        for client in clients.iter() {
            println!("send load message");
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
