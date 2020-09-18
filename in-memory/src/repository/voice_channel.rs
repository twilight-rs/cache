use crate::{config::EntityType, InMemoryBackend, InMemoryBackendError};
use futures_util::{
    future::{self, FutureExt},
    stream::{self, StreamExt},
};
use rarity_cache::{
    entity::{
        channel::{CategoryChannelEntity, VoiceChannelEntity, VoiceChannelRepository},
        guild::GuildEntity,
        Entity,
    },
    repository::{
        GetEntityFuture, ListEntitiesFuture, RemoveEntityFuture, Repository, UpsertEntityFuture,
    },
};
use twilight_model::id::ChannelId;

/// Repository to retrieve and work with voice channels and their related
/// entities.
#[derive(Clone, Debug)]
pub struct InMemoryVoiceChannelRepository(pub(crate) InMemoryBackend);

impl Repository<VoiceChannelEntity, InMemoryBackend> for InMemoryVoiceChannelRepository {
    fn backend(&self) -> &InMemoryBackend {
        &self.0
    }

    fn get(
        &self,
        user_id: ChannelId,
    ) -> GetEntityFuture<'_, VoiceChannelEntity, InMemoryBackendError> {
        future::ok(
            self.0
                 .0
                .channels_voice
                .get(&user_id)
                .map(|r| r.value().clone()),
        )
        .boxed()
    }

    fn list(&self) -> ListEntitiesFuture<'_, VoiceChannelEntity, InMemoryBackendError> {
        let stream = stream::iter(
            (self.0)
                .0
                .channels_voice
                .iter()
                .map(|r| Ok(r.value().clone())),
        )
        .boxed();

        future::ok(stream).boxed()
    }

    fn remove(&self, user_id: ChannelId) -> RemoveEntityFuture<'_, InMemoryBackendError> {
        if !self
            .0
             .0
            .config
            .entity_types()
            .contains(EntityType::CHANNEL_VOICE)
        {
            return future::ok(()).boxed();
        }

        (self.0).0.channels_voice.remove(&user_id);

        future::ok(()).boxed()
    }

    fn upsert(&self, entity: VoiceChannelEntity) -> UpsertEntityFuture<'_, InMemoryBackendError> {
        if !self
            .0
             .0
            .config
            .entity_types()
            .contains(EntityType::CHANNEL_VOICE)
        {
            return future::ok(()).boxed();
        }

        (self.0).0.channels_voice.insert(entity.id(), entity);

        future::ok(()).boxed()
    }
}

impl VoiceChannelRepository<InMemoryBackend> for InMemoryVoiceChannelRepository {
    fn guild(
        &self,
        channel_id: ChannelId,
    ) -> GetEntityFuture<'_, GuildEntity, InMemoryBackendError> {
        let guild = self
            .0
             .0
            .channels_voice
            .get(&channel_id)
            .and_then(|channel| channel.guild_id)
            .and_then(|id| (self.0).0.guilds.get(&id))
            .map(|r| r.value().clone());

        future::ok(guild).boxed()
    }

    fn parent(
        &self,
        channel_id: ChannelId,
    ) -> GetEntityFuture<'_, CategoryChannelEntity, InMemoryBackendError> {
        let parent = self
            .0
             .0
            .channels_voice
            .get(&channel_id)
            .and_then(|channel| channel.parent_id)
            .and_then(|id| (self.0).0.channels_category.get(&id))
            .map(|r| r.value().clone());

        future::ok(parent).boxed()
    }
}

impl InMemoryVoiceChannelRepository {
    /// Retrieve the guild of a voice channel.
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
    /// if let Some(guild) = cache.voice_channels.guild(ChannelId(1)).await? {
    ///     println!("the guild's name is {}", guild.name);
    /// }
    /// # Ok(()) }
    /// ```
    pub fn guild(
        &self,
        channel_id: ChannelId,
    ) -> GetEntityFuture<'_, GuildEntity, InMemoryBackendError> {
        VoiceChannelRepository::guild(self, channel_id)
    }

    /// Retrieve the parent category channel of a voice channel.
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
    /// if let Some(channel) = cache.voice_channels.parent(ChannelId(1)).await? {
    ///     println!("the parent category channel's name is {}", channel.name);
    /// }
    /// # Ok(()) }
    /// ```
    pub fn parent(
        &self,
        channel_id: ChannelId,
    ) -> GetEntityFuture<'_, CategoryChannelEntity, InMemoryBackendError> {
        VoiceChannelRepository::parent(self, channel_id)
    }
}
