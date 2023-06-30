use bevy::gltf::Gltf;
use bevy::prelude::*;

use crate::Animations;
/// Helper resource for tracking our asset
#[derive(Resource)]
pub struct ManAssetPack(pub Handle<Gltf>);

impl FromWorld for ManAssetPack {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.get_resource::<AssetServer>().unwrap();
        let gltf = asset_server.load("rpg_man.glb");
        ManAssetPack(gltf)
    }
}

#[derive(Default, Resource)]
pub struct ShouldLoadAnims(pub bool);

pub fn should_load_anims(should_load_anims: Res<ShouldLoadAnims>) -> bool {
    should_load_anims.0
}

pub fn load_anims(
    mut commands: Commands,
    man_scene: Res<ManAssetPack>,
    assets_gltf: Res<Assets<Gltf>>,
) {
    if let Some(gltf) = assets_gltf.get(&man_scene.0) {
        println!("found scene");
        let mut animations = Animations::default();
        for animation in gltf.animations.iter() {
            let cloned = animation.clone();
            animations.0.push(cloned);
            println!("added an anim");
        }
        commands.insert_resource(animations);
        commands.insert_resource(ShouldLoadAnims(false));
    }
}
