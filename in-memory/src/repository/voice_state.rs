use crate::{config::EntityType, InMemoryBackendError, InMemoryBackendRef};
use futures_util::{
    future::{self, FutureExt},
    stream::{self, StreamExt},
};
use rarity_cache::{
    entity::{
        channel::VoiceChannelEntity,
        voice::{VoiceStateEntity, VoiceStateRepository},
        Entity,
    },
    repository::{
        GetEntityFuture, ListEntitiesFuture, RemoveEntityFuture, Repository, UpsertEntityFuture,
    },
};
use std::sync::Arc;
use twilight_model::id::{GuildId, UserId};

/// Repository to retrieve and work with voice states and their related
/// entities.
#[derive(Clone, Debug)]
pub struct InMemoryVoiceStateRepository(pub(crate) Arc<InMemoryBackendRef>);

impl Repository<VoiceStateEntity, InMemoryBackendError> for InMemoryVoiceStateRepository {
    fn get(
        &self,
        voice_state_id: (GuildId, UserId),
    ) -> GetEntityFuture<'_, VoiceStateEntity, InMemoryBackendError> {
        future::ok(
            self.0
                .voice_states
                .get(&voice_state_id)
                .map(|r| r.value().clone()),
        )
        .boxed()
    }

    fn list(&self) -> ListEntitiesFuture<'_, VoiceStateEntity, InMemoryBackendError> {
        let stream =
            stream::iter(self.0.voice_states.iter().map(|r| Ok(r.value().clone()))).boxed();

        future::ok(stream).boxed()
    }

    fn remove(
        &self,
        voice_state_id: (GuildId, UserId),
    ) -> RemoveEntityFuture<'_, InMemoryBackendError> {
        if !self
            .0
            .config
            .entity_types()
            .contains(EntityType::VOICE_STATE)
        {
            return future::ok(()).boxed();
        }

        self.0.voice_states.remove(&voice_state_id);

        future::ok(()).boxed()
    }

    fn upsert(&self, entity: VoiceStateEntity) -> UpsertEntityFuture<'_, InMemoryBackendError> {
        if !self
            .0
            .config
            .entity_types()
            .contains(EntityType::VOICE_STATE)
        {
            return future::ok(()).boxed();
        }

        self.0.voice_states.insert(entity.id(), entity);

        future::ok(()).boxed()
    }
}

impl VoiceStateRepository<InMemoryBackendError> for InMemoryVoiceStateRepository {
    fn channel(
        &self,
        guild_id: GuildId,
        user_id: UserId,
    ) -> GetEntityFuture<'_, VoiceChannelEntity, InMemoryBackendError> {
        let channel = self
            .0
            .voice_states
            .get(&(guild_id, user_id))
            .and_then(|state| state.channel_id)
            .and_then(|id| self.0.channels_voice.get(&id))
            .map(|r| r.value().clone());

        future::ok(channel).boxed()
    }
}

impl InMemoryVoiceStateRepository {
    pub fn channel(
        &self,
        guild_id: GuildId,
        user_id: UserId,
    ) -> GetEntityFuture<'_, VoiceChannelEntity, InMemoryBackendError> {
        VoiceStateRepository::channel(self, guild_id, user_id)
    }
}
