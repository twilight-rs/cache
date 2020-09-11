use crate::{config::EntityType, InMemoryBackendError, InMemoryBackendRef};
use futures_util::{
    future::{self, FutureExt},
    stream::{self, StreamExt},
};
use rarity_cache::{
    entity::{
        channel::{GroupEntity, GroupRepository, MessageEntity},
        user::UserEntity,
        Entity,
    },
    repository::{
        GetEntityFuture, ListEntitiesFuture, RemoveEntityFuture, Repository, UpsertEntityFuture,
    },
};
use std::sync::Arc;
use twilight_model::id::ChannelId;

/// Repository to retrieve and work with groups and their related entities.
#[derive(Clone, Debug)]
pub struct InMemoryGroupRepository(pub(crate) Arc<InMemoryBackendRef>);

impl Repository<GroupEntity, InMemoryBackendError> for InMemoryGroupRepository {
    fn get(&self, group_id: ChannelId) -> GetEntityFuture<'_, GroupEntity, InMemoryBackendError> {
        future::ok(self.0.groups.get(&group_id).map(|r| r.value().clone())).boxed()
    }

    fn list(&self) -> ListEntitiesFuture<'_, GroupEntity, InMemoryBackendError> {
        let stream = stream::iter(self.0.groups.iter().map(|r| Ok(r.value().clone()))).boxed();

        future::ok(stream).boxed()
    }

    fn remove(&self, group_id: ChannelId) -> RemoveEntityFuture<'_, InMemoryBackendError> {
        if !self
            .0
            .config
            .entity_types()
            .contains(EntityType::CHANNEL_GROUP)
        {
            return future::ok(()).boxed();
        }

        self.0.groups.remove(&group_id);

        future::ok(()).boxed()
    }

    fn upsert(&self, entity: GroupEntity) -> UpsertEntityFuture<'_, InMemoryBackendError> {
        if !self
            .0
            .config
            .entity_types()
            .contains(EntityType::CHANNEL_GROUP)
        {
            return future::ok(()).boxed();
        }

        self.0.groups.insert(entity.id(), entity);

        future::ok(()).boxed()
    }
}

impl GroupRepository<InMemoryBackendError> for InMemoryGroupRepository {
    fn last_message(
        &self,
        group_id: ChannelId,
    ) -> GetEntityFuture<'_, MessageEntity, InMemoryBackendError> {
        let message = self
            .0
            .groups
            .get(&group_id)
            .and_then(|group| group.last_message_id)
            .and_then(|id| self.0.messages.get(&id))
            .map(|r| r.value().clone());

        future::ok(message).boxed()
    }

    fn owner(&self, group_id: ChannelId) -> GetEntityFuture<'_, UserEntity, InMemoryBackendError> {
        let guild = self
            .0
            .groups
            .get(&group_id)
            .map(|message| message.owner_id)
            .and_then(|id| self.0.users.get(&id))
            .map(|r| r.value().clone());

        future::ok(guild).boxed()
    }

    fn recipients(
        &self,
        group_id: ChannelId,
    ) -> ListEntitiesFuture<'_, UserEntity, InMemoryBackendError> {
        let recipient_ids = match self.0.groups.get(&group_id) {
            Some(group) => group.recipient_ids.clone(),
            None => return future::ok(stream::empty().boxed()).boxed(),
        };

        let iter = recipient_ids
            .into_iter()
            .filter_map(move |id| self.0.users.get(&id).map(|r| Ok(r.value().clone())));
        let stream = stream::iter(iter).boxed();

        future::ok(stream).boxed()
    }
}

impl InMemoryGroupRepository {
    /// Retrieve the last message of a group.
    pub fn last_message(
        &self,
        group_id: ChannelId,
    ) -> GetEntityFuture<'_, MessageEntity, InMemoryBackendError> {
        GroupRepository::last_message(self, group_id)
    }

    /// Retrieve the owner of a group.
    pub fn owner(
        &self,
        group_id: ChannelId,
    ) -> GetEntityFuture<'_, UserEntity, InMemoryBackendError> {
        GroupRepository::owner(self, group_id)
    }

    pub fn recipients(
        &self,
        group_id: ChannelId,
    ) -> ListEntitiesFuture<'_, UserEntity, InMemoryBackendError> {
        GroupRepository::recipients(self, group_id)
    }
}
