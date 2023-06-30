use bevy::prelude::*;
use lib::components::{Tile, Untraversable, OpenState};
pub struct InsertUntraversableEvent(Tile);
pub fn update_trav(
    tiles: Query<(Entity, &Tile), (Without<Untraversable>, Without<OpenState>)>,
    mut events: EventReader<InsertUntraversableEvent>,
    mut commands: Commands,
) {
    for event in events.iter() {
        for (e, tile) in tiles.iter() {
            if tile.cell == event.0.cell {
                commands.entity(e).insert(Untraversable);
            }
        }
    }
}
