use super::super::guild::GuildEntity;
use crate::{
    repository::{GetEntityFuture, Repository},
    Entity,
};
use twilight_model::{
    channel::{permission_overwrite::PermissionOverwrite, CategoryChannel, ChannelType},
    id::{ChannelId, GuildId},
};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CategoryChannelEntity {
    pub guild_id: Option<GuildId>,
    pub id: ChannelId,
    pub kind: ChannelType,
    pub name: String,
    pub permission_overwrites: Vec<PermissionOverwrite>,
    pub position: i64,
}

impl From<CategoryChannel> for CategoryChannelEntity {
    fn from(channel: CategoryChannel) -> Self {
        Self {
            guild_id: channel.guild_id,
            id: channel.id,
            kind: channel.kind,
            name: channel.name,
            permission_overwrites: channel.permission_overwrites,
            position: channel.position,
        }
    }
}

impl Entity for CategoryChannelEntity {
    type Id = ChannelId;

    /// Return the category channel's ID.
    fn id(&self) -> Self::Id {
        self.id
    }
}

/// Repository to work with guild category channels and their associated
/// entities.
pub trait CategoryChannelRepository<Error: 'static>:
    Repository<CategoryChannelEntity, Error>
{
    /// Retrieve the guild associated with a guild category channel.
    fn guild(&self, channel_id: ChannelId) -> GetEntityFuture<'_, GuildEntity, Error>;
}
