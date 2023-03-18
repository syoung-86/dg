use std::time::Duration;

use bevy::{
    ecs::schedule::{LogLevel, ScheduleBuildSettings},
    prelude::*,
};
use bevy_renet::{renet::RenetClient, RenetClientPlugin};
use camera::{camera_follow, setup_camera};
use connection::{new_renet_client, server_messages};
use resources::{ClientLobby, NetworkMapping};
use shared::{
    channels::ServerChannel,
    components::{EntityType, TilePos},
};

use rand::Rng;
pub mod camera;
pub mod components;
pub mod connection;
pub mod plugins;
pub mod resources;
pub mod run_conditions;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugin(RenetClientPlugin {
        clear_events: false,
    });

    app.insert_resource(FixedTime::new(Duration::from_millis(100)));
    app.edit_schedule(CoreSchedule::Main, |schedule| {
        schedule.set_build_settings(ScheduleBuildSettings {
            ambiguity_detection: LogLevel::Ignore,
            ..default()
        });
    });

    //app.add_system(fixed_time.in_schedule(CoreSchedule::FixedUpdate));
    app.add_startup_system(setup_camera);
    app.add_system(server_messages);
    app.add_system(camera_follow);
    app.add_system(load);
    app.insert_resource(new_renet_client());
    app.add_systems(().distributive_run_if(bevy_renet::client_connected));
    app.insert_resource(NetworkMapping::default());
    app.insert_resource(ClientLobby::default());
    //app.add_system(run_fixed_update_schedule);
    app.run();
}

pub fn load(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut client: ResMut<RenetClient>,
    mut commands: Commands,
    mut network_mapping: ResMut<NetworkMapping>,
) {
    if let Some(message) = client.receive_message(ServerChannel::Load) {
        let scope: Vec<(Entity, EntityType, TilePos)> = bincode::deserialize(&message).unwrap();

        for (e, e_id, t) in scope {
            //println!("tile: {:?}", t);
            if e_id == EntityType::Tile {
                let mut rng = rand::thread_rng();
                let color: f32 = rng.gen_range(0.9..1.0);
                let transform = t.to_transform();
                let child = commands
                    .spawn((
                        PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Box::new(1., 0.2, 1.))),
                            material: materials.add(Color::rgb(0.2, 0.5, 0.2).into()),
                            transform,
                            ..Default::default()
                        },
                        //PickableBundle::default(),
                        //PickRaycastTarget::default(),
                        //NoDeselect,
                        t,
                        //LeftClick::default(),
                    ))
                    //.forward_events::<PointerDown, PickingEvent>()
                    .id();
                network_mapping.client_to_server.insert(child, e);
            }
        }

        commands.spawn(PointLightBundle {
            point_light: PointLight {
                intensity: 1500.0,
                shadows_enabled: true,
                ..Default::default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..Default::default()
        });
    }
}
