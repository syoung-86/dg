use std::time::Duration;

use bevy::{
    ecs::{
        entity::MapEntities,
        schedule::{LogLevel, ScheduleBuildSettings},
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
    app.add_system(receive_tick);
    app.add_system(receive_message);
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
pub fn receive_message(
    mut client: ResMut<RenetClient>,
    mut spawn_event: EventWriter<SpawnEvent>,
    //mut despawn_event: EventWriter<DespawnEvent>,
    mut update_event: EventWriter<UpdateEvent>,
    //mut tick_event: EventWriter<TickEvent>,
    mut network_mapping: ResMut<NetworkMapping>,
    mut commands: Commands,
) {
    if let Some(message) = client.receive_message(ServerChannel::Load) {
        println!("received load message");
        let load_message: Vec<(Entity, EntityType, Tile)> = bincode::deserialize(&message).unwrap();
        for (server_entity, entity_type, tile) in load_message {
            if let None = network_mapping.server.get(&server_entity) {
                let entity = commands.spawn_empty().id();
                network_mapping.client.insert(entity, server_entity);
                network_mapping.server.insert(server_entity, entity);
                spawn_event.send(SpawnEvent {
                    entity,
                    entity_type,
                    tile,
                });
            }
        }
    }

    if let Some(message) = client.receive_message(ServerChannel::Spawn) {
        let (server_entity, entity_type, tile): (Entity, EntityType, Tile) =
            bincode::deserialize(&message).unwrap();
        if let None = network_mapping.server.get(&server_entity) {
            let entity = commands.spawn_empty().id();
            network_mapping.client.insert(entity, server_entity);
            network_mapping.server.insert(server_entity, entity);
            spawn_event.send(SpawnEvent {
                entity,
                entity_type,
                tile,
            });
        }
    }

    if let Some(message) = client.receive_message(ServerChannel::Update) {
        println!("Received Update Message!");
        let (server_entity, component): (Entity, ComponentType) =
            bincode::deserialize(&message).unwrap();
        if let Some(entity) = network_mapping.server.get(&server_entity) {
            update_event.send(UpdateEvent {
                entity: *entity,
                component,
            });
        }
    }

    if let Some(message) = client.receive_message(ServerChannel::Despawn) {
        let despawn_entity: Entity = bincode::deserialize(&message).unwrap();
        if let Some(entity) = network_mapping.server.remove(&despawn_entity) {
            commands.entity(entity).despawn_recursive();
        }
    }

    //if let Some(message) = client.receive_message(ServerChannel::Tick) {
    //let new_tick: Tick = bincode::deserialize(&message).unwrap();
    //tick_event.send(TickEvent(new_tick));
    //tick.tick = new_tick.tick;
    //}
}

pub fn update(
    mut commands: Commands,
    mut update_event: EventReader<UpdateEvent>,
    mut client: ResMut<RenetClient>,
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
    mut client: ResMut<RenetClient>,
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
pub fn receive_tick(mut client: ResMut<RenetClient>, mut tick: ResMut<Tick>) {
    if let Some(message) = client.receive_message(ServerChannel::Tick) {
        let new_tick: Tick = bincode::deserialize(&message).unwrap();
        tick.tick = new_tick.tick;
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

pub fn load(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut client: ResMut<RenetClient>,
    mut commands: Commands,
    mut network_mapping: ResMut<NetworkMapping>,
    mut lobby: ResMut<ClientLobby>,
) {
    if let Some(message) = client.receive_message(ServerChannel::Load) {
        let scope: Vec<(Entity, EntityType, Tile)> = bincode::deserialize(&message).unwrap();

        println!("scope");
        for (e, entity_type, t) in scope {
            //println!("tile: {:?}", t);
            match entity_type {
                EntityType::Tile => {
                    let mut rng = rand::thread_rng();
                    let color: f32 = rng.gen_range(0.4..0.6);
                    let transform = t.to_transform();
                    let new_tile = commands
                        .spawn((
                            PbrBundle {
                                mesh: meshes.add(Mesh::from(shape::Box::new(1., 0.2, 1.))),
                                material: materials.add(Color::rgb(0.2, 0.5, 0.2).into()),
                                transform,
                                ..Default::default()
                            },
                            //PickableBundle::default(),
                            //PickRaycastTarget::default(),
                            //NoDeselect,
                            t,
                            LeftClick::Walk,
                        ))
                        //.forward_events::<PointerDown, PickingEvent>()
                        .id();
                    network_mapping.client.insert(new_tile, e);
                    network_mapping.server.insert(e, new_tile);
                }
                EntityType::Player(player) => {
                    let transform = t.to_transform();
                    let new_player = commands
                        .spawn((
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
                            t,
                            //LeftClick::default(),
                        ))
                        //.forward_events::<PointerDown, PickingEvent>()
                        .id();
                    let new_client_info = ClientInfo {
                        client_entity: Some(new_player),
                        server_entity: Some(e),
                        controlled_entity: Some(new_player),
                    };
                    lobby.clients.insert(player.id, new_client_info);
                    if player.id == client.client_id() {
                        commands.entity(new_player).insert((
                            ControlledEntity,
                            InputManagerBundle::<Move> {
                                action_state: ActionState::default(),
                                input_map: InputMap::new([
                                    (KeyCode::W, Move::North),
                                    (KeyCode::S, Move::South),
                                    (KeyCode::D, Move::East),
                                    (KeyCode::A, Move::West),
                                ]),
                            },
                        ));
                    }
                    commands.entity(new_player).insert(player);
                    network_mapping.client.insert(new_player, e);
                    network_mapping.server.insert(e, new_player);
                    println!("server e: {:?}, client e: {:?}", e, new_player);
                }
            }
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

pub fn despawn(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    query: Query<(Entity, &Player)>,
    mut network_mapping: ResMut<NetworkMapping>,
) {
    if let Some(message) = client.receive_message(ServerChannel::Despawn) {
        let old_client: u64 = bincode::deserialize(&message).unwrap();
        query
            .iter()
            .filter(|(_e, player)| player.id == old_client)
            .for_each(|(e, _player)| {
                commands.entity(e).despawn();
                network_mapping.client.remove_entry(&e);
            });
    }
}
