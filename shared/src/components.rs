use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Component)]
pub struct ControlledEntity;

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ServerMessages {
    PlayerConnected { id: u64 },
    PlayerDisconnected { id: u64 },
}

#[derive(Default, Serialize, Deserialize, Component, Debug)]
pub struct Client {
    pub id: u64,
    pub scope: Scope,
}
#[derive(Debug, Serialize, Deserialize, Component, Default, Copy, Clone)]
pub struct TilePos {
    pub cell: (u32, u32, u32),
}

#[derive(Serialize, Deserialize, Component)]
pub struct Tile;

#[derive(Serialize, Deserialize, Component)]
pub struct Instance;

#[derive(Serialize, Deserialize, Component, Default, Debug)]
pub struct Scope {
    pub top_left: TilePos,
    pub bottom_right: TilePos,
    pub up: TilePos,
    pub down: TilePos,
}

impl Scope {
    pub fn get(start: TilePos) -> Scope {
        let mut scope = Scope::default();
        let mut top_left = start;
        let mut bottom_right = start;
        let mut up = start;
        let mut down = start;
        top_left.cell.0 += 20;
        top_left.cell.2 += 20;

        if bottom_right.cell.0 > 20 {
            bottom_right.cell.0 -= 20;
        } else {
            bottom_right.cell.0 = 0;
        }

        if bottom_right.cell.2 > 20 {
            bottom_right.cell.2 -= 20;
        } else {
            bottom_right.cell.2 = 0;
        }
        up.cell.1 += 1;
        if down.cell.1 > 0 {
            down.cell.1 -= 1;
        } else {
            down.cell.1 = 0;
        }

        scope.top_left = top_left;
        scope.bottom_right = bottom_right;

        scope.up = up;
        scope.down = down;

        scope
    }
}

#[derive(Serialize, Deserialize, Component)]
pub struct Player;
