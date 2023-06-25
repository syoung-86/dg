use bevy::prelude::{
    Commands, DespawnRecursiveExt, Entity, EventReader, EventWriter, Query, Res, ResMut,
};
use bevy_renet::renet::RenetServer;
use lib::{
    channels::{ClientChannel, ServerChannel},
    components::{
        Action, CombatState, CoolDowns, EntityType, LeftClick, OpenState, PlayerCommand, Target,
    },
    resources::Tick,
    ClickEvent,
};

use crate::{resources::ServerLobby, CombatEvent, LeftClickEvent, MobState};

pub fn message(
    mut server: ResMut<RenetServer>,
    _item_query: Query<(Entity, &EntityType)>,
    mut left_click_event: EventWriter<LeftClickEvent>,
    mut combat_event: EventWriter<CombatEvent>,
    mut target_query: Query<(&Target, &mut CoolDowns)>,
    lobby: Res<ServerLobby>,
    tick: Res<Tick>,
    mut commands: Commands,
) {
    for client_id in server.clients_id().into_iter() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::Command) {
            let command: PlayerCommand = bincode::deserialize(&message).unwrap();
            //println!("receive  msg {:?}", command);
            match command {
                PlayerCommand::LeftClick(left_click, tile) => {
                    left_click_event.send(LeftClickEvent {
                        client_id,
                        left_click,
                        tile,
                    });
                }
                //CombatEvent using target: Entity, instead of Target(Entity)
                PlayerCommand::AutoAttack => {
                    if let Some(client) = lobby.clients.get(&client_id) {
                        if let Ok((target, mut cooldowns)) =
                            target_query.get_mut(client.controlled_entity)
                        {
                            if cooldowns.cd_auto_attack(&tick) {
                                if let Some(target) = target.0 {
                                    commands
                                        .entity(client.controlled_entity)
                                        .insert(CombatState::Punching(tick.tick + 5));
                                    combat_event.send(CombatEvent::new(target, Action::AutoAttack));
                                    println!("received autoattack")
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn left_click(
    mut server: ResMut<RenetServer>,
    mut commands: Commands,
    lobby: ResMut<ServerLobby>,
    mut left_click_event: EventReader<LeftClickEvent>,
) {
    for event in left_click_event.iter() {
        match event.left_click {
            LeftClick::Walk => {
                if let Some(client) = lobby.clients.get(&event.client_id) {
                    //println!("inserted new tile");
                    commands.entity(client.controlled_entity).insert(event.tile);
                    //let message = UpdateEvent {
                    //entity: client.controlled_entity,
                    //component: ComponentType::Tile(event.tile),
                    //};
                    //let serd_message = bincode::serialize(&message).unwrap();
                    //server.broadcast_message(ServerChannel::Update, serd_message);
                    //println!("walk");
                }
            }

            LeftClick::Pickup(Some(e)) => {
                //println!("pickup {:?}", e);
                if let Some(_player_entity) = lobby.clients.get(&event.client_id) {
                    //commands.entity(*player_entity).insert(tile);
                    //for (player, mut inventory) in players.iter_mut() {
                    //if *player_entity == player {
                    //for (item, item_id) in item_query.iter() {
                    //if e == item {
                    //let inventory_item = commands.spawn(*item_id).id();
                    //inventory.slots.insert(inventory_item);
                    //println!("inserted into inventory");
                    //let message = bincode::serialize(item_id).unwrap();
                    //server.send_message(
                    //client_id,
                    //ServerChannel::Test,
                    //message,
                    //);
                    //}
                    //}
                    //}
                    //}
                    commands.entity(e).despawn_recursive();
                    //println!("pickup {:?}", e);

                    let despawn_message = bincode::serialize(&e).unwrap();
                    server.broadcast_message(ServerChannel::Despawn, despawn_message);
                }
            }
            LeftClick::Attack(e) => {
                if let Some(client) = lobby.clients.get(&event.client_id) {
                    println!("inserted target");
                    commands
                        .entity(client.controlled_entity)
                        .insert(Target(Some(e)));
                    commands
                        .entity(e)
                        .insert(MobState::Combat(client.controlled_entity));
                }
            }
            LeftClick::Open(e) => {
                commands.entity(e).insert(OpenState::Open);
            }
            LeftClick::Close(e) => {
                commands.entity(e).insert(OpenState::Closed);
            }
            LeftClick::Pull => todo!(),
            LeftClick::Pickup(_) => todo!(),
            //_ => {}
        }
    }
}
pub fn clicks(lobby: Res<ServerLobby>, mut server: ResMut<RenetServer>) {
    for (client_id, _) in lobby.clients.iter() {
        if let Some(message) = server.receive_message(*client_id, ClientChannel::Click) {
            let click: ClickEvent = bincode::deserialize(&message).unwrap();
            println!("click event: {:?}", click);
        }
    }
}
