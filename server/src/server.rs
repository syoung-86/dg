use std::time::Duration;

use bevy::prelude::*;
use bevy_renet::{
    renet::{RenetServer, ServerEvent},
    RenetServerPlugin,
};
use connection::{client_handler, new_renet_server, spawn_player};
use events::{ChunkRequest, ClientSetup};
use lib::{
    channels::{ClientChannel, ServerChannel},
    components::{
        Client, ComponentType, DespawnEvent, Door, Dummy, EntityType, Health, LeftClick, Lever,
        Open, Player, PlayerCommand, Scope, SpawnEvent, Sword, Tile, UpdateEvent, Wall,
    },
    resources::Tick,
    ClickEvent, TickSet, UpdateComponentEvent,
};
use plugins::{ClearEventPlugin, ConfigPlugin};
use receive::{left_click, message};
use resources::ServerLobby;
use seldom_state::prelude::*;
use state::{send_state, Moving};
use world::create_tiles;

pub mod connection;
pub mod events;
pub mod plugins;
pub mod receive;
pub mod resources;
pub mod send;
pub mod state;
pub mod world;

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
    app.init_resource::<Events<PullEvent>>();
    app.init_resource::<Events<AttackEvent>>();
    app.init_resource::<Events<ServerUpdateEvent>>();
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
        (
            create_scope,
            //entered_left_scope,
            message,
            left_click,
            //replicate_players,
            //send_state,
            //change_health,
            update_tile,
            //update_health,
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
    app.add_startup_system(spawn_room);
    //app.add_system(send_state);
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
    entities: Query<(Entity, &Tile, &EntityType), Changed<Tile>>,
    mut server: ResMut<RenetServer>,
    players: Query<(Entity, &Tile), Changed<Tile>>,
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
                    println!("scope spawn");
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

pub struct ServerUpdateEvent {
    event: UpdateEvent,
    client_id: u64,
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
macro_rules! update_in_scope {
    ($fn_name:ident, $type_name:ident) => {
        pub fn $fn_name(
            clients: Query<&Client>,
            components: Query<(Entity, &$type_name), Changed<$type_name>>,
            mut update_event: EventWriter<ServerUpdateEvent>,
        ) {
            for client in clients.iter() {
                for (entity, component) in components.iter() {
                    //if client.scoped_entities.contains(&entity) {
                    let event = UpdateEvent {
                        entity,
                        component: ComponentType::$type_name(*component),
                    };
                    //let message: (Entity, ComponentType) =
                    //(entity, ComponentType::$type_name(*component));
                    //let message = bincode::serialize(&message).unwrap();
                    //server.send_message(client.id, ServerChannel::Update, message);
                    update_event.send(ServerUpdateEvent {
                        event,
                        client_id: client.id,
                    });
                    //}
                }
            }
        }
    };
}

update_in_scope!(update_health, Health);
update_in_scope!(update_tile, Tile);
//pub fn update_in_scope(
//clients: Query<&Client>,
//components: Query<(Entity, &Health), Changed<Health>>,
//mut server: ResMut<RenetServer>,
//) {
//for client in clients.iter() {
//for (entity, component) in components.iter() {
//if client.scoped_entities.contains(&entity) {
//let message = UpdateEvent {
//entity,
//component: ComponentType::Health(*component),
//};
//let message = bincode::serialize(&message).unwrap();
//server.send_message(client.id, ServerChannel::Update, message);
//}
//}
//}
//}
// scope
// check everything in scope,
// hashset of all e's in scope
// know if something enters/leaves scope
// macro that
pub fn spawn_room(mut commands: Commands) {
    for x in 1..10 {
        commands.spawn((Wall::Horizontal, Tile { cell: (x, 0, 0) }));
        commands.spawn((Wall::Horizontal, Tile { cell: (x, 0, 10) }));
    }

    for z in 0..10 {
        commands.spawn((Wall::Vertical, Tile { cell: (0, 0, z) }));
    }
    for z in 0..3 {
        commands.spawn((Wall::Vertical, Tile { cell: (10, 0, z) }));
    }
    for z in 6..10 {
        commands.spawn((Wall::Vertical, Tile { cell: (10, 0, z) }));
    }
    let door_entity = commands
        .spawn((Door::Vertical, Tile { cell: (10, 0, 4) }))
        .id();
    let lever_entity = commands.spawn((Lever, Tile { cell: (5, 0, 5) })).id();
    commands.entity(door_entity).push_children(&[lever_entity]);
    commands.spawn((Dummy, Health { hp: 99 }, Tile { cell: (2, 0, 2) }));
}
#[derive(Debug)]
pub struct LeftClickEvent {
    pub client_id: u64,
    pub left_click: LeftClick,
    pub tile: Tile,
}

#[derive(Debug)]
pub struct PullEvent {
    pub tile: Tile,
}

#[derive(Debug)]
pub struct AttackEvent {
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
                    //.filter(|(_entity, _entity_type, pos)| client.scope.check(*pos))
                    .map(|(entity, entity_type, pos)| (entity, entity_type.clone(), *pos))
                    .collect();
                let message = bincode::serialize(&scope).unwrap();
                server.send_message(client.id, ServerChannel::Load, message);
            }
        }
    }
}
