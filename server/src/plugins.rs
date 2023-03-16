use bevy::prelude::*;
use bevy_renet::RenetServerPlugin;
use shared::TickSet;

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RenetServerPlugin {
            clear_events: false,
        });

        //app.configure_sets(
            //(
                //TickSet::Connection,
                //TickSet::SendChunk,
                //TickSet::ReceiveUnreliable,
                //TickSet::ReceiveReliable,
                //TickSet::SendUnreliable,
                //TickSet::SendReliable,
                //TickSet::Clear,
            //)
                //.chain(),
        //);
        app.configure_set((TickSet::SendChunk).after(TickSet::Connection));
        app.configure_set((TickSet::ReceiveUnreliable).after(TickSet::SendChunk));
        app.configure_set((TickSet::ReceiveReliable).after(TickSet::ReceiveUnreliable));
        app.configure_set((TickSet::SendUnreliable).after(TickSet::ReceiveReliable));
        app.configure_set((TickSet::SendReliable).after(TickSet::SendUnreliable));
        app.configure_set((TickSet::Clear).after(TickSet::SendReliable).after(CoreSet::Last));
    }
}
