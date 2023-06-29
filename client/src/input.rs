use bevy::{math::vec4, prelude::*};
use bevy_mod_picking::prelude::*;
use lib::{
    components::{LeftClick, Tile},
    ClickEvent,
};
pub enum PickingEvent {
    Clicked(Entity),
    RightClicked(Entity),
}

impl From<ListenedEvent<Down>> for PickingEvent {
    fn from(event: ListenedEvent<Down>) -> Self {
        println!("clicked");
        PickingEvent::Clicked(event.listener)
    }
}

pub fn picking_listener(
    In(event): In<ListenedEvent<Down>>,
    mut picking_event: EventWriter<PickingEvent>,
) -> Bubble {
    picking_event.send(PickingEvent::Clicked(event.listener));
    Bubble::Up
}
pub fn make_pickable(
    mut commands: Commands,
    meshes: Query<Entity, (With<Handle<Mesh>>, Without<RaycastPickTarget>)>,
) {
    for entity in meshes.iter() {
        commands.entity(entity).insert((
            PickableBundle::default(),
            RaycastPickTarget::default(),
            HIGHLIGHT_TINT.clone(),
        ));
    }
}
const HIGHLIGHT_TINT: Highlight<StandardMaterial> = Highlight {
    hovered: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + vec4(-0.5, -0.3, 0.9, 0.8), // hovered is blue
        ..matl.to_owned()
    })),
    pressed: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + vec4(-0.4, -0.4, 0.8, 0.8), // pressed is a different blue
        ..matl.to_owned()
    })),
    selected: Some(HighlightKind::new_dynamic(|matl| StandardMaterial {
        base_color: matl.base_color + vec4(-0.4, 0.8, -0.4, 0.0), // selected is green
        ..matl.to_owned()
    })),
};

pub fn mouse_input(
    mut click_event: EventWriter<ClickEvent>,
    mut events: EventReader<PickingEvent>,
    query: Query<(Entity, &LeftClick, &Tile)>,
) {
    for event in events.iter() {
        if let PickingEvent::Clicked(clicked_entity) = event {
            if let Ok((target, left_click, destination)) = &query.get(*clicked_entity) {
                click_event.send(ClickEvent::new(*target, **left_click, **destination));
            }
        }
    }
}
