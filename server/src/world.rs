use bevy::prelude::*;
use lib::components::{EntityType, Instance, Tile, Wall};

pub fn create_tiles(mut commands: Commands) {
    let instance = commands.spawn(Instance).id();
    let y: u32 = 0;
    for x in (0..20).step_by(10) {
        for z in (0..20).step_by(10) {
            let tiles = spawn_chunk(&mut commands, (x, y, z));
            for child in tiles {
                commands.entity(instance).push_children(&[child]);
            }
        }
    }
    for z in 4..10 {
        commands.spawn((EntityType::Wall(Wall::Vertical), Tile::new((4, 0, z))));
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
