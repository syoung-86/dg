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
    //app.add_system(receive_clicks)
    //.init_schedule(CoreSchedule::FixedUpdate);

    app.add_systems(
        (receive_movement, replicate_players)
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
pub fn receive_movement(
    mut server: ResMut<RenetServer>,
    mut commands: Commands,
    lobby: ResMut<ServerLobby>,
    item_query: Query<(Entity, &EntityType)>,
    //mut players: Query<(Entity, &mut Inventory)>,
) {
    for client_id in server.clients_id().into_iter() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::Command) {
            let command: PlayerCommand = bincode::deserialize(&message).unwrap();
            println!("receive  msg {:?}", command);
            match command {
                PlayerCommand::BasicClick(tile) => {
                    println!(
                        "Received basic click from client {}: tile_key: {:?}",
                        client_id, tile
                    );
                    if let Some(player_entity) = lobby.clients.get(&client_id) {
                        println!("inserted new tile");
                    }
                }
                PlayerCommand::LeftClick(left_click, tile) => match left_click {
                    LeftClick::Walk => {
                        if let Some(client) = lobby.clients.get(&client_id) {
                            println!("inserted new tile");
                            commands.entity(client.controlled_entity).insert(tile);
                            let message: (Entity, ComponentType) =
                                (client.controlled_entity, ComponentType::Tile(tile));
                            let serd_message = bincode::serialize(&message).unwrap();
                            server.broadcast_message(ServerChannel::Update, serd_message);
                            //println!("walk");
                        }
                    }
                    LeftClick::Pickup(Some(e)) => {
                        println!("pickup {:?}", e);
                        if let Some(player_entity) = lobby.clients.get(&client_id) {
                            //commands.entity(*player_entity).insert(tile);
                            //for (player, mut inventory) in players.iter_mut() {
                            //if *player_entity == player {
                            //for (item, item_id) in item_query.iter() {
                            //if e == item {
                            //let inventory_item = commands.spawn(*item_id).id();
                            //inventory.slots.insert(inventory_item);
                            //println!("inserted into inventory");
                            //let message = bincode::serialize(item_id).unwrap();
                            //server.send_message(
                            //client_id,
                            //ServerChannel::Test,
                            //message,
                            //);
                            //}
                            //}
                            //}
                            //}
                            commands.entity(e).despawn_recursive();
                            println!("pickup {:?}", e);

                            let despawn_message = bincode::serialize(&e).unwrap();
                            server.broadcast_message(ServerChannel::Despawn, despawn_message);
                        }
                    }
                    _ => (),
                },
            }
        }
    }
}
pub fn receive_clicks(lobby: Res<ServerLobby>, mut server: ResMut<RenetServer>) {
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
