use crate::{config::EntityType, InMemoryBackendError, InMemoryBackendRef};
use futures_util::{
    future::{self, FutureExt},
    stream::{self, StreamExt},
};
use rarity_cache::{
    entity::{
        guild::{GuildEntity, RoleEntity, RoleRepository},
        Entity,
    },
    repository::{
        GetEntityFuture, ListEntitiesFuture, RemoveEntityFuture, Repository, UpsertEntityFuture,
    },
};
use std::sync::Arc;
use twilight_model::id::RoleId;

/// Repository to retrieve and work with roles and their related entities.
#[derive(Clone, Debug)]
pub struct InMemoryRoleRepository(pub(crate) Arc<InMemoryBackendRef>);

impl Repository<RoleEntity, InMemoryBackendError> for InMemoryRoleRepository {
    fn get(&self, role_id: RoleId) -> GetEntityFuture<'_, RoleEntity, InMemoryBackendError> {
        future::ok(self.0.roles.get(&role_id).map(|r| r.value().clone())).boxed()
    }

    fn list(&self) -> ListEntitiesFuture<'_, RoleEntity, InMemoryBackendError> {
        let stream = stream::iter(self.0.roles.iter().map(|r| Ok(r.value().clone()))).boxed();

        future::ok(stream).boxed()
    }

    fn remove(&self, role_id: RoleId) -> RemoveEntityFuture<'_, InMemoryBackendError> {
        if !self.0.config.entity_types().contains(EntityType::ROLE) {
            return future::ok(()).boxed();
        }

        self.0.roles.remove(&role_id);

        future::ok(()).boxed()
    }

    fn upsert(&self, entity: RoleEntity) -> UpsertEntityFuture<'_, InMemoryBackendError> {
        if !self.0.config.entity_types().contains(EntityType::ROLE) {
            return future::ok(()).boxed();
        }

        self.0.roles.insert(entity.id(), entity);

        future::ok(()).boxed()
    }
}

impl RoleRepository<InMemoryBackendError> for InMemoryRoleRepository {
    fn guild(&self, role_id: RoleId) -> GetEntityFuture<'_, GuildEntity, InMemoryBackendError> {
        let guild = self
            .0
            .roles
            .get(&role_id)
            .map(|role| role.guild_id)
            .and_then(|id| self.0.guilds.get(&id))
            .map(|r| r.value().clone());

        future::ok(guild).boxed()
    }
}

impl InMemoryRoleRepository {
    pub fn guild(&self, role_id: RoleId) -> GetEntityFuture<'_, GuildEntity, InMemoryBackendError> {
        RoleRepository::guild(self, role_id)
    }
}
