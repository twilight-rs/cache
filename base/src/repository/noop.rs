use super::{
    super::{
        backend::Backend,
        entity::{
            channel::{
                attachment::{AttachmentEntity, AttachmentRepository},
                category_channel::{CategoryChannelEntity, CategoryChannelRepository},
                group::GroupRepository,
                message::{MessageEntity, MessageRepository},
                private_channel::PrivateChannelRepository,
                text_channel::{TextChannelEntity, TextChannelRepository},
                voice_channel::{VoiceChannelEntity, VoiceChannelRepository},
                ChannelEntity,
            },
            gateway::presence::{PresenceEntity, PresenceRepository},
            guild::{
                emoji::{EmojiEntity, EmojiRepository},
                member::{MemberEntity, MemberRepository},
                role::{RoleEntity, RoleRepository},
                GuildEntity, GuildRepository,
            },
            user::{UserEntity, UserRepository},
            voice::{VoiceStateEntity, VoiceStateRepository},
            Entity,
        },
    },
    GetEntityFuture, ListEntitiesFuture, RemoveEntitiesFuture, RemoveEntityFuture, Repository,
    UpsertEntitiesFuture, UpsertEntityFuture,
};
use futures_util::{
    future::{self, FutureExt},
    stream::{self, StreamExt},
};
use twilight_model::id::{AttachmentId, ChannelId, EmojiId, GuildId, MessageId, RoleId, UserId};

/// Repository that implements no operations: when called it will do nothing.
///
/// This is a default repository that may be used by backends when a specific
/// entity, such as presences, are not handled by the cache.
///
/// Retrieval methods such as [`get`] will return no entity, and mutating
/// methods such as [`upsert`] will do nothing with the given entity.
/// Entity-specific repositories also function this way.
///
/// The [`backend`] repository method will still continue to return the backend.
///
/// [`backend`]: #method.backend
/// [`get`]: #method.get
/// [`upsert`]: #method.upsert
#[derive(Clone, Debug)]
pub struct NoopRepository<B>(B);

impl<B: Backend> NoopRepository<B> {
    /// Create a new noop repository with the backend.
    pub fn new(backend: B) -> Self {
        Self(backend)
    }
}

impl<B: Backend, E: Entity + 'static> Repository<E, B> for NoopRepository<B> {
    /// Returns an immutable reference to the backend.
    fn backend(&self) -> &B {
        &self.0
    }

    /// Always returns no entity.
    fn get(&self, _: E::Id) -> GetEntityFuture<'_, E, B::Error> {
        future::ok(None).boxed()
    }

    /// Always returns an empty stream with no entities.
    fn list(&self) -> ListEntitiesFuture<'_, E, B::Error> {
        future::ok(stream::empty().boxed()).boxed()
    }

    /// Always does nothing.
    fn remove(&self, _: E::Id) -> RemoveEntityFuture<'_, B::Error> {
        future::ok(()).boxed()
    }

    /// Always does nothing with the provided entity.
    fn upsert(&self, _: E) -> UpsertEntityFuture<'_, B::Error> {
        future::ok(()).boxed()
    }

    /// Always does nothing.
    fn remove_bulk<T: Iterator<Item = E::Id>>(&self, _: T) -> RemoveEntitiesFuture<'_, B::Error> {
        future::ok(()).boxed()
    }

    /// Always does nothing with the provided entity.
    fn upsert_bulk<T: Iterator<Item = E> + Send>(
        &self,
        _: T,
    ) -> UpsertEntitiesFuture<'_, B::Error> {
        future::ok(()).boxed()
    }
}

impl<B: Backend + Send> AttachmentRepository<B> for NoopRepository<B> {
    fn message(&self, _: AttachmentId) -> GetEntityFuture<'_, MessageEntity, B::Error> {
        future::ok(None).boxed()
    }
}

impl<B: Backend + Send> CategoryChannelRepository<B> for NoopRepository<B> {
    fn guild(&self, _: ChannelId) -> GetEntityFuture<'_, GuildEntity, B::Error> {
        future::ok(None).boxed()
    }
}

impl<B: Backend + Send> EmojiRepository<B> for NoopRepository<B> {
    fn guild(&self, _: EmojiId) -> GetEntityFuture<'_, GuildEntity, B::Error> {
        future::ok(None).boxed()
    }

    fn roles(&self, _: EmojiId) -> ListEntitiesFuture<'_, RoleEntity, B::Error> {
        future::ok(stream::empty().boxed()).boxed()
    }

    fn user(&self, _: EmojiId) -> GetEntityFuture<'_, UserEntity, B::Error> {
        future::ok(None).boxed()
    }
}

impl<B: Backend + Send> GroupRepository<B> for NoopRepository<B> {
    fn last_message(&self, _: ChannelId) -> GetEntityFuture<'_, MessageEntity, B::Error> {
        future::ok(None).boxed()
    }

    fn owner(&self, _: ChannelId) -> GetEntityFuture<'_, UserEntity, B::Error> {
        future::ok(None).boxed()
    }

    fn recipients(&self, _: ChannelId) -> ListEntitiesFuture<'_, UserEntity, B::Error> {
        future::ok(stream::empty().boxed()).boxed()
    }
}

impl<B: Backend + Send> GuildRepository<B> for NoopRepository<B> {
    fn afk_channel(&self, _: GuildId) -> GetEntityFuture<'_, VoiceChannelEntity, B::Error> {
        future::ok(None).boxed()
    }

    fn channel_ids(&self, _: GuildId) -> super::ListEntityIdsFuture<'_, ChannelId, B::Error> {
        future::ok(stream::empty().boxed()).boxed()
    }

    fn channels(
        &self,
        _: GuildId,
    ) -> ListEntitiesFuture<'_, crate::entity::channel::GuildChannelEntity, B::Error> {
        future::ok(stream::empty().boxed()).boxed()
    }

    fn emoji_ids(&self, _: GuildId) -> super::ListEntityIdsFuture<'_, EmojiId, B::Error> {
        future::ok(stream::empty().boxed()).boxed()
    }

    fn emojis(&self, _: GuildId) -> ListEntitiesFuture<'_, EmojiEntity, B::Error> {
        future::ok(stream::empty().boxed()).boxed()
    }

    fn member_ids(&self, _: GuildId) -> super::ListEntityIdsFuture<'_, UserId, B::Error> {
        future::ok(stream::empty().boxed()).boxed()
    }

    fn members(&self, _: GuildId) -> ListEntitiesFuture<'_, MemberEntity, B::Error> {
        future::ok(stream::empty().boxed()).boxed()
    }

    fn owner(&self, _: GuildId) -> GetEntityFuture<'_, UserEntity, B::Error> {
        future::ok(None).boxed()
    }

    fn presence_ids(&self, _: GuildId) -> super::ListEntityIdsFuture<'_, UserId, B::Error> {
        future::ok(stream::empty().boxed()).boxed()
    }

    fn presences(&self, _: GuildId) -> ListEntitiesFuture<'_, PresenceEntity, B::Error> {
        future::ok(stream::empty().boxed()).boxed()
    }

    fn role_ids(&self, _: GuildId) -> super::ListEntityIdsFuture<'_, RoleId, B::Error> {
        future::ok(stream::empty().boxed()).boxed()
    }

    fn roles(&self, _: GuildId) -> ListEntitiesFuture<'_, RoleEntity, B::Error> {
        future::ok(stream::empty().boxed()).boxed()
    }

    fn rules_channel(&self, _: GuildId) -> GetEntityFuture<'_, TextChannelEntity, B::Error> {
        future::ok(None).boxed()
    }

    fn system_channel(&self, _: GuildId) -> GetEntityFuture<'_, TextChannelEntity, B::Error> {
        future::ok(None).boxed()
    }

    fn voice_state_ids(&self, _: GuildId) -> super::ListEntityIdsFuture<'_, UserId, B::Error> {
        future::ok(stream::empty().boxed()).boxed()
    }

    fn voice_states(&self, _: GuildId) -> ListEntitiesFuture<'_, VoiceStateEntity, B::Error> {
        future::ok(stream::empty().boxed()).boxed()
    }

    fn widget_channel(
        &self,
        _: GuildId,
    ) -> GetEntityFuture<'_, crate::entity::channel::GuildChannelEntity, B::Error> {
        future::ok(None).boxed()
    }
}

impl<B: Backend + Send> MemberRepository<B> for NoopRepository<B> {
    fn hoisted_role(&self, _: GuildId, _: UserId) -> GetEntityFuture<'_, RoleEntity, B::Error> {
        future::ok(None).boxed()
    }

    fn roles(&self, _: GuildId, _: UserId) -> ListEntitiesFuture<'_, RoleEntity, B::Error> {
        future::ok(stream::empty().boxed()).boxed()
    }
}

impl<B: Backend + Send> MessageRepository<B> for NoopRepository<B> {
    fn attachments(&self, _: MessageId) -> ListEntitiesFuture<'_, AttachmentEntity, B::Error> {
        future::ok(stream::empty().boxed()).boxed()
    }

    fn author(
        &self,
        _: MessageId,
    ) -> GetEntityFuture<'_, crate::entity::user::UserEntity, B::Error> {
        future::ok(None).boxed()
    }

    fn channel(&self, _: MessageId) -> GetEntityFuture<'_, ChannelEntity, B::Error> {
        future::ok(None).boxed()
    }

    fn guild(
        &self,
        _: MessageId,
    ) -> GetEntityFuture<'_, crate::entity::guild::GuildEntity, B::Error> {
        future::ok(None).boxed()
    }

    fn mention_channels(
        &self,
        _: MessageId,
    ) -> ListEntitiesFuture<'_, TextChannelEntity, B::Error> {
        future::ok(stream::empty().boxed()).boxed()
    }

    fn mention_roles(
        &self,
        _: MessageId,
    ) -> ListEntitiesFuture<'_, crate::entity::guild::RoleEntity, B::Error> {
        future::ok(stream::empty().boxed()).boxed()
    }

    fn mentions(
        &self,
        _: MessageId,
    ) -> ListEntitiesFuture<'_, crate::entity::user::UserEntity, B::Error> {
        future::ok(stream::empty().boxed()).boxed()
    }
}

impl<B: Backend + Send> PresenceRepository<B> for NoopRepository<B> {}

impl<B: Backend + Send> PrivateChannelRepository<B> for NoopRepository<B> {
    fn last_message(&self, _: ChannelId) -> GetEntityFuture<'_, MessageEntity, B::Error> {
        future::ok(None).boxed()
    }

    fn recipient(&self, _: ChannelId) -> GetEntityFuture<'_, UserEntity, B::Error> {
        future::ok(None).boxed()
    }
}

impl<B: Backend + Send> RoleRepository<B> for NoopRepository<B> {
    fn guild(&self, _: RoleId) -> GetEntityFuture<'_, GuildEntity, B::Error> {
        future::ok(None).boxed()
    }
}

impl<B: Backend + Send> TextChannelRepository<B> for NoopRepository<B> {
    fn guild(&self, _: ChannelId) -> GetEntityFuture<'_, GuildEntity, B::Error> {
        future::ok(None).boxed()
    }

    fn last_message(&self, _: ChannelId) -> GetEntityFuture<'_, MessageEntity, B::Error> {
        future::ok(None).boxed()
    }

    fn parent(&self, _: ChannelId) -> GetEntityFuture<'_, CategoryChannelEntity, B::Error> {
        future::ok(None).boxed()
    }
}

impl<B: Backend + Send> UserRepository<B> for NoopRepository<B> {
    fn guild_ids(&self, _: UserId) -> super::ListEntityIdsFuture<'_, GuildId, B::Error> {
        future::ok(stream::empty().boxed()).boxed()
    }

    fn guilds(&self, _: UserId) -> ListEntitiesFuture<'_, GuildEntity, B::Error> {
        future::ok(stream::empty().boxed()).boxed()
    }
}

impl<B: Backend + Send> VoiceChannelRepository<B> for NoopRepository<B> {
    fn guild(&self, _: ChannelId) -> GetEntityFuture<'_, GuildEntity, B::Error> {
        future::ok(None).boxed()
    }

    fn parent(&self, _: ChannelId) -> GetEntityFuture<'_, CategoryChannelEntity, B::Error> {
        future::ok(None).boxed()
    }
}

impl<B: Backend + Send> VoiceStateRepository<B> for NoopRepository<B> {
    fn channel(&self, _: GuildId, _: UserId) -> GetEntityFuture<'_, VoiceChannelEntity, B::Error> {
        future::ok(None).boxed()
    }
}
