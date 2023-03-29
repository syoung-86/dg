use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum Direction {
    Bad,
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

#[derive(Default, Debug, Serialize, Deserialize, Component)]
pub struct Path {
    pub destination: Tile,
    pub origin: Tile,
    pub left_click: LeftClick,
}

impl Path {
    pub fn step(&mut self) {
        let mut direction = Direction::Bad;
        if self.origin.cell.0 < self.destination.cell.0
            && self.origin.cell.2 == self.destination.cell.2
        {
            direction = Direction::North;
        }

        if self.origin.cell.0 > self.destination.cell.0
            && self.origin.cell.2 == self.destination.cell.2
        {
            direction = Direction::South;
        }

        if self.origin.cell.0 == self.destination.cell.0
            && self.origin.cell.2 > self.destination.cell.2
        {
            direction = Direction::West;
        }
        if self.origin.cell.0 == self.destination.cell.0
            && self.origin.cell.2 < self.destination.cell.2
        {
            direction = Direction::East;
        }

        if self.origin.cell.0 < self.destination.cell.0
            && self.origin.cell.2 < self.destination.cell.2
        {
            direction = Direction::NorthEast;
        }

        if self.origin.cell.0 < self.destination.cell.0
            && self.origin.cell.2 > self.destination.cell.2
        {
            direction = Direction::NorthWest;
        }

        if self.origin.cell.0 > self.destination.cell.0
            && self.origin.cell.2 > self.destination.cell.2
        {
            direction = Direction::SouthWest;
        }

        if self.origin.cell.0 > self.destination.cell.0
            && self.origin.cell.2 < self.destination.cell.2
        {
            direction = Direction::SouthEast;
        }
        println!("Direction: {:?}", direction);
        match direction {
            Direction::North => self.origin.cell.0 += 1,
            Direction::East => self.origin.cell.2 += 1,
            Direction::South => self.origin.cell.0 -= 1,
            Direction::West => self.origin.cell.2 -= 1,
            Direction::NorthEast => {
                self.origin.cell.0 += 1;
                self.origin.cell.2 += 1
            }
            Direction::SouthEast => {
                self.origin.cell.0 -= 1;
                self.origin.cell.2 += 1
            }
            Direction::SouthWest => {
                self.origin.cell.0 -= 1;
                self.origin.cell.2 -= 1
            }
            Direction::NorthWest => {
                self.origin.cell.0 += 1;
                self.origin.cell.2 -= 1
            }
            Direction::Bad => (),
        }
    }
}
#[derive(Eq, PartialEq, Debug, Clone, Copy, Serialize, Deserialize, Component)]
pub enum EntityType {
    Tile,
    Player(Player),
}
#[derive(Component)]
pub struct ControlledEntity;

#[derive(Eq, PartialEq, Copy, Clone, Default, Debug, Serialize, Deserialize, Component)]
pub enum LeftClick {
    #[default]
    Walk,
    Attack,
    Pickup(Entity),
}

#[derive(Debug, Serialize, Deserialize, Component)]
pub enum ComponentType {
    Tile(Tile),
    Player(Player),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntityBox {
    entity: Entity,
    components: Vec<ComponentType>,
    tile_pos: Tile,
}
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
#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Component, Default, Copy, Clone)]
pub struct Tile {
    pub cell: (u32, u32, u32),
}

impl Tile {
    pub fn to_transform(&self) -> Transform {
        let mut transform = Vec3::new(0.0, 0.0, 0.0);
        transform[0] = self.cell.0 as f32;
        transform[1] = self.cell.1 as f32;
        transform[2] = self.cell.2 as f32;
        Transform::from_xyz(transform[0], transform[1], transform[2])
    }
    pub fn from_xyz(translation: &Vec3) -> Tile {
        let mut new_tile = Tile::default();
        new_tile.cell.0 = translation[0] as u32;
        new_tile.cell.1 = translation[1] as u32;
        new_tile.cell.2 = translation[2] as u32;
        new_tile
    }
}

#[derive(Serialize, Deserialize, Component)]
pub struct Instance;

#[derive(Clone, Copy, Serialize, Deserialize, Component, Default, Debug)]
pub struct Scope {
    pub top_left: Tile,
    pub bottom_right: Tile,
    pub up: Tile,
    pub down: Tile,
}

impl Scope {
    pub fn get(start: Tile) -> Scope {
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

    pub fn check(&self, pos: Tile) -> bool {
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
