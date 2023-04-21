use assets::{load_anims, should_load_anims, ManAssetPack, ShouldLoadAnims};
use bevy::{
    ecs::{
        entity::MapEntities,
        schedule::{LogLevel, ScheduleBuildSettings},
        system::SystemParam,
    },
    gltf::Gltf,
    input::mouse,
    prelude::*,
};
use bevy_easings::*;
use bevy_mod_picking::{DefaultPickingPlugins, PickableBundle, PickableMesh, PickingEvent};
use seldom_state::prelude::*;
use std::{
    f32::consts::FRAC_1_PI,
    f32::consts::{FRAC_2_PI, FRAC_PI_2, FRAC_PI_3, PI},
    time::Duration,
};

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
use movement::{client_send_player_commands, get_path, scheduled_movement, PathMap};
use rand::Rng;
use receive::{despawn_message, load_message, spawn_message, tick, update_message};
use resources::{ClientLobby, NetworkMapping};
use serde::{Deserialize, Serialize};
use smooth_bevy_cameras::{
    controllers::{orbit::OrbitCameraPlugin, unreal::UnrealCameraPlugin},
    LookTransformPlugin,
};

use crate::resources::ClientInfo;
pub mod assets;
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
    app.add_plugin(EasingsPlugin);
    app.add_plugins(DefaultPickingPlugins);
    app.insert_resource(FixedTime::new(Duration::from_millis(100)));
    app.insert_resource(Tick::default());
    app.edit_schedule(CoreSchedule::Main, |schedule| {
        schedule.set_build_settings(ScheduleBuildSettings {
            ambiguity_detection: LogLevel::Ignore,
            ..default()
        });
    });
    app.add_plugin(StateMachinePlugin);
    app.add_plugin(TriggerPlugin::<Moving>::default());
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
    app.insert_resource(Animations::default());
    app.insert_resource(ShouldLoadAnims(true));
    app.init_resource::<ManAssetPack>();
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
    app.add_system(setup_anims);
    app.add_system(client_send_player_commands);
    app.add_system(load_anims.run_if(should_load_anims));
    app.add_event::<ClickEvent>();
    app.add_event::<PlayerCommand>();
    app.add_event::<SpawnEvent>();
    app.add_event::<DespawnEvent>();
    app.add_event::<UpdateEvent>();
    app.add_event::<TickEvent>();
    app.register_type::<Tile>();
    app.run();
}

pub fn setup_anims(
    animations: Res<Animations>,
    mut animation_players: Query<(&Parent, &mut AnimationPlayer)>,
    player_parent: Query<(Entity, &Parent, &Children)>,
    state: Query<(Entity, Option<&Running>), Changed<Transform>>,
    //mut commands: Commands,
) {
    for (parent, mut player) in animation_players.iter_mut() {
        let player_parent_get = parent.get();
        for (player_parent_entity, parent_player_parent, _) in player_parent.iter() {
            if player_parent_get == player_parent_entity {
                let entity_animate = parent_player_parent.get();
                for (e, running) in state.iter() {
                    if entity_animate == e {
                        if let Some(_) = running {
                            player.play(animations.0[9].clone_weak()).repeat();
                        } else {
                            player.play(animations.0[3].clone_weak()).repeat();
                        }
                    }
                }
                //commands.entity(parent_player_parent.get()).log_components();
            }
        }
    }
}
#[derive(Clone, Component, Reflect)]
#[component(storage = "SparseSet")]
pub struct Idle;

#[derive(Clone, Component, Reflect)]
#[component(storage = "SparseSet")]
pub struct Running;

#[derive(Clone, Copy, FromReflect, Reflect)]
pub struct Moving;

impl Trigger for Moving {
    // Put the parameters that your trigger needs here
    // For concision, you may use `bevy_ecs::system::system_param::lifetimeless` variants of system
    // params, like so:
    // type Param<'w, 's> = (SQuery<&'static Transform>, SRes<Time>);
    // Triggers are immutable; you may not access system params mutably
    // Do not query for the `StateMachine` component in this type. This, unfortunately, will panic.
    // `Time` is included here to demonstrate how to get multiple system params
    type Ok = f32;
    type Err = f32;
    //type Param<'w, 's> = (Query<'w, 's, &'static PathMap>, Res<'w, Tick>);
    type Param<'w, 's> = Query<'w, 's, &'static Player, Changed<Transform>>;

    // This function checks if the given entity should trigger
    // It runs once per frame for each entity that is in a state that can transition
    // on this trigger
    fn trigger(&self, _entity: Entity, player: &Self::Param<'_, '_>) -> Result<f32, f32> {
        if let Some(_) = player.iter().next() {
            Ok(0.0)
        } else {
            Err(1.0)
        }
    }
}
#[derive(Bundle)]
pub struct PlayerBundle {
    tile: Tile,
    state: StateMachine,
}

impl PlayerBundle {
    pub fn new(tile: &Tile) -> Self {
        PlayerBundle {
            tile: *tile,
            state: StateMachine::new(Idle)
                .trans::<Idle>(Moving, Running)
                .insert_on_enter::<Running>(Running)
                .remove_on_exit::<Running, Running>()
                .trans::<Running>(NotTrigger(Moving), Idle)
                .insert_on_enter::<Idle>(Idle)
                .remove_on_exit::<Idle, Idle>(),
        }
    }
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
    query: Query<(Entity, &Transform, &Tile)>,
) {
    for event in update_event.iter() {
        println!("Received Update Event");
        match event.component {
            ComponentType::Tile(t) => {
                for (e, old_transform, old_tile) in query.iter() {
                    if e == event.entity {
                        let mut transform = t.to_transform();
                        //let old_transform = old_tile.to_transform();
                        let mut rotation = 0.;
                        if old_tile.cell.0 > t.cell.0 {
                            rotation = -FRAC_PI_2;
                            println!("WEST");
                        } else if old_tile.cell.0 < t.cell.0 {
                            //rotation = 1.5;

                            rotation = FRAC_PI_2;
                            println!("EAST");
                        }
                        if old_tile.cell.2 > t.cell.2 {
                            rotation = -PI;
                            println!("NORTH");
                        } else if old_tile.cell.2 < t.cell.2 {
                            rotation = 0.0;
                            println!("SOUTH");
                        }

                        println!("old_tile: {:?}, new: {:?}", old_tile, t);

                        if old_tile.cell.0 < t.cell.0 && old_tile.cell.2 > t.cell.2 {
                            println!("NORTH EAST");
                            rotation = 2.2;
                        }

                        if old_tile.cell.0 > t.cell.0 && old_tile.cell.2 > t.cell.2 {
                            rotation = -2.2;

                            println!("NORTH WEST");
                        }

                        if old_tile.cell.0 < t.cell.0 && old_tile.cell.2 < t.cell.2 {
                            rotation = FRAC_PI_3;
                            println!("SOUTH EAST");
                        }
                        if old_tile.cell.0 > t.cell.0 && old_tile.cell.2 < t.cell.2 {
                            println!("SOUTH WEST");
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
        };
    }
}

use bevy::animation::AnimationPlayer;
#[derive(Resource, Default)]
pub struct Animations(pub Vec<Handle<AnimationClip>>);

pub fn spawn(
    mut commands: Commands,
    mut spawn_event: EventReader<SpawnEvent>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    client: Res<RenetClient>,
    mut man_scene: ResMut<ManAssetPack>,
    assets: Res<Assets<Gltf>>,
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
                    event.tile,
                    //PickableBundle::default(),
                    //PickRaycastTarget::default(),
                    //NoDeselect,
                    LeftClick::Walk,
                ));
                //.forward_events::<PointerDown, PickingEvent>()
            }
            EntityType::Player(player) => {
                let transform = event.tile.to_transform();
                if let Some(gltf) = assets.get(&man_scene.0) {
                    commands.entity(event.entity).insert((
                        SceneBundle {
                            scene: gltf.scenes[0].clone(),
                            ..Default::default()
                        },
                        PlayerBundle::new(&event.tile),
                        player,
                    ));
                } else {
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
                        PlayerBundle::new(&event.tile),
                        //LeftClick::default(),
                    ));
                }
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
