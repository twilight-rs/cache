use super::{
    super::{
        guild::{GuildEntity, RoleEntity},
        user::UserEntity,
    },
    AttachmentEntity, ChannelEntity, GuildChannelEntity, TextChannelEntity,
};
use crate::{
    repository::{GetEntityFuture, ListEntitiesFuture, Repository},
    utils, Backend, Entity,
};
use twilight_model::{
    channel::{
        embed::Embed,
        message::{MessageActivity, MessageFlags, MessageReaction, MessageType},
        Message,
    },
    gateway::payload::MessageUpdate,
    id::{ApplicationId, AttachmentId, ChannelId, GuildId, MessageId, RoleId, UserId, WebhookId},
};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageEntity {
    pub activity: Option<MessageActivity>,
    pub application_id: Option<ApplicationId>,
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

impl From<Message> for MessageEntity {
    fn from(message: Message) -> Self {
        let application_id = message.application.map(|a| a.id);

        let attachments = message
            .attachments
            .into_iter()
            .map(|attachment| attachment.id)
            .collect();

        let mention_channels = message
            .mention_channels
            .into_iter()
            .map(|mention_channel| mention_channel.id)
            .collect();

        let mentions = message
            .mentions
            .into_iter()
            .map(|mention| mention.0)
            .collect();

        Self {
            activity: message.activity,
            application_id,
            attachments,
            author_id: message.author.id,
            channel_id: message.channel_id,
            content: message.content,
            edited_timestamp: message.edited_timestamp,
            embeds: message.embeds,
            flags: message.flags,
            guild_id: message.guild_id,
            id: message.id,
            kind: message.kind,
            mention_channels,
            mention_everyone: message.mention_everyone,
            mention_roles: message.mention_roles,
            mentions,
            pinned: message.pinned,
            reactions: message.reactions,
            timestamp: message.timestamp,
            tts: message.tts,
            webhook_id: message.webhook_id,
        }
    }
}

impl MessageEntity {
    pub fn update(self, update: MessageUpdate) -> Self {
        let attachments = update
            .attachments
            .map_or(self.attachments, |a| a.into_iter().map(|a| a.id).collect());

        let mentions = update
            .mentions
            .map_or(self.mentions, |m| m.into_iter().map(|m| m.id).collect());

        Self {
            attachments,
            author_id: update.author.map_or(self.author_id, |a| a.id),
            channel_id: update.channel_id,
            content: update.content.map_or(self.content, |m| m),
            edited_timestamp: update.edited_timestamp.or(self.edited_timestamp),
            embeds: update.embeds.map_or(self.embeds, |e| e),
            guild_id: update.guild_id.or(self.guild_id),
            id: update.id,
            kind: update.kind.map_or(self.kind, |k| k),
            mention_everyone: update.mention_everyone.map_or(self.mention_everyone, |m| m),
            mention_roles: update.mention_roles.map_or(self.mention_roles, |m| m),
            mentions,
            pinned: update.pinned.map_or(self.pinned, |p| p),
            timestamp: update.timestamp.map_or(self.timestamp, |t| t),
            tts: update.tts.map_or(self.tts, |t| t),
            ..self
        }
    }
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
    ) -> ListEntitiesFuture<'_, AttachmentEntity, B::Error> {
        utils::stream(
            self.backend().messages(),
            self.backend().attachments(),
            message_id,
            |message| message.attachments.into_iter(),
        )
    }

    fn author(&self, message_id: MessageId) -> GetEntityFuture<'_, UserEntity, B::Error> {
        utils::relation_map(
            self.backend().messages(),
            self.backend().users(),
            message_id,
            |message| message.author_id,
        )
    }

    fn channel(&self, message_id: MessageId) -> GetEntityFuture<'_, ChannelEntity, B::Error> {
        let backend = self.backend();

        Box::pin(async move {
            let messages = backend.messages();

            let channel_id = if let Some(msg) = messages.get(message_id).await? {
                msg.channel_id
            } else {
                return Ok(None);
            };

            let text_channels = backend.text_channels();

            if let Some(channel) = text_channels.get(channel_id).await? {
                return Ok(Some(ChannelEntity::Guild(GuildChannelEntity::Text(
                    channel,
                ))));
            }

            let private_channels = backend.private_channels();

            if let Some(channel) = private_channels.get(channel_id).await? {
                return Ok(Some(ChannelEntity::Private(channel)));
            }

            let groups = backend.groups();

            if let Some(channel) = groups.get(channel_id).await? {
                return Ok(Some(ChannelEntity::Group(channel)));
            }

            Ok(None)
        })
    }

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
    ) -> ListEntitiesFuture<'_, TextChannelEntity, B::Error> {
        utils::stream(
            self.backend().messages(),
            self.backend().text_channels(),
            message_id,
            |message| message.mention_channels.into_iter(),
        )
    }

    fn mention_roles(&self, message_id: MessageId) -> ListEntitiesFuture<'_, RoleEntity, B::Error> {
        utils::stream(
            self.backend().messages(),
            self.backend().roles(),
            message_id,
            |message| message.mention_roles.into_iter(),
        )
    }

    fn mentions(&self, message_id: MessageId) -> ListEntitiesFuture<'_, UserEntity, B::Error> {
        utils::stream(
            self.backend().messages(),
            self.backend().users(),
            message_id,
            |message| message.mentions.into_iter(),
        )
    }
}
