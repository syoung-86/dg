use std::time::Duration;

use bevy::{
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    prelude::*,
};
use bevy_renet::RenetClientPlugin;
use camera::{camera_follow, setup_camera};
use connection::{new_renet_client, server_messages};
use resources::{ClientLobby, NetworkMapping};
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

    //app.add_system(fixed_time.in_schedule(CoreSchedule::FixedUpdate));
    app.add_startup_system(setup_camera);
    app.add_system(server_messages);
    app.add_system(camera_follow);
    app.insert_resource(new_renet_client());
    app.add_systems(().distributive_run_if(bevy_renet::client_connected));
    app.insert_resource(NetworkMapping::default());
    app.insert_resource(ClientLobby::default());
    //app.add_system(run_fixed_update_schedule);
    app.run();
}
