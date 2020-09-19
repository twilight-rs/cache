use crate::{config::EntityType, InMemoryBackend, InMemoryBackendError};
use futures_util::{
    future::{self, FutureExt},
    stream::{self, StreamExt},
};
use rarity_cache::{
    entity::{
        guild::{EmojiEntity, EmojiRepository, GuildEntity, RoleEntity},
        user::UserEntity,
        Entity,
    },
    repository::{
        GetEntityFuture, ListEntitiesFuture, RemoveEntityFuture, Repository, UpsertEntityFuture,
    },
};
use twilight_model::id::EmojiId;

/// Repository to retrieve and work with emojis and their related entities.
#[derive(Clone, Debug)]
pub struct InMemoryEmojiRepository(pub(crate) InMemoryBackend);

impl Repository<EmojiEntity, InMemoryBackend> for InMemoryEmojiRepository {
    fn backend(&self) -> &InMemoryBackend {
        &self.0
    }

    fn get(&self, emoji_id: EmojiId) -> GetEntityFuture<'_, EmojiEntity, InMemoryBackendError> {
        future::ok((self.0).0.emojis.get(&emoji_id).map(|r| r.value().clone())).boxed()
    }

    fn list(&self) -> ListEntitiesFuture<'_, EmojiEntity, InMemoryBackendError> {
        let stream = stream::iter((self.0).0.emojis.iter().map(|r| Ok(r.value().clone()))).boxed();

        future::ok(stream).boxed()
    }

    fn remove(&self, emoji_id: EmojiId) -> RemoveEntityFuture<'_, InMemoryBackendError> {
        if !(self.0).0.config.entity_types().contains(EntityType::EMOJI) {
            return future::ok(()).boxed();
        }

        (self.0).0.emojis.remove(&emoji_id);

        future::ok(()).boxed()
    }

    fn upsert(&self, entity: EmojiEntity) -> UpsertEntityFuture<'_, InMemoryBackendError> {
        if !(self.0).0.config.entity_types().contains(EntityType::EMOJI) {
            return future::ok(()).boxed();
        }

        (self.0).0.emojis.insert(entity.id(), entity);

        future::ok(()).boxed()
    }
}

impl EmojiRepository<InMemoryBackend> for InMemoryEmojiRepository {
    fn guild(&self, emoji_id: EmojiId) -> GetEntityFuture<'_, GuildEntity, InMemoryBackendError> {
        let guild = self
            .0
             .0
            .emojis
            .get(&emoji_id)
            .map(|emoji| emoji.guild_id)
            .and_then(|id| (self.0).0.guilds.get(&id))
            .map(|r| r.value().clone());

        future::ok(guild).boxed()
    }

    fn roles(&self, emoji_id: EmojiId) -> ListEntitiesFuture<'_, RoleEntity, InMemoryBackendError> {
        let role_ids = match (self.0).0.emojis.get(&emoji_id) {
            Some(emoji) => emoji.role_ids.clone(),
            None => return future::ok(stream::empty().boxed()).boxed(),
        };

        let iter = role_ids
            .into_iter()
            .filter_map(move |id| (self.0).0.roles.get(&id).map(|r| Ok(r.value().clone())));
        let stream = stream::iter(iter).boxed();

        future::ok(stream).boxed()
    }

    fn user(&self, emoji_id: EmojiId) -> GetEntityFuture<'_, UserEntity, InMemoryBackendError> {
        let user = self
            .0
             .0
            .emojis
            .get(&emoji_id)
            .and_then(|emoji| emoji.user_id)
            .and_then(|id| (self.0).0.users.get(&id))
            .map(|r| r.value().clone());

        future::ok(user).boxed()
    }
}

impl InMemoryEmojiRepository {
    /// Retrieve the guild of an emoji.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rarity_cache_inmemory::InMemoryCache;
    /// use twilight_model::id::EmojiId;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let cache = InMemoryCache::new();
    ///
    /// if let Some(guild) = cache.emojis.guild(EmojiId(123456)).await? {
    ///     println!("the guild's name is {}", guild.name);
    /// }
    /// # Ok(()) }
    /// ```
    pub fn guild(
        &self,
        emoji_id: EmojiId,
    ) -> GetEntityFuture<'_, GuildEntity, InMemoryBackendError> {
        EmojiRepository::guild(self, emoji_id)
    }

    pub fn roles(
        &self,
        emoji_id: EmojiId,
    ) -> ListEntitiesFuture<'_, RoleEntity, InMemoryBackendError> {
        EmojiRepository::roles(self, emoji_id)
    }

    /// Retrieve the user who created an emoji.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rarity_cache_inmemory::InMemoryCache;
    /// use twilight_model::id::EmojiId;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let cache = InMemoryCache::new();
    ///
    /// if let Some(user) = cache.emojis.user(EmojiId(123456)).await? {
    ///     println!("the emoji creator's name is {}", user.name);
    /// }
    /// # Ok(()) }
    /// ```
    pub fn user(&self, emoji_id: EmojiId) -> GetEntityFuture<'_, UserEntity, InMemoryBackendError> {
        EmojiRepository::user(self, emoji_id)
    }
}

#[cfg(test)]
mod tests {
    use super::{EmojiEntity, EmojiRepository, Repository, InMemoryEmojiRepository, InMemoryBackend};
    use static_assertions::{assert_impl_all, assert_obj_safe};
    use std::fmt::Debug;

    assert_impl_all!(
        InMemoryEmojiRepository:
        EmojiRepository<InMemoryBackend>,
        Clone,
        Debug,
        Repository<EmojiEntity, InMemoryBackend>,
        Send,
        Sync,
    );
    assert_obj_safe!(InMemoryEmojiRepository);
}
