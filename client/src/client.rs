use bevy::prelude::*;
use bevy_renet::RenetClientPlugin;
use camera::setup_camera;
pub mod init;
pub mod plugins;
pub mod camera;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugin(RenetClientPlugin {
        clear_events: false,
    });

    app.add_startup_system(setup_camera);
    app.run();
}
