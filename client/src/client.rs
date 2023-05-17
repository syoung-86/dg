use assets::{load_anims, should_load_anims, ManAssetPack, ShouldLoadAnims};
use bevy::{
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    math::vec4,
    prelude::*,
};
use bevy_easings::*;
use bevy_mod_picking::prelude::*;
use input::PickingEvent;
use player::{setup_anims, update_health_bar};
use seldom_state::prelude::*;
use std::{f32::consts::FRAC_PI_2, time::Duration};
use sync::{spawn, update};

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_renet::RenetClientPlugin;
use camera::{camera_follow, setup_camera};
use connection::{new_renet_client, server_messages};
use leafwing_input_manager::prelude::*;
use lib::{
    components::{
        DespawnEvent, Door, Health, Idle, LeftClick, Open, Player, PlayerCommand, Running,
        SpawnEvent, TickEvent, Tile, UpdateEvent,
    },
    resources::Tick,
    ClickEvent,
};
use movement::{client_send_player_commands, get_path, scheduled_movement};
use receive::{despawn_message, load_message, spawn_message, tick, update_message};
use resources::{ClientLobby, NetworkMapping};
use smooth_bevy_cameras::LookTransformPlugin;

pub mod assets;
pub mod camera;
pub mod components;
pub mod connection;
pub mod input;
pub mod movement;
pub mod player;
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
    app.add_system(open_door);
    app.add_system(update_health_bar);
    app.add_system(client_send_player_commands);
    app.add_system(load_anims.run_if(should_load_anims));
    app.add_event::<ClickEvent>();
    app.add_event::<PickingEvent>();
    app.add_event::<PlayerCommand>();
    app.add_event::<SpawnEvent>();
    app.add_event::<DespawnEvent>();
    app.add_event::<UpdateEvent>();
    app.add_event::<TickEvent>();
    app.register_type::<Tile>();
    app.register_type::<Health>();
    app.run();
}

pub fn open_door(mut query: Query<&mut Transform, (Added<Open>, With<Door>)>) {
    for mut transform in query.iter_mut() {
        transform.rotate_y(FRAC_PI_2);
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
                .remove_on_exit::<Idle, Idle>(),
        }
    }
}

#[derive(Resource, Default)]
pub struct Animations(pub Vec<Handle<AnimationClip>>);

pub fn mouse_input(
    mut click_event: EventWriter<ClickEvent>,
    mut events: EventReader<PickingEvent>,
    query: Query<(Entity, &LeftClick, &Tile)>,
) {
    for event in events.iter() {
        if let PickingEvent::Clicked(clicked_entity) = event {
            if let Ok((target, left_click, destination)) = &query.get(*clicked_entity) {
                click_event.send(ClickEvent::new(*target, **left_click, **destination));
            }
        }
    }
}
fn make_pickable(
    mut commands: Commands,
    meshes: Query<Entity, (With<Handle<Mesh>>, Without<RaycastPickTarget>)>,
) {
    for entity in meshes.iter() {
        commands.entity(entity).insert((
            PickableBundle::default(),
            RaycastPickTarget::default(),
            OnPointer::<Down>::send_event::<PickingEvent>(),
            HIGHLIGHT_TINT.clone(),
        ));
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
