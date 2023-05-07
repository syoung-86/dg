use std::time::Duration;

use bevy::prelude::*;
use bevy_renet::{renet::RenetServer, RenetServerPlugin};
use connection::{client_handler, new_renet_server};
use events::{ChunkRequest, ClientSetup, ServerUpdateEvent};
use lib::{
    channels::ServerChannel,
    components::{
        Client, ComponentType, Door, Dummy, EntityType, Health, LeftClick, Lever, Player, Scope,
        SpawnEvent, Tile, UpdateEvent, Wall,
    },
    resources::Tick,
    TickSet,
};
use plugins::{ClearEventPlugin, ConfigPlugin};
use receive::{left_click, message};
use resources::ServerLobby;
use seldom_state::prelude::*;
use state::Moving;
use sync::{update_tile, update_health};
use world::create_tiles;

pub mod connection;
pub mod events;
pub mod plugins;
pub mod receive;
pub mod resources;
pub mod send;
pub mod state;
pub mod world;
pub mod sync;

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugin(ConfigPlugin);
    app.add_plugin(ClearEventPlugin);
    app.add_plugin(StateMachinePlugin);
    app.add_plugin(TriggerPlugin::<Moving>::default());

    app.insert_resource(FixedTime::new(Duration::from_millis(100)));
    app.insert_resource(Tick::default());
    app.insert_resource(new_renet_server());
    app.init_resource::<ServerLobby>();
    app.init_resource::<Events<ChunkRequest>>();
    app.init_resource::<Events<ClientSetup>>();
    app.init_resource::<Events<LeftClickEvent>>();
    app.init_resource::<Events<SpawnEvent>>();
    app.init_resource::<Events<ServerUpdateEvent>>();
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
            create_scope,
            entered_left_scope,
            message,
            left_click,
            change_health,
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
    app.add_event::<ClientSetup>();
    app.run();
}

pub fn change_health(mut query: Query<&mut Health>, tick: Res<Tick>) {
    for mut hp in query.iter_mut() {
        if tick.tick % 10 == 0 {
            hp.hp += 1;
        }
    }
}
pub fn create_scope(
    mut clients: Query<&mut Client, Added<Client>>,
    entities: Query<(Entity, &Tile)>,
) {
    for mut client in clients.iter_mut() {
        for (e, t) in entities.iter() {
            if client.scope.check(&t) && !client.scoped_entities.contains(&e) {
                client.scoped_entities.insert(e);
                //println!("added e into client: {:?}", client.id);
            }
        }
    }
}

pub fn entered_left_scope(
    mut clients: Query<&mut Client>,
    entities: Query<(Entity, &Tile, &EntityType)>,
    mut server: ResMut<RenetServer>,
    players: Query<(Entity, &Tile), (Changed<Tile>, With<Player>)>,
) {
    for mut client in clients.iter_mut() {
        for (e, t) in players.iter() {
            if client.controlled_entity == e {
                client.scope = Scope::get(*t);
                println!("updated scope");
            }
        }
        for (entity, tile, entity_type) in entities.iter() {
            if client.scoped_entities.contains(&entity) {
                if !client.scope.check(tile) {
                    client.scoped_entities.remove(&entity);
                    //DESPAWN
                    let message = bincode::serialize(&entity).unwrap();
                    server.send_message(client.id, ServerChannel::Despawn, message)
                }
            } else {
                if client.scope.check(tile) {
                    //println!("scope spawn");
                    client.scoped_entities.insert(entity);
                    let message = SpawnEvent {
                        entity,
                        entity_type: *entity_type,
                        tile: *tile,
                    };
                    let message = bincode::serialize(&message).unwrap();
                    server.send_message(client.id, ServerChannel::Spawn, message);
                }
            }
        }
    }
}


pub fn send_updates(
    mut update_event: EventReader<ServerUpdateEvent>,
    mut server: ResMut<RenetServer>,
) {
    for event in update_event.iter() {
        let message = bincode::serialize(&event.event).unwrap();
        server.send_message(event.client_id, ServerChannel::Update, message);
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
            //println!("send load message");
            if client.id == request.0 {
                let scope: Vec<(Entity, EntityType, Tile)> = query
                    .iter()
                    .filter(|(_entity, _entity_type, pos)| client.scope.check(*pos))
                    .map(|(entity, entity_type, pos)| (entity, entity_type.clone(), *pos))
                    .collect();
                let message = bincode::serialize(&scope).unwrap();
                server.send_message(client.id, ServerChannel::Load, message);
            }
        }
    }
}
