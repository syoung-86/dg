use bevy::prelude::*;
use shared::components::{Client, Instance, Player, Scope, Tile, TilePos};

use crate::{events::ClientSetup, resources::ServerLobby};

pub fn create_tiles(mut commands: Commands) {
    let y: u32 = 0;
    for x in (0..100).step_by(10) {
        for z in (0..100).step_by(10) {
            let instance = commands.spawn(Instance).id();
            let tiles = spawn_chunk(&mut commands, (x, y, z));
            for child in tiles {
                commands.entity(instance).push_children(&[child]);
            }
        }
    }
}

pub fn spawn_chunk(commands: &mut Commands, start: (u32, u32, u32)) -> Vec<Entity> {
    let y: u32 = 0;
    let end = (start.0 + 10, start.1, start.2 + 10);
    let mut tiles = vec![];
    for x in start.0..end.0 {
        for z in start.2..end.2 {
            //println!("cell: {} {} {}", x, y, z);
            tiles.push(commands.spawn((Tile, TilePos { cell: (x, y, z) })).id());
        }
    }
    tiles
}
//#[bevycheck::system]
pub fn client_setup(
    mut commands: Commands,
    server_lobby: Res<ServerLobby>,
    mut events: EventReader<ClientSetup>,
    mut clients: Query<&mut Client>,
) {
    println!("event");
    for event in events.iter() {
        println!("event");
        clients
            .iter_mut()
            //.filter(|client| client.id == event.0)
            .for_each(|mut client| {
                client.scope = Scope::get(TilePos { cell: (4, 0, 4) });
                println!("client scope:{:?}", client.scope);
            });
        if let Some(client_entity) = server_lobby.clients.get(&event.0) {
            commands
                .entity(*client_entity)
                .insert((Player, TilePos { cell: (4, 0, 4) }));
        }
    }
}
