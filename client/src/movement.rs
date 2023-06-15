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
) {
    for event in walk_event.iter() {
        if let Ok((entity, origin)) = query.get_single() {
            let path = Path {
                destination: event.destination,
                origin: *origin,
                left_click: event.left_click,
            };
            commands.entity(entity).insert(path);
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Component, Default)]
pub struct PathMap {
    pub steps: Vec<(Tick, LeftClick, Tile)>,
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

                    LeftClick::Open(e) =>{
                        if let Some(server_entity) = network_mapping.client.get(e){
                            player_commands.send(PlayerCommand::LeftClick(LeftClick::Open(*server_entity), *tile));
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
