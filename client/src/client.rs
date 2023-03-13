use bevy::prelude::*;
use bevy_renet::RenetClientPlugin;
use camera::{camera_follow, setup_camera};
use connection::new_renet_client;
pub mod camera;
pub mod connection;
pub mod plugins;
pub mod resources;
pub mod components;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugin(RenetClientPlugin {
        clear_events: false,
    });

    app.add_startup_system(setup_camera);
    app.add_system(camera_follow);
    app.insert_resource(new_renet_client());
    app.run();
}
