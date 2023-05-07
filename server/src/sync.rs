use crate::events::ServerUpdateEvent;
use bevy::prelude::*;
use lib::components::{Client, ComponentType, Health, Tile, UpdateEvent};
macro_rules! update_in_scope {
    ($fn_name:ident, $type_name:ident) => {
        pub fn $fn_name(
            clients: Query<&Client>,
            components: Query<(Entity, &$type_name), Changed<$type_name>>,
            mut update_event: EventWriter<ServerUpdateEvent>,
        ) {
            for client in clients.iter() {
                for (entity, component) in components.iter() {
                    if client.scoped_entities.contains(&entity) {
                        let event = UpdateEvent {
                            entity,
                            component: ComponentType::$type_name(*component),
                        };
                        update_event.send(ServerUpdateEvent {
                            event,
                            client_id: client.id,
                        });
                    }
                }
            }
        }
    };
}

update_in_scope!(update_health, Health);
update_in_scope!(update_tile, Tile);
