use std::time::Duration;

use bevy::{
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    prelude::*,
};
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use bevy_inspector_egui::{prelude::*, quick::WorldInspectorPlugin};
use bevy_renet::{renet::RenetClient, RenetClientPlugin};
use camera::{camera_follow, setup_camera};
use connection::{new_renet_client, server_messages};
use rand::Rng;
use resources::{ClientLobby, NetworkMapping};
use shared::{
    channels::ServerChannel,
    components::{ControlledEntity, EntityType, Player, TilePos},
};
pub mod camera;
pub mod components;
pub mod connection;
pub mod plugins;
pub mod resources;
pub mod run_conditions;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugin(RenetClientPlugin {
        clear_events: false,
    });

    app.insert_resource(FixedTime::new(Duration::from_millis(100)));
    app.edit_schedule(CoreSchedule::Main, |schedule| {
        schedule.set_build_settings(ScheduleBuildSettings {
            ambiguity_detection: LogLevel::Ignore,
            ..default()
        });
    });
    app.add_plugin(WorldInspectorPlugin::default());

    //app.add_system(fixed_time.in_schedule(CoreSchedule::FixedUpdate));
    app.add_startup_system(setup_camera);
    app.add_system(server_messages);
    app.add_system(camera_follow);
    app.add_system(load);
    app.insert_resource(new_renet_client());
    app.add_systems(().distributive_run_if(bevy_renet::client_connected));
    app.insert_resource(NetworkMapping::default());
    app.insert_resource(ClientLobby::default());
    app.add_system(despawn);
    //app.add_system(run_fixed_update_schedule);
    app.run();
}

pub fn load(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut client: ResMut<RenetClient>,
    mut commands: Commands,
    mut network_mapping: ResMut<NetworkMapping>,
) {
    if let Some(message) = client.receive_message(ServerChannel::Load) {
        let scope: Vec<(Entity, EntityType, TilePos)> = bincode::deserialize(&message).unwrap();

        for (e, entity_type, t) in scope {
            //println!("tile: {:?}", t);
            match entity_type {
                EntityType::Tile => {
                    let mut rng = rand::thread_rng();
                    let color: f32 = rng.gen_range(0.9..1.0);
                    let transform = t.to_transform();
                    let new_player = commands
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
                            //LeftClick::default(),
                        ))
                        //.forward_events::<PointerDown, PickingEvent>()
                        .id();
                    network_mapping.client_to_server.insert(new_player, e);
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
                    if player.id == client.client_id() {
                        commands.entity(new_player).insert(ControlledEntity);
                    }
                    commands.entity(new_player).insert(player);
                    network_mapping.client_to_server.insert(new_player, e);
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
                network_mapping.client_to_server.remove_entry(&e);
            });
    }
}
