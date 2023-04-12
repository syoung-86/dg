use bevy::prelude::*;
use serde::{Deserialize, Serialize};
#[derive(
    Reflect, Eq, PartialEq, Debug, Serialize, Deserialize, Component, Default, Copy, Clone,
)]
#[reflect(Component)]
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
