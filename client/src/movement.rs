use bevy::prelude::*;
use bevy_renet::renet::RenetClient;
use lib::{
    channels::ClientChannel,
    components::{ControlledEntity, Direction, LeftClick, Path, PlayerCommand, Tile},
    resources::Tick,
    ClickEvent,
};

use crate::resources::NetworkMapping;
pub fn get_path(
    mut commands: Commands,
    mut walk_event: EventReader<ClickEvent>,
    query: Query<(Entity, &Tile), With<ControlledEntity>>,
    tick: Res<Tick>,
) {
    for event in walk_event.iter() {
        if let Ok((entity, origin)) = query.get_single() {
            let mut_tick = Tick { tick: tick.tick };
            let path = Path {
                destination: event.destination,
                origin: *origin,
                left_click: event.left_click,
            };
            commands.entity(entity).insert(path);
            let path_map = create_path(path, mut_tick);
            //println!("path_map: {:?}", path_map);
            commands.entity(entity).insert(path_map);
        }
    }
}

pub fn step(mut path: &mut Path) {
    let mut direction = Direction::Bad;
    if path.origin.cell.0 < path.destination.cell.0 && path.origin.cell.2 == path.destination.cell.2
    {
        direction = Direction::North;
    }

    if path.origin.cell.0 > path.destination.cell.0 && path.origin.cell.2 == path.destination.cell.2
    {
        direction = Direction::South;
    }

    if path.origin.cell.0 == path.destination.cell.0 && path.origin.cell.2 > path.destination.cell.2
    {
        direction = Direction::West;
    }
    if path.origin.cell.0 == path.destination.cell.0 && path.origin.cell.2 < path.destination.cell.2
    {
        direction = Direction::East;
    }

    if path.origin.cell.0 < path.destination.cell.0 && path.origin.cell.2 < path.destination.cell.2
    {
        direction = Direction::NorthEast;
    }

    if path.origin.cell.0 < path.destination.cell.0 && path.origin.cell.2 > path.destination.cell.2
    {
        direction = Direction::NorthWest;
    }

    if path.origin.cell.0 > path.destination.cell.0 && path.origin.cell.2 > path.destination.cell.2
    {
        direction = Direction::SouthWest;
    }

    if path.origin.cell.0 > path.destination.cell.0 && path.origin.cell.2 < path.destination.cell.2
    {
        direction = Direction::SouthEast;
    }
    //println!("Direction: {:?}", direction);
    match direction {
        Direction::North => path.origin.cell.0 += 1,
        Direction::East => path.origin.cell.2 += 1,
        Direction::South => path.origin.cell.0 -= 1,
        Direction::West => path.origin.cell.2 -= 1,
        Direction::NorthEast => {
            path.origin.cell.0 += 1;
            path.origin.cell.2 += 1
        }
        Direction::SouthEast => {
            path.origin.cell.0 -= 1;
            path.origin.cell.2 += 1
        }
        Direction::SouthWest => {
            path.origin.cell.0 -= 1;
            path.origin.cell.2 -= 1
        }
        Direction::NorthWest => {
            path.origin.cell.0 += 1;
            path.origin.cell.2 -= 1
        }
        Direction::Bad => (),
    }
}
#[derive(Clone, Eq, PartialEq, Debug, Component, Default)]
pub struct PathMap {
    pub steps: Vec<(Tick, LeftClick, Tile)>,
}
pub fn create_path(mut path: Path, client_tick: Tick) -> PathMap {
    let mut path_map: PathMap = PathMap::default();
    let mut step_tick = client_tick;
    while path.origin.cell != path.destination.cell {
        step_tick.tick += 2;
        path.step();
        match path.left_click {
            LeftClick::Walk => {
                path_map.steps.push((
                    step_tick,
                    path.left_click,
                    Tile {
                        cell: path.origin.cell,
                    },
                ));
            }
            LeftClick::Pickup(Some(_)) => {
                if path.origin.cell.1 == path.destination.cell.1 {
                    path_map.steps.push((
                        step_tick,
                        path.left_click,
                        Tile {
                            cell: path.origin.cell,
                        },
                    ));
                } else {
                    path_map.steps.push((
                        step_tick,
                        LeftClick::Walk,
                        Tile {
                            cell: path.origin.cell,
                        },
                    ));
                }
            }
            LeftClick::Pull => {
                if path.origin.cell.0 != path.destination.cell.0
                    || path.origin.cell.2 != path.destination.cell.2
                {
                    path_map.steps.push((
                        step_tick,
                        LeftClick::Walk,
                        Tile {
                            cell: path.origin.cell,
                        },
                    ));
                } else {
                    path_map.steps.push((
                        step_tick,
                        LeftClick::Pull,
                        Tile {
                            cell: path.origin.cell,
                        },
                    ));
                }
            }

            LeftClick::Attack(e) => {
                if path.origin.cell.0 != path.destination.cell.0
                    || path.origin.cell.2 != path.destination.cell.2
                {
                    path_map.steps.push((
                        step_tick,
                        LeftClick::Walk,
                        Tile {
                            cell: path.origin.cell,
                        },
                    ));
                } else {
                    path_map.steps.push((
                        step_tick,
                        LeftClick::Attack(e),
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
                        //println!("some e: {:?}", e);
                        if let Some(server_entity) = network_mapping.client.remove(e) {
                            player_commands.send(PlayerCommand::LeftClick(
                                LeftClick::Pickup(Some(server_entity)),
                                *tile,
                            ));
                            //println!("command send pickup {:?}", e);
                        }
                        //delete_writer.send(DeleteMe(*e));
                        commands.entity(*e).despawn_recursive();
                    }
                    LeftClick::Attack(e) =>{
                        if let Some(server_entity) = network_mapping.client.get(e){
                            player_commands.send(PlayerCommand::LeftClick(LeftClick::Attack(*server_entity), *tile));
                        }
                    }
                    _=> {
                        //println!("walk");
                        player_commands.send(PlayerCommand::LeftClick(*left_click, *tile));
                    }
                    //LeftClick::Pull => {
                        //player_commands.send(PlayerCommand::LeftClick(*left_click, *tile));
                        //println!("PULL");
                    //}

                    //LeftClick::Attack => {
                        //player_commands.send(PlayerCommand::LeftClick(*left_click, *tile));
                        //println!("PULL");
                    //}
                    //_ => (),
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

        //println!("send");
    }
}
