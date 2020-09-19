use super::{super::guild::GuildEntity, CategoryChannelEntity};
use crate::{
    repository::{GetEntityFuture, Repository},
    utils, Backend, Entity,
};
use twilight_model::{
    channel::{permission_overwrite::PermissionOverwrite, ChannelType, VoiceChannel},
    id::{ChannelId, GuildId},
};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VoiceChannelEntity {
    pub bitrate: u64,
    pub guild_id: Option<GuildId>,
    pub id: ChannelId,
    pub kind: ChannelType,
    pub name: String,
    pub permission_overwrites: Vec<PermissionOverwrite>,
    pub parent_id: Option<ChannelId>,
    pub position: i64,
    pub user_limit: Option<u64>,
}

impl From<VoiceChannel> for VoiceChannelEntity {
    fn from(channel: VoiceChannel) -> Self {
        Self {
            bitrate: channel.bitrate,
            guild_id: channel.guild_id,
            id: channel.id,
            kind: channel.kind,
            name: channel.name,
            permission_overwrites: channel.permission_overwrites,
            parent_id: channel.parent_id,
            position: channel.position,
            user_limit: channel.user_limit,
        }
    }
}

impl Entity for VoiceChannelEntity {
    type Id = ChannelId;

    /// Return the voice channel's ID.
    fn id(&self) -> Self::Id {
        self.id
    }
}

/// Repository to work with guild voice channels and their associated entities.
pub trait VoiceChannelRepository<B: Backend>: Repository<VoiceChannelEntity, B> {
    /// Retrieve the guild associated with a guild voice channel.
    fn guild(&self, channel_id: ChannelId) -> GetEntityFuture<'_, GuildEntity, B::Error> {
        utils::relation_and_then(
            self.backend().voice_channels(),
            self.backend().guilds(),
            channel_id,
            |channel| channel.guild_id,
        )
    }

    /// Retrieve the parent category channel of the voice channel.
    fn parent(
        &self,
        channel_id: ChannelId,
    ) -> GetEntityFuture<'_, CategoryChannelEntity, B::Error> {
        utils::relation_and_then(
            self.backend().voice_channels(),
            self.backend().category_channels(),
            channel_id,
            |channel| channel.parent_id,
        )
    }
}
