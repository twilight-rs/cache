use super::{super::guild::GuildEntity, CategoryChannelEntity, MessageEntity};
use crate::{
    repository::{GetEntityFuture, Repository},
    utils, Backend, Entity,
};
use twilight_model::{
    channel::{permission_overwrite::PermissionOverwrite, ChannelType, TextChannel},
    id::{ChannelId, GuildId, MessageId},
};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextChannelEntity {
    pub guild_id: Option<GuildId>,
    pub id: ChannelId,
    pub kind: ChannelType,
    pub last_message_id: Option<MessageId>,
    pub last_pin_timestamp: Option<String>,
    pub name: String,
    pub nsfw: bool,
    pub permission_overwrites: Vec<PermissionOverwrite>,
    pub parent_id: Option<ChannelId>,
    pub position: i64,
    pub rate_limit_per_user: Option<u64>,
    pub topic: Option<String>,
}

impl From<TextChannel> for TextChannelEntity {
    fn from(channel: TextChannel) -> Self {
        Self {
            guild_id: channel.guild_id,
            id: channel.id,
            kind: channel.kind,
            last_message_id: channel.last_message_id,
            last_pin_timestamp: channel.last_pin_timestamp,
            name: channel.name,
            nsfw: channel.nsfw,
            permission_overwrites: channel.permission_overwrites,
            parent_id: channel.parent_id,
            position: channel.position,
            rate_limit_per_user: channel.rate_limit_per_user,
            topic: channel.topic,
        }
    }
}

impl Entity for TextChannelEntity {
    type Id = ChannelId;

    /// Return the text channel's ID.
    fn id(&self) -> Self::Id {
        self.id
    }
}

/// Repository to work with guild text channels and their associated entities.
pub trait TextChannelRepository<B: Backend>: Repository<TextChannelEntity, B> {
    /// Retrieve the guild associated with a guild text channel.
    fn guild(&self, channel_id: ChannelId) -> GetEntityFuture<'_, GuildEntity, B::Error> {
        utils::relation_and_then(
            self.backend().text_channels(),
            self.backend().guilds(),
            channel_id,
            |channel| channel.guild_id,
        )
    }

    /// Retrieve the last message of a text channel.
    fn last_message(&self, channel_id: ChannelId) -> GetEntityFuture<'_, MessageEntity, B::Error> {
        utils::relation_and_then(
            self.backend().text_channels(),
            self.backend().messages(),
            channel_id,
            |channel| channel.last_message_id,
        )
    }

    /// Retrieve the parent category channel of the voice channel.
    fn parent(
        &self,
        channel_id: ChannelId,
    ) -> GetEntityFuture<'_, CategoryChannelEntity, B::Error> {
        utils::relation_and_then(
            self.backend().text_channels(),
            self.backend().category_channels(),
            channel_id,
            |channel| channel.parent_id,
        )
    }
}
