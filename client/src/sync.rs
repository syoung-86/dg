use std::f32::consts::{FRAC_PI_2, FRAC_PI_3, PI};

use bevy::{gltf::Gltf, prelude::*};
use bevy_easings::*;
use bevy_mod_picking::prelude::*;
use bevy_renet::renet::RenetClient;
use bevy_scene_hook::{HookedSceneBundle, SceneHook};
use leafwing_input_manager::prelude::*;
use lib::components::{
    Action, Arch, CombatState, ComponentType, ControlledEntity, Door, EntityType, FloorTile,
    Health, HealthBar, LeftClick, OpenState, SpawnEvent, Sword, Target, Tile, Untraversable,
    UpdateEvent, Wall,
};

use crate::{
    assets::{ManAssetPack, WallAssetPack},
    input::{picking_listener, PickingEvent},
    resources::NetworkMapping,
    InsertUntraversableEvent, PlayerBundle, SpawnSlimeEvent, SpawnWallEvent,
};

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

            ComponentType::CombatState(c) => {
                commands.entity(event.entity).insert(c);
            }
            ComponentType::Target(c) => {
                if let Some(client_entity) = c.0 {
                    if let Some(client_entity) = network_mapping.server.get(&client_entity) {
                        commands
                            .entity(event.entity)
                            .insert(Target(Some(*client_entity)));
                        //println!("inserted target");
                    }
                }
            }
            ComponentType::OpenState(open_state) => {
                if let Ok((e, transform, _tile)) = query.get(event.entity) {
                    let mut open_trans = transform.clone();
                    match open_state {
                        OpenState::Open => {
                            open_trans.rotate_y(1.570796);
                        }
                        OpenState::Closed => {
                            open_trans.rotate_y(-1.570796);
                        }
                    }
                    commands.entity(e).insert(open_trans);
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
    cube_scene: Res<WallAssetPack>,
    assets: Res<Assets<Gltf>>,
    mut spawn_wall_event: EventWriter<SpawnWallEvent>,
    mut spawn_slime_event: EventWriter<SpawnSlimeEvent>,
    mut untrav_event: EventWriter<InsertUntraversableEvent>,
) {
    for event in spawn_event.iter() {
        let event_tile = event.tile;
        match event.entity_type {
            EntityType::Tile => {
                if let Some(gltf) = assets.get(&cube_scene.0) {
                    commands.entity(event.entity).insert((
                        SceneBundle {
                            scene: gltf.named_scenes.get("Scene.001").unwrap().clone(),
                            transform: event.tile.to_transform(),
                            ..Default::default()
                        },
                        LeftClick::Walk,
                        FloorTile,
                        event.tile,
                        //OnPointer::<Down>::send_event::<PickingEvent>(),
                        OnPointer::<Down>::run_callback(picking_listener),
                    ));
                } else {
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
                        FloorTile,
                    ));
                    //.forward_events::<PointerDown, PickingEvent>()
                }
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
                        CombatState::Idle,
                    ));
                    let mut transform = Transform::from_xyz(0., 3., 0.);
                    transform.rotate_z(FRAC_PI_2);
                    let hp_bar = commands.spawn((HealthBar,)).id();
                    commands.entity(event.entity).push_children(&[hp_bar]);
                } //.forward_events::<PointerDown, PickingEvent>()
                  //

                println!("spawn player: {:?}", player);
                if player.id == client.client_id() {
                    commands.entity(event.entity).insert((
                        ControlledEntity,
                        InputManagerBundle::<Action> {
                            action_state: ActionState::default(),
                            input_map: InputMap::new([(KeyCode::Key2, Action::AutoAttack)]),
                        },
                    ));
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
            EntityType::Wall(wall) => {
                spawn_wall_event.send(SpawnWallEvent {
                    wall,
                    tile: event.tile,
                });
                untrav_event.send(InsertUntraversableEvent(event.tile));
            }
            //Wall::Horizontal => {
            //commands.entity(event.entity).insert((
            //wall,
            //event.tile,
            //PbrBundle {
            //mesh: meshes.add(Mesh::from(shape::Box::new(1.0, 5., 0.2))),
            //material: materials.add(Color::rgb(1.0, 0.5, 0.2).into()),
            //transform: event.tile.to_transform(),
            //..Default::default()
            //},
            //));
            //}
            //Wall::Vertical => {
            //commands.entity(event.entity).insert((
            //wall,
            //event.tile,
            //PbrBundle {
            //mesh: meshes.add(Mesh::from(shape::Box::new(0.2, 5., 1.0))),
            //material: materials.add(Color::rgb(1.0, 0.5, 0.2).into()),
            //transform: event.tile.to_transform(),
            //..Default::default()
            //},
            //));
            //}
            //,
            EntityType::Arch(arch) => match arch {
                Arch::Vertical => {
                    if let Some(gltf) = assets.get(&cube_scene.0) {
                        commands.entity(event.entity).insert((
                            SceneBundle {
                                scene: gltf.named_scenes.get("arch").unwrap().clone(),
                                transform: event.tile.to_transform(),
                                ..Default::default()
                            },
                            //LeftClick::Walk,
                            //FloorTile,
                            event.tile,
                            //OnPointer::<Down>::send_event::<PickingEvent>(),
                        ));
                        untrav_event.send(InsertUntraversableEvent(event.tile));
                        let mut arch_v_tile: Tile = event.tile;
                        arch_v_tile.cell.0 += 2;
                        untrav_event.send(InsertUntraversableEvent(arch_v_tile));
                    }
                }
                Arch::Horizontal => {
                    if let Some(gltf) = assets.get(&cube_scene.0) {
                        let mut transform = event.tile.to_transform();
                        transform.rotate_y(-1.570796);
                        commands.entity(event.entity).insert((
                            SceneBundle {
                                scene: gltf.named_scenes.get("arch").unwrap().clone(),
                                transform,
                                ..Default::default()
                            },
                            //LeftClick::Walk,
                            //FloorTile,
                            event.tile,
                            //OnPointer::<Down>::send_event::<PickingEvent>(),
                        ));
                        untrav_event.send(InsertUntraversableEvent(event.tile));
                        let mut arch_h_tile: Tile = event.tile;
                        arch_h_tile.cell.2 += 2;
                        untrav_event.send(InsertUntraversableEvent(arch_h_tile));
                    }
                }
            },
            EntityType::Door(door) => match door {
                Door::Vertical => {
                    if let Some(gltf) = assets.get(&cube_scene.0) {
                        commands.entity(event.entity).insert((
                            SceneBundle {
                                scene: gltf.named_scenes.get("door").unwrap().clone(),
                                transform: event.tile.to_transform(),
                                ..Default::default()
                            },
                            //insert floor tile for pathing
                            FloorTile,
                            LeftClick::Open(event.entity),
                            OpenState::Closed,
                            event.tile,
                            OnPointer::<Down>::run_callback(picking_listener),
                        ));
                    }
                }
                _ => {
                    if let Some(gltf) = assets.get(&cube_scene.0) {
                        let mut transform = event.tile.to_transform();
                        transform.rotate_y(-1.570796);
                        commands.entity(event.entity).insert((
                            SceneBundle {
                                scene: gltf.named_scenes.get("door").unwrap().clone(),
                                transform,
                                ..Default::default()
                            },
                            //insert floor tile for pathing
                            FloorTile,
                            LeftClick::Open(event.entity),
                            OpenState::Closed,
                            event.tile,
                        ));
                    }
                }
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
                    OnPointer::<Down>::run_callback(picking_listener),
                ));
                let hp_bar = commands.spawn((HealthBar,)).id();
                commands.entity(event.entity).push_children(&[hp_bar]);
            }
            EntityType::Slime(slime) => {
                println!("send spawn slime event");
                println!("send spawn slime event");
                println!("send spawn slime event");
                println!("send spawn slime event");
                println!("send spawn slime event");
                println!("send spawn slime event");
                println!("send spawn slime event");
                spawn_slime_event.send(SpawnSlimeEvent {
                    tile: event.tile,
                    slime,
                    entity: event.entity,
                });
            }
        }
    }
}
