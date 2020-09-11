use super::{super::user::UserEntity, MessageEntity};
use crate::{
    repository::{GetEntityFuture, Repository},
    Entity,
};
use twilight_model::{
    channel::{ChannelType, PrivateChannel},
    id::{ChannelId, MessageId, UserId},
};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrivateChannelEntity {
    pub id: ChannelId,
    pub last_message_id: Option<MessageId>,
    pub last_pin_timestamp: Option<String>,
    pub kind: ChannelType,
    pub recipient_id: Option<UserId>,
}

impl From<PrivateChannel> for PrivateChannelEntity {
    fn from(channel: PrivateChannel) -> Self {
        Self {
            id: channel.id,
            last_message_id: channel.last_message_id,
            last_pin_timestamp: channel.last_pin_timestamp,
            kind: channel.kind,
            recipient_id: channel.recipients.first().map(|user| user.id),
        }
    }
}

impl Entity for PrivateChannelEntity {
    type Id = ChannelId;

    /// Return the private channel's ID.
    fn id(&self) -> Self::Id {
        self.id
    }
}

/// Repository to work with guild channels and their associated entities.
pub trait PrivateChannelRepository<Error: 'static>:
    Repository<PrivateChannelEntity, Error>
{
    /// Retrieve the last message of a private channel.
    fn last_message(&self, group_id: ChannelId) -> GetEntityFuture<'_, MessageEntity, Error>;

    /// Retrieve the recipient user associated with a private channel.
    fn recipient(&self, channel_id: ChannelId) -> GetEntityFuture<'_, UserEntity, Error>;
}
