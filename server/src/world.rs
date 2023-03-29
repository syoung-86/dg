use bevy::prelude::*;
use lib::components::{Client, EntityType, Instance, Player, Scope, Tile};

pub fn create_tiles(mut commands: Commands) {
    let instance = commands.spawn(Instance).id();
    let y: u32 = 0;
    for x in (0..100).step_by(10) {
        for z in (0..100).step_by(10) {
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
            tiles.push(
                commands
                    .spawn((EntityType::Tile, Tile { cell: (x, y, z) }))
                    .id(),
            );
        }
    }
    tiles
}
