use crate::{config::EntityType, InMemoryBackend, InMemoryBackendError};
use futures_util::{
    future::{self, FutureExt},
    stream::{self, StreamExt},
};
use rarity_cache::{
    entity::{
        channel::{GuildChannelEntity, TextChannelEntity, VoiceChannelEntity},
        gateway::PresenceEntity,
        guild::{EmojiEntity, GuildEntity, GuildRepository, MemberEntity, RoleEntity},
        user::UserEntity,
        voice::VoiceStateEntity,
        Entity,
    },
    repository::{
        GetEntityFuture, ListEntitiesFuture, ListEntityIdsFuture, RemoveEntityFuture, Repository,
        UpsertEntityFuture,
    },
};
use twilight_model::id::{ChannelId, EmojiId, GuildId, RoleId, UserId};

/// Repository to retrieve and work with guilds and their related entities.
#[derive(Clone, Debug)]
pub struct InMemoryGuildRepository(pub(crate) InMemoryBackend);

impl Repository<GuildEntity, InMemoryBackend> for InMemoryGuildRepository {
    fn backend(&self) -> &InMemoryBackend {
        &self.0
    }

    fn get(&self, guild_id: GuildId) -> GetEntityFuture<'_, GuildEntity, InMemoryBackendError> {
        future::ok((self.0).0.guilds.get(&guild_id).map(|r| r.value().clone())).boxed()
    }

    fn list(&self) -> ListEntitiesFuture<'_, GuildEntity, InMemoryBackendError> {
        let stream = stream::iter((self.0).0.guilds.iter().map(|r| Ok(r.value().clone()))).boxed();

        future::ok(stream).boxed()
    }

    fn remove(&self, guild_id: GuildId) -> RemoveEntityFuture<'_, InMemoryBackendError> {
        if !(self.0).0.config.entity_types().contains(EntityType::GUILD) {
            return future::ok(()).boxed();
        }

        (self.0).0.guilds.remove(&guild_id);

        future::ok(()).boxed()
    }

    fn upsert(&self, entity: GuildEntity) -> UpsertEntityFuture<'_, InMemoryBackendError> {
        if !(self.0).0.config.entity_types().contains(EntityType::GUILD) {
            return future::ok(()).boxed();
        }

        (self.0).0.guilds.insert(entity.id(), entity);

        future::ok(()).boxed()
    }
}

impl InMemoryGuildRepository {
    pub fn members(
        &self,
        guild_id: GuildId,
    ) -> ListEntitiesFuture<'_, MemberEntity, InMemoryBackendError> {
        GuildRepository::members(self, guild_id)
    }
}

impl GuildRepository<InMemoryBackend> for InMemoryGuildRepository {
    fn afk_channel(
        &self,
        guild_id: GuildId,
    ) -> GetEntityFuture<'_, VoiceChannelEntity, InMemoryBackendError> {
        let guild = self
            .0
             .0
            .guilds
            .get(&guild_id)
            .and_then(|guild| guild.afk_channel_id)
            .and_then(|id| (self.0).0.channels_voice.get(&id))
            .map(|r| r.value().clone());

        future::ok(guild).boxed()
    }

    fn channel_ids(
        &self,
        guild_id: GuildId,
    ) -> ListEntityIdsFuture<'_, ChannelId, InMemoryBackendError> {
        let stream = (self.0).0.guild_channels.get(&guild_id).map_or_else(
            || stream::empty().boxed(),
            |set| stream::iter(set.iter().map(|x| Ok(*x)).collect::<Vec<_>>()).boxed(),
        );

        future::ok(stream).boxed()
    }

    fn channels(
        &self,
        guild_id: GuildId,
    ) -> ListEntitiesFuture<'_, GuildChannelEntity, InMemoryBackendError> {
        let channel_ids = match (self.0).0.guild_channels.get(&guild_id) {
            Some(guild_channels) => guild_channels.clone(),
            None => return future::ok(stream::empty().boxed()).boxed(),
        };

        let iter = channel_ids.into_iter().filter_map(move |id| {
            if let Some(r) = (self.0).0.channels_text.get(&id) {
                return Some(Ok(GuildChannelEntity::Text(r.value().clone())));
            }

            if let Some(r) = (self.0).0.channels_voice.get(&id) {
                return Some(Ok(GuildChannelEntity::Voice(r.value().clone())));
            }

            if let Some(r) = (self.0).0.channels_category.get(&id) {
                return Some(Ok(GuildChannelEntity::Category(r.value().clone())));
            }

            None
        });
        let stream = stream::iter(iter).boxed();

        future::ok(stream).boxed()
    }

    fn emoji_ids(
        &self,
        guild_id: GuildId,
    ) -> ListEntityIdsFuture<'_, EmojiId, InMemoryBackendError> {
        let stream = (self.0).0.guild_emojis.get(&guild_id).map_or_else(
            || stream::empty().boxed(),
            |set| stream::iter(set.iter().map(|x| Ok(*x)).collect::<Vec<_>>()).boxed(),
        );

        future::ok(stream).boxed()
    }

    fn emojis(
        &self,
        guild_id: GuildId,
    ) -> ListEntitiesFuture<'_, EmojiEntity, InMemoryBackendError> {
        let emoji_ids = match (self.0).0.guild_emojis.get(&guild_id) {
            Some(guild_emojis) => guild_emojis.clone(),
            None => return future::ok(stream::empty().boxed()).boxed(),
        };

        let iter = emoji_ids
            .into_iter()
            .filter_map(move |id| (self.0).0.emojis.get(&id).map(|r| Ok(r.value().clone())));
        let stream = stream::iter(iter).boxed();

        future::ok(stream).boxed()
    }

    fn member_ids(
        &self,
        guild_id: GuildId,
    ) -> ListEntityIdsFuture<'_, UserId, InMemoryBackendError> {
        let stream = (self.0).0.guild_members.get(&guild_id).map_or_else(
            || stream::empty().boxed(),
            |set| stream::iter(set.iter().map(|x| Ok(*x)).collect::<Vec<_>>()).boxed(),
        );

        future::ok(stream).boxed()
    }

    fn members(
        &self,
        guild_id: GuildId,
    ) -> ListEntitiesFuture<'_, MemberEntity, InMemoryBackendError> {
        let user_ids = match (self.0).0.guild_members.get(&guild_id) {
            Some(guild_members) => guild_members.clone(),
            None => return future::ok(stream::empty().boxed()).boxed(),
        };

        let iter = user_ids.into_iter().filter_map(move |id| {
            self.0
                 .0
                .members
                .get(&(guild_id, id))
                .map(|r| Ok(r.value().clone()))
        });
        let stream = stream::iter(iter).boxed();

        future::ok(stream).boxed()
    }

    fn owner(&self, guild_id: GuildId) -> GetEntityFuture<'_, UserEntity, InMemoryBackendError> {
        let guild = self
            .0
             .0
            .guilds
            .get(&guild_id)
            .map(|guild| guild.owner_id)
            .and_then(|id| (self.0).0.users.get(&id))
            .map(|r| r.value().clone());

        future::ok(guild).boxed()
    }

    fn presence_ids(
        &self,
        guild_id: GuildId,
    ) -> ListEntityIdsFuture<'_, UserId, InMemoryBackendError> {
        let stream = (self.0).0.guild_presences.get(&guild_id).map_or_else(
            || stream::empty().boxed(),
            |set| stream::iter(set.iter().map(|x| Ok(*x)).collect::<Vec<_>>()).boxed(),
        );

        future::ok(stream).boxed()
    }

    fn presences(
        &self,
        guild_id: GuildId,
    ) -> ListEntitiesFuture<'_, PresenceEntity, InMemoryBackendError> {
        let user_ids = match (self.0).0.guild_presences.get(&guild_id) {
            Some(guild_presences) => guild_presences.clone(),
            None => return future::ok(stream::empty().boxed()).boxed(),
        };

        let iter = user_ids.into_iter().filter_map(move |id| {
            self.0
                 .0
                .presences
                .get(&(guild_id, id))
                .map(|r| Ok(r.value().clone()))
        });
        let stream = stream::iter(iter).boxed();

        future::ok(stream).boxed()
    }

    fn role_ids(&self, guild_id: GuildId) -> ListEntityIdsFuture<'_, RoleId, InMemoryBackendError> {
        let stream = (self.0).0.guild_roles.get(&guild_id).map_or_else(
            || stream::empty().boxed(),
            |set| stream::iter(set.iter().map(|x| Ok(*x)).collect::<Vec<_>>()).boxed(),
        );

        future::ok(stream).boxed()
    }

    fn roles(&self, guild_id: GuildId) -> ListEntitiesFuture<'_, RoleEntity, InMemoryBackendError> {
        let role_ids = match (self.0).0.guild_roles.get(&guild_id) {
            Some(guild_roles) => guild_roles.clone(),
            None => return future::ok(stream::empty().boxed()).boxed(),
        };

        let iter = role_ids
            .into_iter()
            .filter_map(move |id| (self.0).0.roles.get(&id).map(|r| Ok(r.value().clone())));
        let stream = stream::iter(iter).boxed();

        future::ok(stream).boxed()
    }

    fn rules_channel(
        &self,
        guild_id: GuildId,
    ) -> GetEntityFuture<'_, TextChannelEntity, InMemoryBackendError> {
        let guild = self
            .0
             .0
            .guilds
            .get(&guild_id)
            .and_then(|guild| guild.rules_channel_id)
            .and_then(|id| (self.0).0.channels_text.get(&id))
            .map(|r| r.value().clone());

        future::ok(guild).boxed()
    }

    fn system_channel(
        &self,
        guild_id: GuildId,
    ) -> GetEntityFuture<'_, TextChannelEntity, InMemoryBackendError> {
        let guild = self
            .0
             .0
            .guilds
            .get(&guild_id)
            .and_then(|guild| guild.system_channel_id)
            .and_then(|id| (self.0).0.channels_text.get(&id))
            .map(|r| r.value().clone());

        future::ok(guild).boxed()
    }

    fn voice_state_ids(
        &self,
        guild_id: GuildId,
    ) -> ListEntityIdsFuture<'_, UserId, InMemoryBackendError> {
        let stream = (self.0).0.guild_voice_states.get(&guild_id).map_or_else(
            || stream::empty().boxed(),
            |set| stream::iter(set.iter().map(|x| Ok(*x)).collect::<Vec<_>>()).boxed(),
        );

        future::ok(stream).boxed()
    }

    fn voice_states(
        &self,
        guild_id: GuildId,
    ) -> ListEntitiesFuture<'_, VoiceStateEntity, InMemoryBackendError> {
        let user_ids = match (self.0).0.guild_voice_states.get(&guild_id) {
            Some(guild_voice_states) => guild_voice_states.clone(),
            None => return future::ok(stream::empty().boxed()).boxed(),
        };

        let iter = user_ids.into_iter().filter_map(move |id| {
            self.0
                 .0
                .voice_states
                .get(&(guild_id, id))
                .map(|r| Ok(r.value().clone()))
        });
        let stream = stream::iter(iter).boxed();

        future::ok(stream).boxed()
    }

    fn widget_channel(
        &self,
        guild_id: GuildId,
    ) -> GetEntityFuture<'_, GuildChannelEntity, InMemoryBackendError> {
        let id = match (self.0).0.guilds.get(&guild_id) {
            Some(guild) if guild.widget_channel_id.is_some() => guild.widget_channel_id.unwrap(),
            _ => return future::ok(None).boxed(),
        };

        if let Some(r) = (self.0).0.channels_text.get(&id) {
            let entity = GuildChannelEntity::Text(r.value().clone());

            return future::ok(Some(entity)).boxed();
        }

        if let Some(r) = (self.0).0.channels_voice.get(&id) {
            let entity = GuildChannelEntity::Voice(r.value().clone());

            return future::ok(Some(entity)).boxed();
        }

        if let Some(r) = (self.0).0.channels_category.get(&id) {
            let entity = GuildChannelEntity::Category(r.value().clone());

            return future::ok(Some(entity)).boxed();
        }

        future::ok(None).boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::{GuildEntity, GuildRepository, Repository, InMemoryGuildRepository, InMemoryBackend};
    use static_assertions::{assert_impl_all, assert_obj_safe};
    use std::fmt::Debug;

    assert_impl_all!(
        InMemoryGuildRepository:
        GuildRepository<InMemoryBackend>,
        Clone,
        Debug,
        Repository<GuildEntity, InMemoryBackend>,
        Send,
        Sync,
    );
    assert_obj_safe!(InMemoryGuildRepository);
}
