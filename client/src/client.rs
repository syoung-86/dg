use crate::entities::player::control::Moving;
use assets::{load_anims, should_load_anims, ManAssetPack, ShouldLoadAnims};
use bevy::{
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    prelude::*,
};
use bevy_easings::*;
use bevy_mod_picking::prelude::*;
use entities::{
    player::{anims::setup_anims, healthbar::update_health_bar, pathing::find_path, control::auto_attack},
    slime::{
        anims::{slime_anims, SlimeAnimations},
        extra::{LoadedSlime, SlimeAssetPack, SpawnSlimeEvent},
        spawn::spawn_slime,
    },
    wall::{assets::WallAssetPack, extra::SpawnWallEvent}, extra::InsertUntraversableEvent,
};
use input::{make_pickable, mouse_input, PickingEvent};
use seldom_state::prelude::*;
use std::{any::type_name, f32::consts::FRAC_PI_2, time::Duration};
use sync::{spawn, update};

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_proto::prelude::*;
use bevy_renet::RenetClientPlugin;
use camera::{camera_follow, setup_camera};
use connection::{new_renet_client, server_messages};
use leafwing_input_manager::prelude::*;
use lib::{
    components::{
        Action, DespawnEvent, Door, Health, HealthBar, Idle, LeftClick, Open, OpenState, Player,
        PlayerCommand, Running, Slime, SpawnEvent, TickEvent, Tile, Untraversable, UpdateEvent,
        Wall,
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
pub mod entities;
pub mod input;
pub mod movement;
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
    app.add_plugin(ProtoPlugin::new());
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
    app.add_system(entities::wall::spawn::dg_wall);
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
    app.add_system(entities::door::control::open_door);
    app.add_system(auto_attack);
    app.add_system(entities::extra::update_trav);
    app.add_system(update_health_bar);
    app.add_system(client_send_player_commands);
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
    app.add_startup_system(load_sword_proto);
    app.add_system(spawn_proto.run_if(prototype_ready("TwoHander").and_then(run_once())));
    app.run();
}

fn load_sword_proto(mut prototypes: PrototypesMut) {
    prototypes.load("prototypes/two_hander.prototype.ron");
}

fn spawn_proto(mut commands: ProtoCommands) {
    commands.spawn("TwoHander");
}

// this inserts untrav twice for some reason
#[derive(Resource, Default)]
pub struct Animations(pub Vec<Handle<AnimationClip>>);
