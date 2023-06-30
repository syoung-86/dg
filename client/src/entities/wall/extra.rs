use lib::components::{Wall, Tile};

pub struct SpawnWallEvent {
    pub wall: Wall,
    pub tile: Tile,
}
