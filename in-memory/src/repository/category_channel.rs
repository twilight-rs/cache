use crate::{config::EntityType, InMemoryBackend, InMemoryBackendError};
use futures_util::{
    future::{self, FutureExt},
    stream::{self, StreamExt},
};
use rarity_cache::{
    entity::{
        channel::{CategoryChannelEntity, CategoryChannelRepository},
        guild::GuildEntity,
        Entity,
    },
    repository::{
        GetEntityFuture, ListEntitiesFuture, RemoveEntityFuture, Repository, UpsertEntityFuture,
    },
};
use twilight_model::id::ChannelId;

/// Repository to retrieve and work with category channels and their related entities.
#[derive(Clone, Debug)]
pub struct InMemoryCategoryChannelRepository(pub(crate) InMemoryBackend);

impl Repository<CategoryChannelEntity, InMemoryBackend> for InMemoryCategoryChannelRepository {
    fn backend(&self) -> InMemoryBackend {
        self.0.clone()
    }

    fn get(
        &self,
        channel_id: ChannelId,
    ) -> GetEntityFuture<'_, CategoryChannelEntity, InMemoryBackendError> {
        future::ok(
            self.0
                 .0
                .channels_category
                .get(&channel_id)
                .map(|r| r.value().clone()),
        )
        .boxed()
    }

    fn list(&self) -> ListEntitiesFuture<'_, CategoryChannelEntity, InMemoryBackendError> {
        let iter = stream::iter(
            self.0
                 .0
                .channels_category
                .iter()
                .map(|r| Ok(r.value().clone())),
        )
        .boxed();

        future::ok(iter).boxed()
    }

    fn remove(&self, channel_id: ChannelId) -> RemoveEntityFuture<'_, InMemoryBackendError> {
        (self.0).0.channels_category.remove(&channel_id);

        future::ok(()).boxed()
    }

    fn upsert(
        &self,
        category_channel: CategoryChannelEntity,
    ) -> UpsertEntityFuture<'_, InMemoryBackendError> {
        if !self
            .0
             .0
            .config
            .entity_types()
            .contains(EntityType::CHANNEL_CATEGORY)
        {
            return future::ok(()).boxed();
        }

        self.0
             .0
            .channels_category
            .insert(category_channel.id(), category_channel);

        future::ok(()).boxed()
    }
}

impl CategoryChannelRepository<InMemoryBackend> for InMemoryCategoryChannelRepository {
    fn guild(
        &self,
        channel_id: ChannelId,
    ) -> GetEntityFuture<'_, GuildEntity, InMemoryBackendError> {
        let guild = self
            .0
             .0
            .channels_category
            .get(&channel_id)
            .and_then(|channel| channel.guild_id)
            .and_then(|id| (self.0).0.guilds.get(&id))
            .map(|r| r.value().clone());

        future::ok(guild).boxed()
    }
}

impl InMemoryCategoryChannelRepository {
    /// Retrieve the guild of a category channel.
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
    /// if let Some(guild) = cache.category_channels.guild(ChannelId(123456)).await? {
    ///     println!("the guild's name is {}", guild.name);
    /// }
    /// # Ok(()) }
    /// ```
    pub fn guild(
        &self,
        channel_id: ChannelId,
    ) -> GetEntityFuture<'_, GuildEntity, InMemoryBackendError> {
        CategoryChannelRepository::guild(self, channel_id)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CategoryChannelEntity, CategoryChannelRepository, InMemoryBackend,
        InMemoryCategoryChannelRepository, Repository,
    };
    use static_assertions::{assert_impl_all, assert_obj_safe};
    use std::fmt::Debug;

    assert_impl_all!(
        InMemoryCategoryChannelRepository:
        CategoryChannelRepository<InMemoryBackend>,
        Clone,
        Debug,
        Repository<CategoryChannelEntity, InMemoryBackend>,
        Send,
        Sync,
    );
    assert_obj_safe!(InMemoryCategoryChannelRepository);
}
