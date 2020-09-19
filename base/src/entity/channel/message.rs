use super::{
    super::{
        guild::{GuildEntity, RoleEntity},
        user::UserEntity,
    },
    AttachmentEntity, ChannelEntity, TextChannelEntity,
};
use crate::{
    repository::{GetEntityFuture, ListEntitiesFuture, Repository},
    utils, Backend, Entity,
};
use twilight_model::{
    channel::{
        embed::Embed,
        message::{MessageFlags, MessageReaction, MessageType},
    },
    id::{ApplicationId, AttachmentId, ChannelId, GuildId, MessageId, RoleId, UserId, WebhookId},
};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageEntity {
    pub application_id: ApplicationId,
    pub attachments: Vec<AttachmentId>,
    pub author_id: UserId,
    pub channel_id: ChannelId,
    pub content: String,
    pub edited_timestamp: Option<String>,
    pub embeds: Vec<Embed>,
    pub flags: Option<MessageFlags>,
    pub guild_id: Option<GuildId>,
    pub id: MessageId,
    pub kind: MessageType,
    pub mention_channels: Vec<ChannelId>,
    pub mention_everyone: bool,
    pub mention_roles: Vec<RoleId>,
    pub mentions: Vec<UserId>,
    pub pinned: bool,
    pub reactions: Vec<MessageReaction>,
    pub timestamp: String,
    pub tts: bool,
    pub webhook_id: Option<WebhookId>,
}

impl Entity for MessageEntity {
    type Id = MessageId;

    /// Return the message's ID.
    fn id(&self) -> Self::Id {
        self.id
    }
}

pub trait MessageRepository<B: Backend>: Repository<MessageEntity, B> + Send {
    fn attachments(
        &self,
        message_id: MessageId,
    ) -> ListEntitiesFuture<'_, AttachmentEntity, B::Error>;

    fn author(&self, message_id: MessageId) -> GetEntityFuture<'_, UserEntity, B::Error> {
        utils::relation_map(
            self.backend().messages(),
            self.backend().users(),
            message_id,
            |message| message.author_id,
        )
    }

    fn channel(&self, message_id: MessageId) -> GetEntityFuture<'_, ChannelEntity, B::Error>;

    fn guild(&self, message_id: MessageId) -> GetEntityFuture<'_, GuildEntity, B::Error> {
        utils::relation_and_then(
            self.backend().messages(),
            self.backend().guilds(),
            message_id,
            |message| message.guild_id,
        )
    }

    fn mention_channels(
        &self,
        message_id: MessageId,
    ) -> ListEntitiesFuture<'_, TextChannelEntity, B::Error>;

    fn mention_roles(&self, message_id: MessageId) -> ListEntitiesFuture<'_, RoleEntity, B::Error>;

    fn mentions(&self, message_id: MessageId) -> ListEntitiesFuture<'_, UserEntity, B::Error>;
}
