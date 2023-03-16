use bevy::prelude::*;
use bevy_renet::RenetServerPlugin;
use shared::TickSet;

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RenetServerPlugin {
            clear_events: false,
        });

        app.configure_sets(
            (
                TickSet::Connection,
                TickSet::SendChunk,
                TickSet::ReceiveUnreliable,
                TickSet::ReceiveReliable,
                TickSet::SendUnreliable,
                TickSet::SendReliable,
            )
                .chain(),
        );
    }
}
