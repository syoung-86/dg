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
        Client, ComponentType, Door, Dummy, EntityType, Health, LeftClick, Lever, Open, Player,
        PlayerCommand, SpawnEvent, Sword, Tile, Wall,
    },
    resources::Tick,
    ClickEvent, TickSet, UpdateComponentEvent,
};
use plugins::{ClearEventPlugin, ConfigPlugin};
use receive::{left_click, message};
use resources::ServerLobby;
use seldom_state::prelude::*;
use state::Moving;
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
            message,
            left_click,
            replicate_players,
            send_item,
            send_room,
            open_door,
        )
            .chain()
            .in_schedule(CoreSchedule::FixedUpdate),
    );
    app.add_systems(
        (RenetServerPlugin::get_clear_event_systems().in_set(TickSet::Clear))
            .in_schedule(CoreSchedule::FixedUpdate),
    );
    app.add_startup_system(create_tiles);
    app.add_startup_system(spawn_item);
    app.add_startup_system(spawn_room);

    app.add_event::<ClientSetup>();
    app.run();
}

pub fn open_door(
    mut commands: Commands,
    mut events: EventReader<PullEvent>,
    lever_query: Query<(&Tile, &Parent), With<Lever>>,
    door_query: Query<Entity, With<Door>>,
    mut server: ResMut<RenetServer>,
) {
    for event in events.iter() {
        for (lever_tile, lever_parent) in lever_query.iter() {
            if event.tile == *lever_tile {
                for door_entity in door_query.iter() {
                    if lever_parent.get() == door_entity {
                        commands.entity(door_entity).insert(Open);
                        println!("FOUND LEVER PARENT");
                        let message =
                            bincode::serialize(&(door_entity, ComponentType::Open(Open))).unwrap();
                        server.broadcast_message(ServerChannel::Update, message);
                    }
                }
            }
        }
        println!("Pull event: {:?}", event);
    }
}
pub fn send_room(
    mut server: ResMut<RenetServer>,
    walls: Query<(Entity, &Tile, &Wall)>,
    doors: Query<(Entity, &Tile, &Door, Option<&Open>)>,
    levers: Query<(Entity, &Tile, &Lever)>,
    dummy: Query<(Entity, &Dummy, &Tile)>,
    mut events: EventReader<ServerEvent>,
) {
    for event in events.iter() {
        match event {
            ServerEvent::ClientConnected(id, _) => {
                println!("send item");
                for (entity, tile, wall) in walls.iter() {
                    let spawn_event = SpawnEvent {
                        entity,
                        entity_type: EntityType::Wall(*wall),
                        tile: *tile,
                    };

                    let message = bincode::serialize(&spawn_event).unwrap();
                    server.send_message(*id, ServerChannel::Spawn, message);
                }

                for (entity, dummy, tile) in dummy.iter() {
                    let spawn_event = SpawnEvent {
                        entity,
                        entity_type: EntityType::Dummy(*dummy),
                        tile: *tile,
                    };

                    let message = bincode::serialize(&spawn_event).unwrap();
                    server.send_message(*id, ServerChannel::Spawn, message);
                }
                for (entity, tile, door, open) in doors.iter() {
                    let spawn_event = SpawnEvent {
                        entity,
                        entity_type: EntityType::Door(*door),
                        tile: *tile,
                    };

                    let message = bincode::serialize(&spawn_event).unwrap();
                    server.send_message(*id, ServerChannel::Spawn, message);

                    match open {
                        Some(open) => {
                            println!("MATCH OPEN");
                            let message =
                                bincode::serialize(&(entity, ComponentType::Open(*open))).unwrap();
                            server.send_message(*id, ServerChannel::Update, message);
                        }
                        None => (),
                    }
                }

                for (entity, tile, lever) in levers.iter() {
                    let spawn_event = SpawnEvent {
                        entity,
                        entity_type: EntityType::Lever(*lever),
                        tile: *tile,
                    };

                    let message = bincode::serialize(&spawn_event).unwrap();
                    server.send_message(*id, ServerChannel::Spawn, message);
                }
            }
            _ => (),
        }
    }
}
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
pub fn send_item(
    mut server: ResMut<RenetServer>,
    query: Query<Entity, With<Sword>>,
    mut events: EventReader<ServerEvent>,
) {
    for event in events.iter() {
        match event {
            ServerEvent::ClientConnected(id, _) => {
                println!("send item");
                if let Ok(entity) = query.get_single() {
                    let spawn_event = SpawnEvent {
                        entity,
                        entity_type: EntityType::Sword(Sword),
                        tile: Tile { cell: (4, 0, 4) },
                    };

                    let message = bincode::serialize(&spawn_event).unwrap();
                    server.send_message(*id, ServerChannel::Spawn, message);
                }
            }
            _ => (),
        }
    }
}
pub fn spawn_item(mut commands: Commands) {
    commands.spawn((Sword, Tile { cell: (4, 0, 4) }));
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
            //println!("update component: {:?}", update_component);
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
                    .filter(|(_entity, _entity_type, pos)| client.scope.check(**pos))
                    .map(|(entity, entity_type, pos)| (entity, entity_type.clone(), *pos))
                    .collect();
                let message = bincode::serialize(&scope).unwrap();
                server.send_message(client.id, ServerChannel::Load, message);
            }
        }
    }
}
