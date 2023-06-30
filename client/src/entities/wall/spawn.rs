use bevy::{gltf::Gltf, prelude::*};

use super::{assets::WallAssetPack, extra::SpawnWallEvent};
pub fn dg_wall(
    mut commands: Commands,
    assets: Res<Assets<Gltf>>,
    cube_scene: Res<WallAssetPack>,
    mut spawn_wall_event: EventReader<SpawnWallEvent>,
) {
    for event in spawn_wall_event.iter() {
        if let Some(gltf) = assets.get(&cube_scene.0) {
            commands.spawn((
                SceneBundle {
                    scene: gltf.named_scenes.get("Scene").unwrap().clone(),
                    transform: event.tile.to_transform(),
                    ..Default::default()
                },
                event.wall,
                event.tile,
            ));
        } else {
            loop {
                if let Some(gltf) = assets.get(&cube_scene.0) {
                    commands.spawn((
                        SceneBundle {
                            scene: gltf.named_scenes.get("Scene").unwrap().clone(),
                            transform: event.tile.to_transform(),
                            ..Default::default()
                        },
                        event.wall,
                        event.tile,
                    ));
                }
            }
        }
    }
}
