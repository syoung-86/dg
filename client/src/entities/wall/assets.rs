use bevy::{prelude::*, gltf::Gltf};

#[derive(Resource)]
pub struct WallAssetPack(pub Handle<Gltf>);

impl FromWorld for WallAssetPack {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let gltf = asset_server.load("wall_cube.glb");
        WallAssetPack(gltf)
    }
}
