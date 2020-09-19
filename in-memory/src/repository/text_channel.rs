use crate::{config::EntityType, InMemoryBackend, InMemoryBackendError};
use futures_util::{
    future::{self, FutureExt},
    stream::{self, StreamExt},
};
use rarity_cache::{
    entity::{
        channel::{CategoryChannelEntity, MessageEntity, TextChannelEntity, TextChannelRepository},
        guild::GuildEntity,
        Entity,
    },
    repository::{
        GetEntityFuture, ListEntitiesFuture, RemoveEntityFuture, Repository, UpsertEntityFuture,
    },
};
use twilight_model::id::ChannelId;

/// Repository to retrieve and work with text channels and their related
/// entities.
#[derive(Clone, Debug)]
pub struct InMemoryTextChannelRepository(pub(crate) InMemoryBackend);

impl Repository<TextChannelEntity, InMemoryBackend> for InMemoryTextChannelRepository {
    fn backend(&self) -> &InMemoryBackend {
        &self.0
    }

    fn get(
        &self,
        channel_id: ChannelId,
    ) -> GetEntityFuture<'_, TextChannelEntity, InMemoryBackendError> {
        future::ok(
            self.0
                 .0
                .channels_text
                .get(&channel_id)
                .map(|r| r.value().clone()),
        )
        .boxed()
    }

    fn list(&self) -> ListEntitiesFuture<'_, TextChannelEntity, InMemoryBackendError> {
        let stream = stream::iter(
            (self.0)
                .0
                .channels_text
                .iter()
                .map(|r| Ok(r.value().clone())),
        )
        .boxed();

        future::ok(stream).boxed()
    }

    fn remove(&self, channel_id: ChannelId) -> RemoveEntityFuture<'_, InMemoryBackendError> {
        if !self
            .0
             .0
            .config
            .entity_types()
            .contains(EntityType::CHANNEL_TEXT)
        {
            return future::ok(()).boxed();
        }

        (self.0).0.channels_text.remove(&channel_id);

        future::ok(()).boxed()
    }

    fn upsert(&self, entity: TextChannelEntity) -> UpsertEntityFuture<'_, InMemoryBackendError> {
        if !self
            .0
             .0
            .config
            .entity_types()
            .contains(EntityType::CHANNEL_TEXT)
        {
            return future::ok(()).boxed();
        }

        (self.0).0.channels_text.insert(entity.id(), entity);

        future::ok(()).boxed()
    }
}

impl TextChannelRepository<InMemoryBackend> for InMemoryTextChannelRepository {
    fn guild(
        &self,
        channel_id: ChannelId,
    ) -> GetEntityFuture<'_, GuildEntity, InMemoryBackendError> {
        let guild = self
            .0
             .0
            .channels_text
            .get(&channel_id)
            .and_then(|channel| channel.guild_id)
            .and_then(|id| (self.0).0.guilds.get(&id))
            .map(|r| r.value().clone());

        future::ok(guild).boxed()
    }

    fn last_message(
        &self,
        channel_id: ChannelId,
    ) -> GetEntityFuture<'_, MessageEntity, InMemoryBackendError> {
        let message = self
            .0
             .0
            .channels_text
            .get(&channel_id)
            .and_then(|channel| channel.last_message_id)
            .and_then(|id| (self.0).0.messages.get(&id))
            .map(|r| r.value().clone());

        future::ok(message).boxed()
    }

    fn parent(
        &self,
        channel_id: ChannelId,
    ) -> GetEntityFuture<'_, CategoryChannelEntity, InMemoryBackendError> {
        let parent = self
            .0
             .0
            .channels_text
            .get(&channel_id)
            .and_then(|channel| channel.parent_id)
            .and_then(|id| (self.0).0.channels_category.get(&id))
            .map(|r| r.value().clone());

        future::ok(parent).boxed()
    }
}

impl InMemoryTextChannelRepository {
    /// Retrieve the guild of a text channel.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rarity_cache_inmemory::InMemoryCache;
    /// use twilight_model::id::ChannelId;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let cache = InMemoryCache::new();
    ///
    /// if let Some(guild) = cache.text_channels.guild(ChannelId(1)).await? {
    ///     println!("the guild's name is {}", guild.name);
    /// }
    /// # Ok(()) }
    /// ```
    pub fn guild(
        &self,
        channel_id: ChannelId,
    ) -> GetEntityFuture<'_, GuildEntity, InMemoryBackendError> {
        TextChannelRepository::guild(self, channel_id)
    }

    /// Retrieve the last message of a text channel.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rarity_cache_inmemory::InMemoryCache;
    /// use twilight_model::id::ChannelId;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let cache = InMemoryCache::new();
    ///
    /// if let Some(message) = cache.text_channels.last_message(ChannelId(1)).await? {
    ///     println!("the last message author's ID is {}", message.author_id);
    /// }
    /// # Ok(()) }
    /// ```
    pub fn last_message(
        &self,
        channel_id: ChannelId,
    ) -> GetEntityFuture<'_, MessageEntity, InMemoryBackendError> {
        TextChannelRepository::last_message(self, channel_id)
    }

    /// Retrieve the parent category channel of a text channel.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rarity_cache_inmemory::InMemoryCache;
    /// use twilight_model::id::ChannelId;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let cache = InMemoryCache::new();
    ///
    /// if let Some(channel) = cache.text_channels.parent(ChannelId(1)).await? {
    ///     println!("the parent category channel's name is {}", channel.name);
    /// }
    /// # Ok(()) }
    /// ```
    pub fn parent(
        &self,
        channel_id: ChannelId,
    ) -> GetEntityFuture<'_, CategoryChannelEntity, InMemoryBackendError> {
        TextChannelRepository::parent(self, channel_id)
    }
}

#[cfg(test)]
mod tests {
    use super::{TextChannelEntity, TextChannelRepository, Repository, InMemoryTextChannelRepository, InMemoryBackend};
    use static_assertions::{assert_impl_all, assert_obj_safe};
    use std::fmt::Debug;

    assert_impl_all!(
        InMemoryTextChannelRepository:
        TextChannelRepository<InMemoryBackend>,
        Clone,
        Debug,
        Repository<TextChannelEntity, InMemoryBackend>,
        Send,
        Sync,
    );
    assert_obj_safe!(InMemoryTextChannelRepository);
}
