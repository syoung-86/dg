use bevy::{gltf::Gltf, prelude::*};
use bevy_mod_picking::prelude::{Down, OnPointer};
use lib::components::{Health, HealthBar, LeftClick, Slime};

use crate::input::picking_listener;

use super::{
    anims::SlimeAnimations,
    extra::{LoadedSlime, SlimeAssetPack, SpawnSlimeEvent},
};

pub fn spawn_slime(
    mut commands: Commands,
    assets: Res<Assets<Gltf>>,
    slime_scene: Res<SlimeAssetPack>,
    mut loaded: ResMut<LoadedSlime>,
    mut events: EventReader<SpawnSlimeEvent>,
) {
    for event in events.iter() {
        if let Some(gltf) = assets.get(&slime_scene.0) {
            commands.entity(event.entity).insert((
                SceneBundle {
                    scene: gltf.named_scenes.get("Scene").unwrap().clone(),
                    transform: event.tile.to_transform(),
                    ..Default::default()
                },
                LeftClick::Attack(event.entity),
                event.tile,
                Health::new(99),
                Slime,
                OnPointer::<Down>::run_callback(picking_listener),
            ));
            let mut animations = SlimeAnimations::default();
            for animation in gltf.animations.iter() {
                let cloned = animation.clone();
                animations.0.push(cloned);
            }
            let hp_bar = commands.spawn((HealthBar,)).id();
            commands.entity(event.entity).push_children(&[hp_bar]);
            commands.insert_resource(animations);
            loaded.0 = false;
        }
    }
}
