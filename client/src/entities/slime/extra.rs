use bevy::{gltf::Gltf, prelude::*};
use lib::components::{Slime, Tile};

pub struct SpawnSlimeEvent {
    pub slime: Slime,
    pub tile: Tile,
    pub entity: Entity,
}

//#[bevycheck::system]
#[derive(Resource)]
pub struct LoadedSlime(pub bool);

pub fn loaded_slime(loaded_slime: Res<LoadedSlime>) -> bool {
    loaded_slime.0
}

#[derive(Resource)]
pub struct SlimeAssetPack(pub Handle<Gltf>);

impl FromWorld for SlimeAssetPack {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let gltf = asset_server.load("slime.glb");
        SlimeAssetPack(gltf)
    }
}

