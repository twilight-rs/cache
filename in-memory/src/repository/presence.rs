use crate::{config::EntityType, InMemoryBackendError, InMemoryBackendRef};
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
use std::sync::Arc;
use twilight_model::id::{GuildId, UserId};

/// Repository to retrieve and work with presences and their related entities.
#[derive(Clone, Debug)]
pub struct InMemoryPresenceRepository(pub(crate) Arc<InMemoryBackendRef>);

impl Repository<PresenceEntity, InMemoryBackendError> for InMemoryPresenceRepository {
    fn get(
        &self,
        presence_id: (GuildId, UserId),
    ) -> GetEntityFuture<'_, PresenceEntity, InMemoryBackendError> {
        future::ok(
            self.0
                .presences
                .get(&presence_id)
                .map(|r| r.value().clone()),
        )
        .boxed()
    }

    fn list(&self) -> ListEntitiesFuture<'_, PresenceEntity, InMemoryBackendError> {
        let stream = stream::iter(self.0.presences.iter().map(|r| Ok(r.value().clone()))).boxed();

        future::ok(stream).boxed()
    }

    fn remove(
        &self,
        presence_id: (GuildId, UserId),
    ) -> RemoveEntityFuture<'_, InMemoryBackendError> {
        if !self.0.config.entity_types().contains(EntityType::PRESENCE) {
            return future::ok(()).boxed();
        }

        self.0.presences.remove(&presence_id);

        future::ok(()).boxed()
    }

    fn upsert(&self, entity: PresenceEntity) -> UpsertEntityFuture<'_, InMemoryBackendError> {
        if !self.0.config.entity_types().contains(EntityType::PRESENCE) {
            return future::ok(()).boxed();
        }

        self.0.presences.insert(entity.id(), entity);

        future::ok(()).boxed()
    }
}

impl PresenceRepository<InMemoryBackendError> for InMemoryPresenceRepository {}
