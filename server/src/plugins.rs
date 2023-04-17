use bevy::prelude::*;
use bevy_renet::RenetServerPlugin;
use lib::TickSet;

use crate::{events::{clear_event, ChunkRequest, ClientSetup}, LeftClickEvent};

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RenetServerPlugin {
            clear_events: false,
        });

        app.configure_set((TickSet::SendChunk).after(TickSet::Connection));
        app.configure_set((TickSet::ReceiveUnreliable).after(TickSet::SendChunk));
        app.configure_set((TickSet::ReceiveReliable).after(TickSet::ReceiveUnreliable));
        app.configure_set((TickSet::SendUnreliable).after(TickSet::ReceiveReliable));
        app.configure_set((TickSet::SendReliable).after(TickSet::SendUnreliable));
        //app.configure_set(
        //(TickSet::Clear)
        //.after(CoreSet::Last),
        //);
    }
}

pub struct ClearEventPlugin;

impl Plugin for ClearEventPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems((
            //clear_event::<ClientSetup>.in_base_set(CoreSet::Last),
            //.in_schedule(CoreSchedule::FixedUpdate)
            //.in_set(TickSet::Clear),
            //clear_event::<ChunkRequest>.in_base_set(CoreSet::Last),
            clear_event::<LeftClickEvent>.in_base_set(CoreSet::Last),
            //.in_schedule(CoreSchedule::FixedUpdate)
            //.in_set(TickSet::Clear),
        ));
    }
}
