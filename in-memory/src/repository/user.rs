use crate::{config::EntityType, InMemoryBackend, InMemoryBackendError};
use futures_util::{
    future::{self, FutureExt},
    stream::{self, StreamExt},
};
use rarity_cache::{
    entity::{
        guild::GuildEntity,
        user::{UserEntity, UserRepository},
        Entity,
    },
    repository::{
        GetEntityFuture, ListEntitiesFuture, ListEntityIdsFuture, RemoveEntitiesFuture,
        RemoveEntityFuture, Repository,
    },
};
use twilight_model::id::{GuildId, UserId};

/// Repository to retrieve and work with users and their related entities.
#[derive(Clone, Debug)]
pub struct InMemoryUserRepository(pub(crate) InMemoryBackend);

impl Repository<UserEntity, InMemoryBackend> for InMemoryUserRepository {
    fn backend(&self) -> InMemoryBackend {
        self.0.clone()
    }

    fn get(&self, user_id: UserId) -> GetEntityFuture<'_, UserEntity, InMemoryBackendError> {
        future::ok((self.0).0.users.get(&user_id).map(|r| r.value().clone())).boxed()
    }

    fn list(&self) -> ListEntitiesFuture<'_, UserEntity, InMemoryBackendError> {
        let stream = stream::iter((self.0).0.users.iter().map(|r| Ok(r.value().clone()))).boxed();

        future::ok(stream).boxed()
    }

    fn remove(&self, user_id: UserId) -> RemoveEntityFuture<'_, InMemoryBackendError> {
        (self.0).0.users.remove(&user_id);

        future::ok(()).boxed()
    }

    fn upsert(&self, entity: UserEntity) -> RemoveEntitiesFuture<'_, InMemoryBackendError> {
        if !(self.0).0.config.entity_types().contains(EntityType::USER) {
            return future::ok(()).boxed();
        }

        (self.0).0.users.insert(entity.id(), entity);

        future::ok(()).boxed()
    }
}

impl UserRepository<InMemoryBackend> for InMemoryUserRepository {
    fn guild_ids(&self, user_id: UserId) -> ListEntityIdsFuture<'_, GuildId, InMemoryBackendError> {
        let stream = (self.0).0.user_guilds.get(&user_id).map_or_else(
            || stream::empty().boxed(),
            |r| stream::iter(r.value().iter().map(|x| Ok(*x)).collect::<Vec<_>>()).boxed(),
        );

        future::ok(stream).boxed()
    }

    fn guilds(&self, user_id: UserId) -> ListEntitiesFuture<'_, GuildEntity, InMemoryBackendError> {
        let guild_ids = match (self.0).0.user_guilds.get(&user_id) {
            Some(user_guilds) => user_guilds.clone(),
            None => return future::ok(stream::empty().boxed()).boxed(),
        };

        let iter = guild_ids
            .into_iter()
            .filter_map(move |id| (self.0).0.guilds.get(&id).map(|r| Ok(r.value().clone())));
        let stream = stream::iter(iter).boxed();

        future::ok(stream).boxed()
    }
}

impl InMemoryUserRepository {
    pub fn guild_ids(
        &self,
        user_id: UserId,
    ) -> ListEntityIdsFuture<'_, GuildId, InMemoryBackendError> {
        UserRepository::guild_ids(self, user_id)
    }

    pub fn guilds(
        &self,
        user_id: UserId,
    ) -> ListEntitiesFuture<'_, GuildEntity, InMemoryBackendError> {
        UserRepository::guilds(self, user_id)
    }
}

#[cfg(test)]
mod tests {
    use super::{InMemoryBackend, InMemoryUserRepository, Repository, UserEntity, UserRepository};
    use static_assertions::{assert_impl_all, assert_obj_safe};
    use std::fmt::Debug;

    assert_impl_all!(
        InMemoryUserRepository:
        UserRepository<InMemoryBackend>,
        Clone,
        Debug,
        Repository<UserEntity, InMemoryBackend>,
        Send,
        Sync,
    );
    assert_obj_safe!(InMemoryUserRepository);
}
