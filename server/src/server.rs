use bevy::prelude::*;
use plugins::MyPlugins;
pub mod plugins;

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(MyPlugins);
    
    app.run();
}
