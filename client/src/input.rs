use bevy::prelude::*;
use bevy_mod_picking::prelude::*;
use lib::ClickEvent;
pub enum PickingEvent {
    Clicked(Entity),
    RightClicked(Entity),
}

impl From<ListenedEvent<Down>> for PickingEvent {
    fn from(event: ListenedEvent<Down>) -> Self {
        println!("clicked");
        PickingEvent::Clicked(event.target)
    }
}

//impl PickingEvent {
//pub fn handle_events(mut left_click: EventReader<PickingEvent>) {
//for event in left_click.iter() {
//info!("{:?}", event.entity);
//println!("WTF");
//println!("WTF");
//println!("WTF");
//println!("WTF");
//println!("WTF");
//println!("WTF");
//println!("WTF");
//println!("WTF");
//}
//}
//}
