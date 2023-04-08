use bevy::prelude::*;
use bevy_renet::renet::RenetClient;
use lib::{
    components::{ControlledEntity, LeftClick, Path, PlayerCommand, Tile},
    resources::Tick,
    ClickEvent, channels::ClientChannel,
};

use crate::resources::NetworkMapping;
pub fn get_path(
    mut commands: Commands,
    mut walk_event: EventReader<ClickEvent>,
    query: Query<(Entity, &Tile), With<ControlledEntity>>,
    tick: Res<Tick>,
) {
    for event in walk_event.iter() {
        let (entity, origin) = query.get_single().unwrap();
        let mut_tick = Tick { tick: tick.tick };
        let path = Path {
            destination: event.destination,
            origin: *origin,
            left_click: event.left_click,
        };
        let path_map = create_path(path, mut_tick);
        println!("path_map: {:?}", path_map);
        commands.entity(entity).insert(path_map);
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Component, Default)]
pub struct PathMap {
    //pub steps: HashMap<Tick, Tile>,
    pub steps: Vec<(Tick, LeftClick, Tile)>,
}
pub fn create_path(mut path: Path, client_tick: Tick) -> PathMap {
    let mut path_map: PathMap = PathMap::default();
    let mut step_tick = client_tick;
    while path.origin.cell != path.destination.cell {
        step_tick.tick += 1;
        path.step();
        match path.left_click {
            LeftClick::Walk => {
                path_map.steps.push((
                    step_tick.clone(),
                    path.left_click,
                    Tile {
                        cell: path.origin.cell,
                    },
                ));
            }
            LeftClick::Pickup(Some(_)) => {
                if path.origin.cell.1 == path.destination.cell.1 {
                    path_map.steps.push((
                        step_tick.clone(),
                        path.left_click,
                        Tile {
                            cell: path.origin.cell,
                        },
                    ));
                } else {
                    path_map.steps.push((
                        step_tick.clone(),
                        LeftClick::Walk,
                        Tile {
                            cell: path.origin.cell,
                        },
                    ));
                }
            }
            _ => (),
        }
    }
    path_map
}
pub fn scheduled_movement(
    mut query: Query<&mut PathMap>,
    game_tick: ResMut<Tick>,
    mut player_commands: EventWriter<PlayerCommand>,
    mut commands: Commands,
    mut network_mapping: ResMut<NetworkMapping>,
) {
    if let Ok(mut path_map) = query.get_single_mut() {
        path_map.steps.retain(|(scheduled_tick, left_click, tile)| {
            if scheduled_tick.tick <= game_tick.tick {
                //player_commands.send(PlayerCommand::LeftClick(*left_click, *tile));
                match left_click {
                    LeftClick::Pickup(Some(e)) => {
                        println!("some e: {:?}", e);
                        if let Some(server_entity) = network_mapping.client.remove(e) {
                            player_commands.send(PlayerCommand::LeftClick(
                                LeftClick::Pickup(Some(server_entity)),
                                *tile,
                            ));
                            println!("command send pickup {:?}", e);
                        }
                        //delete_writer.send(DeleteMe(*e));
                        commands.entity(*e).despawn_recursive();
                    }
                    LeftClick::Walk => {
                        println!("walk");
                        player_commands.send(PlayerCommand::LeftClick(*left_click, *tile));
                    }
                    _ => (),
                }
                false // Remove the current element from the vector
            } else {
                true // Keep the current element in the vector
            }
        });
    }
    //}
}

pub fn client_send_player_commands(
    mut player_commands: EventReader<PlayerCommand>,
    mut client: ResMut<RenetClient>,
) {
    for command in player_commands.iter() {
        let command_message = bincode::serialize(command).unwrap();
        client.send_message(ClientChannel::Command, command_message);

                        println!("send");
    }
}
