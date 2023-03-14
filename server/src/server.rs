use bevy::prelude::*;
use connection::{new_renet_server, client_handler};
use plugins::MyPlugins;
use resources::ServerLobby;

pub mod connection;
pub mod plugins;
pub mod resources;
pub mod world;

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(MyPlugins);

    app.insert_resource(new_renet_server());
    app.insert_resource(ServerLobby::default());
    app.add_system(client_handler);
    app.add_startup_system(create_tiles);
    app.run();
}
