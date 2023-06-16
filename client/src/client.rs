use assets::{
    load_anims, should_load_anims, ManAssetPack, ShouldLoadAnims, SlimeAssetPack, WallAssetPack,
};
use bevy::{
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    gltf::Gltf,
    math::vec4,
    prelude::*,
};
use bevy_easings::*;
use bevy_mod_picking::prelude::*;
use input::PickingEvent;
use player::{setup_anims, update_health_bar};
use player_pathing::find_path;
use seldom_state::prelude::*;
use std::{any::type_name, f32::consts::FRAC_PI_2, time::Duration};
use sync::{spawn, update};

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_renet::RenetClientPlugin;
use camera::{camera_follow, setup_camera};
use connection::{new_renet_client, server_messages};
use leafwing_input_manager::prelude::*;
use lib::{
    components::{
        Action, Arch, DespawnEvent, Door, FloorTile, Health, HealthBar, Idle, LeftClick, Open,
        OpenState, Player, PlayerCommand, Running, Slime, SpawnEvent, TickEvent, Tile,
        Untraversable, UpdateEvent, Wall,
    },
    resources::Tick,
    ClickEvent,
};
use movement::{client_send_player_commands, get_path, scheduled_movement};
use receive::{despawn_message, load_message, spawn_message, tick, update_message};
use resources::{ClientLobby, NetworkMapping};
use smooth_bevy_cameras::{controllers::orbit::OrbitCameraPlugin, LookTransformPlugin};

pub mod assets;
pub mod camera;
pub mod components;
pub mod connection;
pub mod input;
pub mod movement;
pub mod player;
pub mod player_pathing;
pub mod plugins;
pub mod receive;
pub mod resources;
pub mod run_conditions;
pub mod sync;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum Move {
    North,
    South,
    West,
    East,
}
fn main() {
    let mut app = App::new();
    //app.add_plugins(DefaultPlugins);
    app.add_plugins(DefaultPlugins.build().disable::<bevy::audio::AudioPlugin>());
    app.add_plugin(RenetClientPlugin {
        clear_events: false,
    });

    app.add_plugin(InputManagerPlugin::<Action>::default());
    app.add_plugin(EasingsPlugin);
    app.add_plugin(OrbitCameraPlugin::default());
    app.add_plugins(
        DefaultPickingPlugins
            .build()
            .disable::<DebugPickingPlugin>(),
    );
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
    app.add_system(spawn_cube);
    app.add_system(server_messages);
    app.add_system(camera_follow);
    //app.add_system(load);
    app.insert_resource(new_renet_client());
    app.insert_resource(NetworkMapping::default());
    app.insert_resource(ClientLobby::default());
    app.insert_resource(Animations::default());
    app.insert_resource(SlimeAnimations::default());
    app.insert_resource(ShouldLoadAnims(true));
    app.insert_resource(LoadedSlime(true));
    app.init_resource::<ManAssetPack>();
    app.init_resource::<WallAssetPack>();
    app.init_resource::<SlimeAssetPack>();
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
    app.add_system(open_door);
    app.add_system(auto_attack);
    app.add_system(update_trav);
    app.add_system(update_health_bar);
    app.add_system(client_send_player_commands);
    //app.add_system(spawn_slime.run_if(loaded_slime));
    app.add_system(spawn_slime);
    app.add_system(load_anims.run_if(should_load_anims));
    app.add_system(find_path);
    app.add_system(slime_anims);
    app.add_event::<ClickEvent>();
    app.add_event::<PickingEvent>();
    app.add_event::<PlayerCommand>();
    app.add_event::<SpawnEvent>();
    app.add_event::<DespawnEvent>();
    app.add_event::<UpdateEvent>();
    app.add_event::<TickEvent>();
    app.add_event::<SpawnSlimeEvent>();
    app.add_event::<InsertUntraversableEvent>();
    app.add_event::<SpawnWallEvent>();
    app.register_type::<Tile>();
    app.register_type::<Health>();
    app.run();
}

pub struct SpawnSlimeEvent {
    pub slime: Slime,
    pub tile: Tile,
    pub entity: Entity,
}
pub struct SpawnWallEvent {
    pub wall: Wall,
    pub tile: Tile,
}
//#[bevycheck::system]
#[derive(Resource)]
pub struct LoadedSlime(bool);

pub fn loaded_slime(loaded_slime: Res<LoadedSlime>) -> bool {
    loaded_slime.0
}

pub fn slime_anims(
    animations: Res<SlimeAnimations>,
    mut animation_players: Query<(&Parent, &mut AnimationPlayer)>,
    player_parent: Query<(Entity, &Parent, &Children)>,
    slime_query: Query<Entity, With<Slime>>,
) {
    for (parent, mut player) in animation_players.iter_mut() {
        let player_parent_get = parent.get();
        for (player_parent_entity, parent_player_parent, _) in player_parent.iter() {
            if player_parent_get == player_parent_entity {
                let entity_animate = parent_player_parent.get();
                for e in slime_query.iter() {
                    if e == entity_animate {
                        player.play(animations.0[1].clone_weak()).repeat();
                    }
                }
            }
        }
    }
}
#[derive(Resource, Default)]
pub struct SlimeAnimations(pub Vec<Handle<AnimationClip>>);

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
                OnPointer::<Down>::run_callback(test),
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
pub fn spawn_cube(
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

// this inserts untrav twice for some reason
pub struct InsertUntraversableEvent(Tile);
pub fn update_trav(
    //walls: Query<&Tile, With<Wall>>,
    //arches: Query<(&Tile, &Arch)>,
    tiles: Query<(Entity, &Tile), (Without<Untraversable>, Without<OpenState>)>,
    mut events: EventReader<InsertUntraversableEvent>,
    mut commands: Commands,
) {
    for event in events.iter() {
        for (e, tile) in tiles.iter() {
            if tile.cell == event.0.cell {
                commands.entity(e).insert(Untraversable);
            }
        }
        //for wall in walls.iter() {
        //if tile.cell == wall.cell {
        //commands.entity(e).insert(Untraversable);
        //}
        //}
        //for (arch_tile, arch) in arches.iter() {
        //let mut arch_v_tile: Tile = *arch_tile;
        //arch_v_tile.cell.2 += 2;
        //let mut arch_h_tile: Tile = *arch_tile;
        //arch_h_tile.cell.0 += 2;
        //match arch {
        //Arch::Vertical => if tile.cell == arch_tile.cell || tile.cell == arch_tile.cell {},
        //Arch::Horizontal => (),
        //}
        //}
    }
}
pub fn auto_attack(
    //query: Query<&ActionState<Action>, (With<Player>, With<Target>)>,
    query: Query<&ActionState<Action>>,
    mut player_command: EventWriter<PlayerCommand>,
) {
    if let Ok(action_state) = query.get_single() {
        if action_state.just_pressed(Action::AutoAttack) {
            println!("auto attack!");
            player_command.send(PlayerCommand::AutoAttack);
        }
    }
}
pub fn open_door(mut query: Query<&mut Transform, (Added<Open>, With<Door>)>) {
    for mut transform in query.iter_mut() {
        transform.rotate_y(FRAC_PI_2);
    }
}

#[derive(Default, Clone, Copy, Component, Reflect)]
#[reflect(Component)]
pub struct Punching;

//impl Trigger for Punching {
//type Ok = f32;
//type Err = f32;
//type Param<'w, 's> = (
//Query<'w, 's, &'static ActionState<Action>>,
//EventWriter<'w, PlayerCommand>,
//);
//fn trigger(
//&self,
//entity: Entity,
//(query, mut player_command): &(
//Query<'_, '_, &'static ActionState<Action>>,
//EventWriter<'_, PlayerCommand>,
//),
//) -> Result<f32, f32> {
//if let Ok(action_state) = query.get_single() {
//if action_state.just_pressed(Action::AutoAttack) {
//println!("auto attack!");
//player_command.send(PlayerCommand::AutoAttack);
//return Ok(1.0);
//}
//}
//Err(0.0)
//}
//}
#[derive(Debug, Deref, DerefMut, Reflect)]
pub struct JustPressedTrigger<A: Actionlike>(pub A);

impl<A: Actionlike + Reflect> BoolTrigger for JustPressedTrigger<A> {
    type Param<'w, 's> = Query<'w, 's, &'static ActionState<A>>;

    fn trigger(
        &self,
        entity: Entity,
        actors: &bevy::prelude::Query<
            '_,
            '_,
            &'static leafwing_input_manager::action_state::ActionState<A>,
        >,
    ) -> bool {
        let Self(action) = self;
        actors
            .get(entity)
            .unwrap_or_else(|_| {
                panic!(
                    "entity {entity:?} with `JustPressedTrigger<{0}>` is missing `ActionState<{0}>`",
                    type_name::<A>()
                )
            })
            .just_pressed(action.clone())
    }
}
#[derive(Clone, Copy, FromReflect, Reflect)]
pub struct Moving;

impl Trigger for Moving {
    // put the parameters that your trigger needs here
    // for concision, you may use `bevy_ecs::system::system_param::lifetimeless` variants of system
    // params, like so:
    // type param<'w, 's> = (squery<&'static transform>, sres<time>);
    // triggers are immutable; you may not access system params mutably
    // do not query for the `statemachine` component in this type. this, unfortunately, will panic.
    // `time` is included here to demonstrate how to get multiple system params
    type Ok = f32;
    type Err = f32;
    //type param<'w, 's> = (query<'w, 's, &'static pathmap>, res<'w, tick>);
    type Param<'w, 's> = Query<'w, 's, (Entity, &'static Player), Changed<Transform>>;

    // this function checks if the given entity should trigger
    // it runs once per frame for each entity that is in a state that can transition
    // on this trigger
    fn trigger(&self, self_entity: Entity, player: &Self::Param<'_, '_>) -> Result<f32, f32> {
        if let Some((moving_entity, _)) = player.iter().next() {
            if self_entity == moving_entity {
                Ok(0.0)
            } else {
                Err(1.0)
            }
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
                .remove_on_exit::<Idle, Idle>(), //.trans::<Idle>(JustPressedTrigger(Action::AutoAttack), Punching)
                                                 //.insert_on_enter::<Punching>(Punching)
                                                 //.remove_on_exit::<Punching, Punching>()
        }
    }
}

#[derive(Resource, Default)]
pub struct Animations(pub Vec<Handle<AnimationClip>>);

pub fn mouse_input(
    mut click_event: EventWriter<ClickEvent>,
    mut events: EventReader<PickingEvent>,
    query: Query<(Entity, &LeftClick, &Tile)>,
    parent: Query<&Parent>,
    mut commands: Commands,
) {
    for event in events.iter() {
        if let PickingEvent::Clicked(clicked_entity) = event {
            commands.entity(*clicked_entity).log_components();
            if let Ok(p) = parent.get(*clicked_entity) {
                if let Ok(p) = parent.get(p.get()) {
                    if let Ok(p) = parent.get(p.get()) {
                        //commands.entity(p.get()).log_components();
                        if let Ok((target, left_click, destination)) = &query.get(p.get()) {
                            //println!("target:");
                            //commands.entity(*target).log_components();
                            click_event.send(ClickEvent::new(*target, **left_click, **destination));
                            match **left_click {
                                LeftClick::Open(_) => {
                                    commands.entity(*target).insert(LeftClick::Close(*target));
                                }
                                _ => (),
                            }
                        }
                    }
                }
            }
            if let Ok((target, left_click, destination)) = &query.get(*clicked_entity) {
                click_event.send(ClickEvent::new(*target, **left_click, **destination));
            }
        }
    }
}
pub fn test(
    In(event): In<ListenedEvent<Down>>,
    mut commands: Commands,
    parent: Query<&Parent>,
    mut picking_event: EventWriter<PickingEvent>,
) -> Bubble {
    //commands.entity(event.target).log_components();

    if let Ok(p) = parent.get(event.target) {
        if let Ok(p) = parent.get(p.get()) {
            if let Ok(p) = parent.get(p.get()) {
                if let Ok(p) = parent.get(p.get()) {
                    //commands.entity(p.get()).log_components();
                    picking_event.send(PickingEvent::Clicked(p.get()));
                }
            }
        }
    }
    Bubble::Burst
}
fn make_pickable(
    mut commands: Commands,
    meshes: Query<Entity, (With<Handle<Mesh>>, Without<RaycastPickTarget>)>,
    mut pick_event: EventWriter<PickingEvent>,
) {
    for entity in meshes.iter() {
        commands.entity(entity).insert((
            PickableBundle::default(),
            RaycastPickTarget::default(),
            HIGHLIGHT_TINT.clone(),
            //OnPointer::<Down>::run_callback(test),
            (OnPointer::<Down>::send_event::<PickingEvent>()),
        ));
        //if let None = pointer {
        //commands
        //.entity(entity)
        //.insert(OnPointer::<Down>::send_event::<PickingEvent>());
        //}
    }
}
const HIGHLIGHT_TINT: Highlight<StandardMaterial> = Highlight {
    hovered: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + vec4(-0.5, -0.3, 0.9, 0.8), // hovered is blue
        ..matl.to_owned()
    })),
    pressed: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + vec4(-0.4, -0.4, 0.8, 0.8), // pressed is a different blue
        ..matl.to_owned()
    })),
    selected: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + vec4(-0.4, 0.8, -0.4, 0.0), // selected is green
        ..matl.to_owned()
    })),
};
