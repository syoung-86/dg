use bevy::prelude::*;
use bevy_renet::RenetClientPlugin;
pub mod init;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugin(RenetClientPlugin{clear_events: false});

}
