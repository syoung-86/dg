use std::f32::consts::{FRAC_PI_2, FRAC_PI_3, PI};

use bevy::{gltf::Gltf, prelude::*};
use bevy_easings::*;
use bevy_mod_picking::prelude::*;
use bevy_renet::renet::RenetClient;
use lib::components::{
    ComponentType, ControlledEntity, Door, EntityType, Health, HealthBar, LeftClick, SpawnEvent,
    Sword, Target, Tile, UpdateEvent, Wall,
};

use crate::{assets::ManAssetPack, resources::NetworkMapping, PlayerBundle};

pub fn update(
    mut commands: Commands,
    mut update_event: EventReader<UpdateEvent>,
    //mut client: ResMut<RenetClient>,
    query: Query<(Entity, &Transform, &Tile)>,
    network_mapping: Res<NetworkMapping>,
) {
    for event in update_event.iter() {
        //println!("Received Update Event");
        match event.component {
            ComponentType::Tile(t) => {
                for (e, old_transform, old_tile) in query.iter() {
                    if e == event.entity {
                        let mut transform = t.to_transform();
                        //let old_transform = old_tile.to_transform();
                        let mut rotation = 0.;
                        if old_tile.cell.0 > t.cell.0 {
                            rotation = -FRAC_PI_2;
                            //println!("WEST");
                        } else if old_tile.cell.0 < t.cell.0 {
                            //rotation = 1.5;

                            rotation = FRAC_PI_2;
                            //println!("EAST");
                        }
                        if old_tile.cell.2 > t.cell.2 {
                            rotation = -PI;
                            //println!("NORTH");
                        } else if old_tile.cell.2 < t.cell.2 {
                            rotation = 0.0;
                            //println!("SOUTH");
                        }

                        //println!("old_tile: {:?}, new: {:?}", old_tile, t);

                        if old_tile.cell.0 < t.cell.0 && old_tile.cell.2 > t.cell.2 {
                            //println!("NORTH EAST");
                            rotation = 2.2;
                        }

                        if old_tile.cell.0 > t.cell.0 && old_tile.cell.2 > t.cell.2 {
                            rotation = -2.2;

                            //println!("NORTH WEST");
                        }

                        if old_tile.cell.0 < t.cell.0 && old_tile.cell.2 < t.cell.2 {
                            rotation = FRAC_PI_3;
                            //println!("SOUTH EAST");
                        }
                        if old_tile.cell.0 > t.cell.0 && old_tile.cell.2 < t.cell.2 {
                            //println!("SOUTH WEST");
                            rotation = -FRAC_PI_3;
                        }
                        transform.rotate_y(rotation);
                        commands.entity(event.entity).insert(old_transform.ease_to(
                            transform,
                            bevy_easings::EaseFunction::QuadraticOut,
                            bevy_easings::EasingType::Once {
                                duration: std::time::Duration::from_millis(300),
                            },
                        ));
                        commands.entity(event.entity).insert(t);
                    }
                    // else {
                    //commands.entity(event.entity).insert((t, t.to_transform()));
                    //}
                }
            }
            ComponentType::Player(c) => {
                commands.entity(event.entity).insert(c);
            }

            ComponentType::Open(c) => {
                commands.entity(event.entity).insert(c);
            }
            ComponentType::Health(c) => {
                commands.entity(event.entity).insert(c);
            }
            ComponentType::Running(c) => {
                commands.entity(event.entity).insert(c);
            }
            ComponentType::Target(c) => {
                if let Some(client_entity) = c.0 {
                    if let Some(client_entity) = network_mapping.server.get(&client_entity) {
                        commands
                            .entity(event.entity)
                            .insert(Target(Some(*client_entity)));
                        println!("inserted target");
                    }
                }
            }
        };
    }
}

pub fn spawn(
    mut commands: Commands,
    mut spawn_event: EventReader<SpawnEvent>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    client: Res<RenetClient>,
    man_scene: Res<ManAssetPack>,
    assets: Res<Assets<Gltf>>,
) {
    for event in spawn_event.iter() {
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
                    event.tile,
                    //PickableBundle::default(),
                    //RaycastPickTarget::default(),
                    //NoDeselect,
                    LeftClick::Walk,
                ));
                //.forward_events::<PointerDown, PickingEvent>()
            }
            EntityType::Player(player) => {
                println!("event.tile:{:?}", event.tile);
                let transform = event.tile.to_transform();
                if let Some(gltf) = assets.get(&man_scene.0) {
                    commands.entity(event.entity).insert((
                        SceneBundle {
                            scene: gltf.scenes[0].clone(),
                            transform,
                            ..Default::default()
                        },
                        PlayerBundle::new(&event.tile),
                        player,
                        Health::new(50),
                    ));
                    let mut transform = Transform::from_xyz(0., 3., 0.);
                    transform.rotate_z(FRAC_PI_2);
                    let hp_bar = commands.spawn((HealthBar,)).id();
                    commands.entity(event.entity).push_children(&[hp_bar]);
                } //.forward_events::<PointerDown, PickingEvent>()
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
            EntityType::Sword(_sword) => {
                commands.spawn(Sword);
                println!("spawned sword");
            }
            EntityType::Wall(wall) => match wall {
                Wall::Horizontal => {
                    commands.entity(event.entity).insert((
                        wall,
                        event.tile,
                        PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Box::new(1.0, 5., 0.2))),
                            material: materials.add(Color::rgb(1.0, 0.5, 0.2).into()),
                            transform: event.tile.to_transform(),
                            ..Default::default()
                        },
                    ));
                }
                Wall::Vertical => {
                    commands.entity(event.entity).insert((
                        wall,
                        event.tile,
                        PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Box::new(0.2, 5., 1.0))),
                            material: materials.add(Color::rgb(1.0, 0.5, 0.2).into()),
                            transform: event.tile.to_transform(),
                            ..Default::default()
                        },
                    ));
                }
            },
            EntityType::Door(door) => match door {
                Door::Vertical => {
                    commands.entity(event.entity).insert((
                        door,
                        event.tile,
                        PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Box::new(0.2, 5., 2.0))),
                            material: materials.add(Color::rgb(2.0, 0.5, 0.8).into()),
                            transform: event.tile.to_transform(),
                            ..Default::default()
                        },
                    ));
                }
                _ => (),
            },
            EntityType::Lever(lever) => {
                commands.entity(event.entity).insert((
                    lever,
                    event.tile,
                    PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Box::new(1.0, 1.0, 1.0))),
                        material: materials.add(Color::rgb(1.0, 0.2, 0.0).into()),
                        transform: event.tile.to_transform(),
                        ..Default::default()
                    },
                    LeftClick::Pull,
                ));
            }
            EntityType::Dummy(dummy) => {
                commands.entity(event.entity).insert((
                    dummy,
                    event.tile,
                    PbrBundle {
                        mesh: meshes.add(Mesh::from(shape::Capsule::default())),
                        material: materials.add(Color::rgb(0.2, 0.2, 0.2).into()),
                        transform: event.tile.to_transform(),
                        ..Default::default()
                    },
                    LeftClick::Attack(event.entity),
                    Health::new(99),
                ));
                let hp_bar = commands.spawn((HealthBar,)).id();
                commands.entity(event.entity).push_children(&[hp_bar]);
            }
        }
    }
}
