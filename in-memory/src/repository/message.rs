use crate::{config::EntityType, InMemoryBackendError, InMemoryBackendRef};
use futures_util::{
    future::{self, FutureExt},
    stream::{self, StreamExt},
};
use rarity_cache::{
    entity::{
        channel::{
            AttachmentEntity, ChannelEntity, GuildChannelEntity, MessageEntity, MessageRepository,
            TextChannelEntity,
        },
        guild::{GuildEntity, RoleEntity},
        user::UserEntity,
        Entity,
    },
    repository::{
        GetEntityFuture, ListEntitiesFuture, RemoveEntityFuture, Repository, UpsertEntityFuture,
    },
};
use std::sync::Arc;
use twilight_model::id::{ChannelId, MessageId};

/// Repository to retrieve and work with messages and their related entities.
#[derive(Clone, Debug)]
pub struct InMemoryMessageRepository(pub(crate) Arc<InMemoryBackendRef>);

impl InMemoryMessageRepository {
    /// Insert a message into a channel's set of message IDs.
    ///
    /// If the number of cached messages for the channel is equal to the size of
    /// the configured message cache, then the oldest message ID (meaning the
    /// lowest ID, not the oldest entry in the list) will be removed from the
    /// channel's list and from the message cache.
    ///
    /// This means that an old message that was updated and was not previously
    /// in the cache may be inserted and then immediately removed.
    fn insert_message_id(&self, channel_id: ChannelId, message_id: MessageId) {
        let cache_size = self.0.config.message_cache_size();

        if cache_size == 0 {
            return;
        }

        let mut channel_messages = self.0.channel_messages.entry(channel_id).or_default();
        channel_messages.insert(message_id);

        if channel_messages.len() < self.0.config.message_cache_size() {
            return;
        }

        // BTreeSets will iterate in order from the lowest ID entry, so we can
        // get the first entry this way. This should always be Some.
        //
        // `map_first_last` is on nightly which would allow using
        // `BTreeMap::first` instead.
        if let Some(oldest_message_id) = channel_messages.iter().next().copied() {
            channel_messages.remove(&oldest_message_id);
            self.0.messages.remove(&oldest_message_id);
        }
    }
}

impl Repository<MessageEntity, InMemoryBackendError> for InMemoryMessageRepository {
    fn get(
        &self,
        message_id: MessageId,
    ) -> GetEntityFuture<'_, MessageEntity, InMemoryBackendError> {
        future::ok(self.0.messages.get(&message_id).map(|r| r.value().clone())).boxed()
    }

    fn list(&self) -> ListEntitiesFuture<'_, MessageEntity, InMemoryBackendError> {
        let stream = stream::iter(self.0.messages.iter().map(|r| Ok(r.value().clone()))).boxed();

        future::ok(stream).boxed()
    }

    fn remove(&self, message_id: MessageId) -> RemoveEntityFuture<'_, InMemoryBackendError> {
        if !self.0.config.entity_types().contains(EntityType::MESSAGE) {
            return future::ok(()).boxed();
        }

        if let Some((_, message)) = self.0.messages.remove(&message_id) {
            if let Some(mut channel_messages) = self.0.channel_messages.get_mut(&message.channel_id)
            {
                channel_messages.remove(&message_id);
            }
        }

        future::ok(()).boxed()
    }

    fn upsert(&self, entity: MessageEntity) -> UpsertEntityFuture<'_, InMemoryBackendError> {
        if !self.0.config.entity_types().contains(EntityType::MESSAGE) {
            return future::ok(()).boxed();
        }

        let channel_id = entity.channel_id;

        if !self.0.messages.contains_key(&entity.id) {
            self.insert_message_id(channel_id, entity.id);
        }

        self.0.messages.insert(entity.id(), entity);

        future::ok(()).boxed()
    }
}

impl MessageRepository<InMemoryBackendError> for InMemoryMessageRepository {
    fn attachments(
        &self,
        message_id: MessageId,
    ) -> ListEntitiesFuture<'_, AttachmentEntity, InMemoryBackendError> {
        let attachment_ids = match self.0.messages.get(&message_id) {
            Some(message) => message.attachments.clone(),
            None => return future::ok(stream::empty().boxed()).boxed(),
        };

        let iter = attachment_ids
            .into_iter()
            .filter_map(move |id| self.0.attachments.get(&id).map(|r| Ok(r.value().clone())));
        let stream = stream::iter(iter).boxed();

        future::ok(stream).boxed()
    }

    fn author(
        &self,
        message_id: MessageId,
    ) -> GetEntityFuture<'_, UserEntity, InMemoryBackendError> {
        let author = self
            .0
            .messages
            .get(&message_id)
            .map(|message| message.author_id)
            .and_then(|id| self.0.users.get(&id))
            .map(|r| r.value().clone());

        future::ok(author).boxed()
    }

    fn channel(
        &self,
        message_id: MessageId,
    ) -> GetEntityFuture<'_, ChannelEntity, InMemoryBackendError> {
        let id = match self.0.messages.get(&message_id) {
            Some(message) => message.channel_id,
            None => return future::ok(None).boxed(),
        };

        if let Some(r) = self.0.channels_text.get(&id) {
            let entity = ChannelEntity::Guild(GuildChannelEntity::Text(r.value().clone()));

            return future::ok(Some(entity)).boxed();
        }

        if let Some(r) = self.0.channels_private.get(&id) {
            let entity = ChannelEntity::Private(r.value().clone());

            return future::ok(Some(entity)).boxed();
        }

        if let Some(r) = self.0.groups.get(&id) {
            let entity = ChannelEntity::Group(r.value().clone());

            return future::ok(Some(entity)).boxed();
        }

        future::ok(None).boxed()
    }

    fn guild(
        &self,
        message_id: MessageId,
    ) -> GetEntityFuture<'_, GuildEntity, InMemoryBackendError> {
        let guild = self
            .0
            .messages
            .get(&message_id)
            .and_then(|message| message.guild_id)
            .and_then(|id| self.0.guilds.get(&id))
            .map(|r| r.value().clone());

        future::ok(guild).boxed()
    }

    fn mention_channels(
        &self,
        message_id: MessageId,
    ) -> ListEntitiesFuture<'_, TextChannelEntity, InMemoryBackendError> {
        let channel_ids = match self.0.messages.get(&message_id) {
            Some(member) => member.mention_channels.clone(),
            None => return future::ok(stream::empty().boxed()).boxed(),
        };

        let iter = channel_ids
            .into_iter()
            .filter_map(move |id| self.0.channels_text.get(&id).map(|r| Ok(r.value().clone())));
        let stream = stream::iter(iter).boxed();

        future::ok(stream).boxed()
    }

    fn mention_roles(
        &self,
        message_id: MessageId,
    ) -> ListEntitiesFuture<'_, RoleEntity, InMemoryBackendError> {
        let role_ids = match self.0.messages.get(&message_id) {
            Some(member) => member.mention_roles.clone(),
            None => return future::ok(stream::empty().boxed()).boxed(),
        };

        let iter = role_ids
            .into_iter()
            .filter_map(move |id| self.0.roles.get(&id).map(|r| Ok(r.value().clone())));
        let stream = stream::iter(iter).boxed();

        future::ok(stream).boxed()
    }

    fn mentions(
        &self,
        message_id: MessageId,
    ) -> ListEntitiesFuture<'_, UserEntity, InMemoryBackendError> {
        let user_ids = match self.0.messages.get(&message_id) {
            Some(member) => member.mentions.clone(),
            None => return future::ok(stream::empty().boxed()).boxed(),
        };

        let iter = user_ids
            .into_iter()
            .filter_map(move |id| self.0.users.get(&id).map(|r| Ok(r.value().clone())));
        let stream = stream::iter(iter).boxed();

        future::ok(stream).boxed()
    }
}

impl InMemoryMessageRepository {
    pub fn attachments(
        &self,
        message_id: MessageId,
    ) -> ListEntitiesFuture<'_, AttachmentEntity, InMemoryBackendError> {
        MessageRepository::attachments(self, message_id)
    }

    /// Retrieve the author of a message.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rarity_cache_inmemory::InMemoryCache;
    /// use twilight_model::id::MessageId;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let cache = InMemoryCache::new();
    ///
    /// if let Some(author) = cache.messages.author(MessageId(123456)).await? {
    ///     println!("the author's username is {}", author.name);
    /// }
    /// # Ok(()) }
    /// ```
    pub fn author(
        &self,
        message_id: MessageId,
    ) -> GetEntityFuture<'_, UserEntity, InMemoryBackendError> {
        MessageRepository::author(self, message_id)
    }

    /// Retrieve the channel of a message.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rarity_cache_inmemory::InMemoryCache;
    /// use twilight_model::id::MessageId;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let cache = InMemoryCache::new();
    ///
    /// if let Some(channel) = cache.messages.channel(MessageId(123456)).await? {
    ///     println!("{:?}", channel);
    /// }
    /// # Ok(()) }
    /// ```
    pub fn channel(
        &self,
        message_id: MessageId,
    ) -> GetEntityFuture<'_, ChannelEntity, InMemoryBackendError> {
        MessageRepository::channel(self, message_id)
    }

    /// Retrieve the guild a message was posted in.
    ///
    /// This will return `None` if the message or guild is not in the cache.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rarity_cache_inmemory::InMemoryCache;
    /// use twilight_model::id::MessageId;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let cache = InMemoryCache::new();
    ///
    /// if let Some(guild) = cache.messages.guild(MessageId(123456)).await? {
    ///     println!("the guild that the message was posted in is '{}'", guild.name);
    /// }
    /// # Ok(()) }
    /// ```
    pub fn guild(
        &self,
        message_id: MessageId,
    ) -> GetEntityFuture<'_, GuildEntity, InMemoryBackendError> {
        MessageRepository::guild(self, message_id)
    }

    pub fn mention_channels(
        &self,
        message_id: MessageId,
    ) -> ListEntitiesFuture<'_, TextChannelEntity, InMemoryBackendError> {
        MessageRepository::mention_channels(self, message_id)
    }

    pub fn mention_roles(
        &self,
        message_id: MessageId,
    ) -> ListEntitiesFuture<'_, RoleEntity, InMemoryBackendError> {
        MessageRepository::mention_roles(self, message_id)
    }

    pub fn mentions(
        &self,
        message_id: MessageId,
    ) -> ListEntitiesFuture<'_, UserEntity, InMemoryBackendError> {
        MessageRepository::mentions(self, message_id)
    }
}
