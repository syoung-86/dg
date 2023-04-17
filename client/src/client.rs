use std::time::Duration;

use bevy::{
    ecs::{
        entity::MapEntities,
        schedule::{LogLevel, ScheduleBuildSettings},
        system::SystemParam,
    },
    input::mouse,
    prelude::*,
};
use bevy_mod_picking::{DefaultPickingPlugins, PickableBundle, PickableMesh, PickingEvent};

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_renet::{renet::RenetClient, RenetClientPlugin};
use camera::{camera_follow, setup_camera};
use connection::{new_renet_client, server_messages};
use leafwing_input_manager::prelude::*;
use lib::{
    channels::{ClientChannel, ServerChannel},
    components::{
        ComponentType, ControlledEntity, EntityType, LeftClick, Path, Player, PlayerCommand, Tile,
    },
    resources::Tick,
    ClickEvent,
};
use movement::{client_send_player_commands, get_path, scheduled_movement};
use rand::Rng;
use receive::{despawn_message, load_message, spawn_message, tick, update_message};
use resources::{ClientLobby, NetworkMapping};
use serde::{Deserialize, Serialize};
use smooth_bevy_cameras::{
    controllers::{orbit::OrbitCameraPlugin, unreal::UnrealCameraPlugin},
    LookTransformPlugin,
};

use crate::resources::ClientInfo;
pub mod camera;
pub mod components;
pub mod connection;
pub mod movement;
pub mod plugins;
pub mod receive;
pub mod resources;
pub mod run_conditions;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum Move {
    North,
    South,
    West,
    East,
}
fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugin(RenetClientPlugin {
        clear_events: false,
    });

    app.add_plugin(InputManagerPlugin::<Move>::default());
    app.add_plugins(DefaultPickingPlugins);
    app.insert_resource(FixedTime::new(Duration::from_millis(100)));
    app.insert_resource(Tick::default());
    app.edit_schedule(CoreSchedule::Main, |schedule| {
        schedule.set_build_settings(ScheduleBuildSettings {
            ambiguity_detection: LogLevel::Ignore,
            ..default()
        });
    });
    app.add_plugin(WorldInspectorPlugin::default());
    app.add_plugin(LookTransformPlugin);
    //app.add_plugin(UnrealCameraPlugin::default());

    app.add_startup_system(setup_camera);
    app.add_system(server_messages);
    app.add_system(camera_follow);
    //app.add_system(load);
    app.insert_resource(new_renet_client());
    app.insert_resource(NetworkMapping::default());
    app.insert_resource(ClientLobby::default());
    //app.add_system(despawn);
    app.add_system(get_path);
    app.add_system(scheduled_movement);
    app.add_system(make_pickable);
    app.add_system(mouse_input);
    app.add_system(tick);
    app.add_system(load_message);
    app.add_system(spawn_message);
    app.add_system(update_message);
    app.add_system(despawn_message);
    app.add_system(spawn);
    app.add_system(update);
    app.add_system(client_send_player_commands);
    app.add_event::<ClickEvent>();
    app.add_event::<PlayerCommand>();
    app.add_event::<SpawnEvent>();
    app.add_event::<DespawnEvent>();
    app.add_event::<UpdateEvent>();
    app.add_event::<TickEvent>();
    app.run();
}

pub struct SpawnEvent {
    entity: Entity,
    entity_type: EntityType,
    tile: Tile,
}

pub struct DespawnEvent(Entity);
pub struct UpdateEvent {
    entity: Entity,
    component: ComponentType,
}
pub struct TickEvent(Tick);

pub fn update(
    mut commands: Commands,
    mut update_event: EventReader<UpdateEvent>,
    //mut client: ResMut<RenetClient>,
) {
    for event in update_event.iter() {
        println!("Received Update Event");
        match event.component {
            ComponentType::Tile(t) => commands.entity(event.entity).insert((t, t.to_transform())),
            ComponentType::Player(c) => commands.entity(event.entity).insert(c),
        };
    }
}
pub fn spawn(
    mut commands: Commands,
    mut spawn_event: EventReader<SpawnEvent>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    client: Res<RenetClient>,
) {
    for event in spawn_event.iter() {
        //println!(
        //"spawn e: {:?}, type: {:?}, components: {:?}",
        //event.entity, event.entity_type, event.tile
        //);
        match event.entity_type {
            EntityType::Tile => {
                let transform = event.tile.to_transform();
                commands.entity(event.entity).insert((
                    PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Box::new(1., 0.2, 1.))),
                        material: materials.add(Color::rgb(0.2, 0.5, 0.2).into()),
                        transform,
                        ..Default::default()
                    },
                    //PickableBundle::default(),
                    //PickRaycastTarget::default(),
                    //NoDeselect,
                    event.tile,
                    LeftClick::Walk,
                ));
                //.forward_events::<PointerDown, PickingEvent>()
            }
            EntityType::Player(player) => {
                let transform = event.tile.to_transform();
                commands.entity(event.entity).insert((
                    PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Capsule {
                            rings: 10,
                            ..default()
                        })),
                        material: materials.add(Color::rgb(0.8, 0.8, 0.8).into()),
                        transform,
                        ..Default::default()
                    },
                    //PickableBundle::default(),
                    //PickRaycastTarget::default(),
                    //NoDeselect,
                    event.tile,
                    //LeftClick::default(),
                ));
                //.forward_events::<PointerDown, PickingEvent>()
                //

                println!("spawn player: {:?}", player);
                if player.id == client.client_id() {
                    commands.entity(event.entity).insert(ControlledEntity);
                }

                commands.spawn(PointLightBundle {
                    point_light: PointLight {
                        intensity: 1500.0,
                        shadows_enabled: true,
                        ..Default::default()
                    },
                    transform: Transform::from_xyz(4.0, 8.0, 4.0),
                    ..Default::default()
                });
            }
        }
    }
}

pub fn mouse_input(
    mut click_event: EventWriter<ClickEvent>,
    mut events: EventReader<PickingEvent>,
    query: Query<(Entity, &LeftClick, &Tile)>,
) {
    for event in events.iter() {
        match event {
            PickingEvent::Clicked(clicked_entity) => {
                query
                    .iter()
                    .filter(|(entity, _action, _)| entity == clicked_entity)
                    .for_each(|(entity, action, tile)| {
                        click_event.send(ClickEvent {
                            target: entity,
                            left_click: *action,
                            destination: *tile,
                        })
                    });
            }
            _ => (),
        }
    }
}
fn make_pickable(
    mut commands: Commands,
    meshes: Query<Entity, (With<Handle<Mesh>>, Without<PickableMesh>)>,
) {
    for entity in meshes.iter() {
        commands.entity(entity).insert((PickableBundle::default(),));
    }
}
