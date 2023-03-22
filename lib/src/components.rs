use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Debug, Clone, Copy, Serialize, Deserialize, Component)]
pub enum EntityType {
    Tile,
    Player(Player),
}
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

impl TilePos {
    pub fn to_transform(&self) -> Transform {
        let mut transform = Vec3::new(0.0, 0.0, 0.0);
        transform[0] = self.cell.0 as f32;
        transform[1] = self.cell.1 as f32;
        transform[2] = self.cell.2 as f32;
        Transform::from_xyz(transform[0], transform[1], transform[2])
    }
    pub fn from_xyz(translation: &Vec3) -> TilePos {
        let mut new_tile = TilePos::default();
        new_tile.cell.0 = translation[0] as u32;
        new_tile.cell.1 = translation[1] as u32;
        new_tile.cell.2 = translation[2] as u32;
        new_tile
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy, Serialize, Deserialize, Component)]
pub struct Tile;

#[derive(Serialize, Deserialize, Component)]
pub struct Instance;

#[derive(Clone, Copy, Serialize, Deserialize, Component, Default, Debug)]
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

    pub fn check(&self, pos: TilePos) -> bool {
        let x = pos.cell.0;
        let z = pos.cell.2;

        let tl_x = self.top_left.cell.0;
        let tl_z = self.top_left.cell.2;

        let br_x = self.bottom_right.cell.0;
        let br_z = self.bottom_right.cell.2;

        x <= tl_x && x >= br_x && z <= tl_z && z >= br_z
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Component)]
pub struct Player {
    pub id: u64,
}
