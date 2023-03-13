use bevy::{app::PluginGroupBuilder, prelude::*};
use bevy_renet::RenetServerPlugin;

pub struct MyPlugins;

impl PluginGroup for MyPlugins {
    fn build(self) -> bevy::app::PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>().add(RenetServerPlugin {
            clear_events: false,
        })
    }
}
