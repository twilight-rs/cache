use crate::{config::EntityType, InMemoryBackend, InMemoryBackendError};
use futures_util::{
    future::{self, FutureExt},
    stream::{self, StreamExt},
};
use rarity_cache::{
    entity::{
        gateway::{PresenceEntity, PresenceRepository},
        Entity,
    },
    repository::{
        GetEntityFuture, ListEntitiesFuture, RemoveEntityFuture, Repository, UpsertEntityFuture,
    },
};
use twilight_model::id::{GuildId, UserId};

/// Repository to retrieve and work with presences and their related entities.
#[derive(Clone, Debug)]
pub struct InMemoryPresenceRepository(pub(crate) InMemoryBackend);

impl Repository<PresenceEntity, InMemoryBackend> for InMemoryPresenceRepository {
    fn backend(&self) -> &InMemoryBackend {
        &self.0
    }

    fn get(
        &self,
        presence_id: (GuildId, UserId),
    ) -> GetEntityFuture<'_, PresenceEntity, InMemoryBackendError> {
        future::ok(
            self.0
                 .0
                .presences
                .get(&presence_id)
                .map(|r| r.value().clone()),
        )
        .boxed()
    }

    fn list(&self) -> ListEntitiesFuture<'_, PresenceEntity, InMemoryBackendError> {
        let stream =
            stream::iter((self.0).0.presences.iter().map(|r| Ok(r.value().clone()))).boxed();

        future::ok(stream).boxed()
    }

    fn remove(
        &self,
        presence_id: (GuildId, UserId),
    ) -> RemoveEntityFuture<'_, InMemoryBackendError> {
        (self.0).0.presences.remove(&presence_id);

        future::ok(()).boxed()
    }

    fn upsert(&self, entity: PresenceEntity) -> UpsertEntityFuture<'_, InMemoryBackendError> {
        if !(self.0)
            .0
            .config
            .entity_types()
            .contains(EntityType::PRESENCE)
        {
            return future::ok(()).boxed();
        }

        (self.0).0.presences.insert(entity.id(), entity);

        future::ok(()).boxed()
    }
}

impl PresenceRepository<InMemoryBackend> for InMemoryPresenceRepository {}

#[cfg(test)]
mod tests {
    use super::{
        InMemoryBackend, InMemoryPresenceRepository, PresenceEntity, PresenceRepository, Repository,
    };
    use static_assertions::{assert_impl_all, assert_obj_safe};
    use std::fmt::Debug;

    assert_impl_all!(
        InMemoryPresenceRepository:
        PresenceRepository<InMemoryBackend>,
        Clone,
        Debug,
        Repository<PresenceEntity, InMemoryBackend>,
        Send,
        Sync,
    );
    assert_obj_safe!(InMemoryPresenceRepository);
}
