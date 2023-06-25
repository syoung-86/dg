use std::time::Duration;

use bevy::prelude::*;
use bevy_renet::{renet::RenetServer, RenetServerPlugin};
use connection::{client_handler, new_renet_server};
use events::{ChunkRequest, ClientSetup};
use lib::{
    channels::ServerChannel,
    components::{
        Action, Arch, Client, CombatState, Direction, Door, Dummy, EntityType, Health, LeftClick,
        OpenState, Player, Scope, Slime, SpawnEvent, SyncEvent, Tile, Untraversable, Wall,
    },
    resources::Tick,
    TickSet,
};
use plugins::{ClearEventPlugin, ConfigPlugin};
use rand::Rng;
use receive::{left_click, message};
use resources::ServerLobby;
use seldom_state::prelude::*;
use send::spawn;
use sync::{
    create_scope, entered_left_scope, send_chunk, send_updates, update_combat_state, update_health,
    update_open_state, update_target, update_tile,
};
use world::create_tiles;

pub mod connection;
pub mod events;
pub mod plugins;
pub mod receive;
pub mod resources;
pub mod send;
pub mod state;
pub mod sync;
pub mod world;

fn main() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugin(ConfigPlugin);
    app.add_plugin(ClearEventPlugin);
    app.add_plugin(StateMachinePlugin);

    app.insert_resource(FixedTime::new(Duration::from_millis(100)));
    app.insert_resource(Tick::default());
    app.insert_resource(new_renet_server());
    app.init_resource::<ServerLobby>();
    app.init_resource::<Events<ChunkRequest>>();
    app.init_resource::<Events<ClientSetup>>();
    app.init_resource::<Events<LeftClickEvent>>();
    app.init_resource::<Events<SpawnEvent>>();
    app.init_resource::<Events<SyncEvent>>();
    app.init_resource::<Events<CombatEvent>>();
    app.add_systems(
        (tick, send_tick)
            .chain()
            .in_schedule(CoreSchedule::FixedUpdate),
    );
    app.add_system(
        client_handler
            .in_set(TickSet::Connection)
            .in_schedule(CoreSchedule::FixedUpdate),
    );
    app.add_system(
        send_chunk
            .in_set(TickSet::SendChunk)
            .in_schedule(CoreSchedule::FixedUpdate),
    );
    app.add_systems(
        (
            create_scope,
            entered_left_scope,
            message,
            left_click,
            combat_events,
            update_tile,
            update_health,
            update_target,
            update_combat_state,
            update_open_state,
            send_updates,
            move_slime,
        )
            .chain()
            .in_schedule(CoreSchedule::FixedUpdate),
    );
    app.add_systems(
        (RenetServerPlugin::get_clear_event_systems().in_set(TickSet::Clear))
            .in_schedule(CoreSchedule::FixedUpdate),
    );
    app.add_startup_system(create_tiles);
    app.add_startup_system(spawn_room.after(create_tiles));
    app.add_startup_system(spawn_dummy);
    app.add_startup_system(spawn_slime);
    app.add_event::<ClientSetup>();
    app.run();
}
#[derive(Component)]
pub enum MobState {
    Wonder(Direction),
    Combat(Entity),
}
#[derive(Component)]
pub struct MobRange {
    pub top_left: Tile,
    pub bottom_right: Tile,
}
impl MobRange {
    pub fn check(&self, pos: &Tile) -> bool {
        let x = pos.cell.0;
        let z = pos.cell.2;

        let tl_x = self.top_left.cell.0;
        let tl_z = self.top_left.cell.2;

        let br_x = self.bottom_right.cell.0;
        let br_z = self.bottom_right.cell.2;

        x >= tl_x && x <= br_x && z >= tl_z && z <= br_z
    }
}
pub fn move_slime(
    mut query: Query<(Entity, &mut Tile, &MobState, &MobRange), With<Slime>>,
    tick: Res<Tick>,
    mut commands: Commands,
) {
    if tick.tick % 10 == 0 {
        for (e, mut t, state, range) in query.iter_mut() {
            if tick.tick % 25 == 0 {
                let mut rng = rand::thread_rng();
                let x: u32 = rng.gen_range(0..4);
                if x == 0 {
                    commands.entity(e).insert(MobState::Wonder(Direction::East));
                }
                if x == 1 {
                    commands
                        .entity(e)
                        .insert(MobState::Wonder(Direction::South));
                }
                if x == 2 {
                    commands.entity(e).insert(MobState::Wonder(Direction::West));
                }
                if x == 3 {
                    commands
                        .entity(e)
                        .insert(MobState::Wonder(Direction::North));
                }
            }
            match state {
                MobState::Wonder(direction) => match direction {
                    Direction::Bad => (),
                    Direction::North => {
                        if range.check(&Tile::new((t.cell.0, t.cell.1, t.cell.2 + 1))) {
                            t.cell.2 += 1;
                        }
                    }
                    Direction::NorthEast => (),
                    Direction::East => {
                        if range.check(&Tile::new((t.cell.0 + 1, t.cell.1, t.cell.2))) {
                            t.cell.0 += 1;
                        }
                    }
                    Direction::SouthEast => (),
                    Direction::South => {
                        if range.check(&Tile::new((t.cell.0, t.cell.1, t.cell.2 - 1))) {
                            t.cell.2 -= 1;
                        }
                    }
                    Direction::SouthWest => (),
                    Direction::West => {
                        if range.check(&Tile::new((t.cell.0 - 1, t.cell.1, t.cell.2))) {
                            t.cell.0 -= 1;
                        }
                    }

                    Direction::NorthWest => (),
                },
                MobState::Combat(_) => (),
            }
        }
    }
}
const ROOM_SIZE: u32 = 14;
pub fn spawn_room(mut commands: Commands) {
    for z in 0..ROOM_SIZE {
        commands.spawn((EntityType::Wall(Wall::Horizontal), Tile::new((0, 0, z))));
    }

    for z in 0..ROOM_SIZE {
        if z >= 7 && z <= 8 {
            continue;
        }
        if z == 6 {
            commands.spawn((
                EntityType::Arch(Arch::Horizontal),
                Tile::new((ROOM_SIZE - 1, 0, z)),
            ));
            commands.spawn((
                EntityType::Door(Door::Horizontal),
                Tile::new((ROOM_SIZE - 1, 0, z)),
                OpenState::Closed,
            ));
        } else {
            commands.spawn((
                EntityType::Wall(Wall::Horizontal),
                Tile::new((ROOM_SIZE - 1, 0, z)),
            ));
        }
    }
    for x in 0..ROOM_SIZE {
        //if x >= 7 && x <= 8 {
        //continue;
        //}
        //if x == 6 {
        //commands.spawn((EntityType::Arch(Arch::Vertical), Tile::new((x, 0, 0))));
        //commands.spawn((EntityType::Door(Door::Vertical), Tile::new((x, 0, 0))));
        //} else {
        commands.spawn((EntityType::Wall(Wall::Horizontal), Tile::new((x, 0, 0))));
        //}
    }

    for x in 0..ROOM_SIZE {
        if x >= 7 && x <= 8 {
            continue;
        }
        if x == 6 {
            commands.spawn((
                EntityType::Arch(Arch::Vertical),
                Tile::new((x, 0, ROOM_SIZE - 1)),
            ));
            commands.spawn((
                EntityType::Door(Door::Vertical),
                Tile::new((x, 0, ROOM_SIZE - 1)),
            ));
        } else {
            commands.spawn((
                EntityType::Wall(Wall::Horizontal),
                Tile::new((x, 0, ROOM_SIZE - 1)),
            ));
        }
    }
}
pub fn combat_events(
    mut query: Query<(Entity, &mut Health)>,
    mut combat_event: EventReader<CombatEvent>,
) {
    for event in combat_event.iter() {
        match event.action {
            Action::AutoAttack => {
                if let Ok((e, mut target_health)) = query.get_mut(event.target) {
                    if target_health.hp >= 10 {
                        target_health.hp -= 10;
                    } else {
                        target_health.hp = 100;
                    }
                }
            }
        }
    }
}

pub fn spawn_slime(mut commands: Commands, mut spawn_event: EventWriter<SpawnEvent>) {
    let id = commands
        .spawn((
            Slime,
            EntityType::Slime(Slime),
            Health::new(99),
            Tile::new((4, 0, 4)),
            MobState::Wonder(Direction::East),
            MobRange {
                top_left: Tile::new((1, 0, 1)),
                bottom_right: Tile::new((10, 0, 10)),
            },
        ))
        .id();
    commands.entity(id).insert(LeftClick::Attack(id));
    spawn_event.send(SpawnEvent::new(
        id,
        EntityType::Slime(Slime),
        Tile::new((4, 0, 4)),
    ));
}
pub fn spawn_dummy(mut commands: Commands, mut spawn_event: EventWriter<SpawnEvent>) {
    let id = commands
        .spawn((
            EntityType::Dummy(Dummy),
            Health::new(99),
            Tile::new((1, 0, 1)),
        ))
        .id();
    spawn_event.send(SpawnEvent::new(
        id,
        EntityType::Dummy(Dummy),
        Tile::new((1, 0, 1)),
    ));
}
pub fn change_health(mut query: Query<&mut Health>, tick: Res<Tick>) {
    for mut hp in query.iter_mut() {
        if tick.tick % 10 == 0 {
            hp.hp += 1;
        }
    }
}
#[derive(Debug)]
pub struct LeftClickEvent {
    pub client_id: u64,
    pub left_click: LeftClick,
    pub tile: Tile,
}

#[derive(Debug)]
pub struct CombatEvent {
    pub action: Action,
    pub target: Entity,
}

impl CombatEvent {
    pub fn new(target: Entity, action: Action) -> Self {
        Self { action, target }
    }
}

//#[bevycheck::system]
pub fn tick(mut tick: ResMut<Tick>) {
    tick.tick += 1;
}
pub fn send_tick(mut server: ResMut<RenetServer>, tick: Res<Tick>) {
    let tick = Tick { tick: tick.tick };
    let message = bincode::serialize(&tick).unwrap();
    server.broadcast_message(ServerChannel::Tick, message)
}
