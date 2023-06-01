use bevy_renet::renet::{
    ChannelConfig, ChunkChannelConfig, ReliableChannelConfig, UnreliableChannelConfig,
};
use std::time::Duration;

pub enum ClientChannel {
    Command,
    Input,
    Click,
}

impl From<ClientChannel> for u8 {
    fn from(channel_id: ClientChannel) -> Self {
        match channel_id {
            ClientChannel::Command => 0,
            ClientChannel::Input => 1,
            ClientChannel::Click => 2,
        }
    }
}

impl ClientChannel {
    #[must_use]
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![
            UnreliableChannelConfig {
                channel_id: Self::Input.into(),
                sequenced: true,
                ..Default::default()
            }
            .into(),
            UnreliableChannelConfig {
                channel_id: Self::Command.into(),
                sequenced: true,
                ..Default::default()
            }
            .into(),
            UnreliableChannelConfig {
                channel_id: Self::Click.into(),
                sequenced: false,
                ..Default::default()
            }
            .into(),
        ]
    }
}

pub enum ServerChannel {
    Spawn,
    Despawn,
    Update,
    Load,
    ServerMessages,
    Tick,
    Test,
    ServerEvents,
}

impl From<ServerChannel> for u8 {
    fn from(channel_id: ServerChannel) -> Self {
        match channel_id {
            ServerChannel::Spawn => 0,
            ServerChannel::Despawn => 1,
            ServerChannel::Update => 2,
            ServerChannel::Load => 3,
            ServerChannel::ServerMessages => 4,
            ServerChannel::Tick => 5,
            ServerChannel::Test => 6,
            ServerChannel::ServerEvents => 7,
        }
    }
}

impl ServerChannel {
    #[must_use]
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec![
            ReliableChannelConfig {
                channel_id: Self::Spawn.into(),
                message_resend_time: Duration::from_millis(200),
                ..Default::default()
            }
            .into(),
            ReliableChannelConfig {
                channel_id: Self::Despawn.into(),
                message_resend_time: Duration::from_millis(200),
                ..Default::default()
            }
            .into(),
            UnreliableChannelConfig {
                channel_id: Self::Update.into(),
                sequenced: false,
                ..Default::default() //ReliableChannelConfig {
                                     //channel_id: Self::Update.into(),
                                     //message_resend_time: Duration::from_millis(200),
                                     //..Default::default()
            }
            .into(),
            ChunkChannelConfig {
                channel_id: Self::Load.into(),
                ..Default::default()
            }
            .into(),
            ReliableChannelConfig {
                channel_id: Self::ServerMessages.into(),
                message_resend_time: Duration::from_millis(200),
                ..Default::default()
            }
            .into(),
            UnreliableChannelConfig {
                channel_id: Self::Tick.into(),
                sequenced: true,
                ..Default::default()
            }
            .into(),
            ReliableChannelConfig {
                channel_id: Self::Test.into(),
                message_resend_time: Duration::from_millis(200),
                ..Default::default()
            }
            .into(),
            ReliableChannelConfig {
                channel_id: Self::ServerEvents.into(),
                message_resend_time: Duration::from_millis(200),
                ..Default::default()
            }
            .into(),
        ]
    }
}
