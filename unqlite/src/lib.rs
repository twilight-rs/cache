use futures_util::future::{self, FutureExt};
use rarity_cache::{
    entity::{
        channel::{
            attachment::{AttachmentEntity, AttachmentRepository},
            category_channel::{CategoryChannelEntity, CategoryChannelRepository},
            group::{GroupEntity, GroupRepository},
            message::{MessageEntity, MessageRepository},
            private_channel::{PrivateChannelEntity, PrivateChannelRepository},
            text_channel::{TextChannelEntity, TextChannelRepository},
            voice_channel::{VoiceChannelEntity, VoiceChannelRepository},
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
    repository::{GetEntityFuture, ListEntitiesFuture, RemoveEntityFuture, UpsertEntityFuture},
    Backend, Cache, Repository,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{marker::PhantomData, sync::Arc};
use twilight_model::id::{AttachmentId, ChannelId, EmojiId, GuildId, MessageId, RoleId, UserId};
use unqlite::{Error, UnQLite, KV};

pub type UnqliteCache = Cache<UnqliteBackend>;

pub trait UnqliteEntity: Entity {
    fn key(id: Self::Id) -> Vec<u8>;
}

impl UnqliteEntity for AttachmentEntity {
    fn key(id: AttachmentId) -> Vec<u8> {
        format!("at:{}", id).into_bytes()
    }
}

impl UnqliteEntity for CategoryChannelEntity {
    fn key(id: ChannelId) -> Vec<u8> {
        format!("cc:{}", id).into_bytes()
    }
}

impl UnqliteEntity for EmojiEntity {
    fn key(id: EmojiId) -> Vec<u8> {
        format!("em:{}", id).into_bytes()
    }
}

impl UnqliteEntity for GroupEntity {
    fn key(id: ChannelId) -> Vec<u8> {
        format!("gr:{}", id).into_bytes()
    }
}

impl UnqliteEntity for GuildEntity {
    fn key(id: GuildId) -> Vec<u8> {
        format!("g:{}", id).into_bytes()
    }
}

impl UnqliteEntity for MemberEntity {
    fn key((guild_id, user_id): (GuildId, UserId)) -> Vec<u8> {
        format!("m:{}:{}", guild_id, user_id).into_bytes()
    }
}

impl UnqliteEntity for MessageEntity {
    fn key(id: MessageId) -> Vec<u8> {
        format!("ms:{}", id).into_bytes()
    }
}

impl UnqliteEntity for PresenceEntity {
    fn key((guild_id, user_id): (GuildId, UserId)) -> Vec<u8> {
        format!("pr:{}:{}", guild_id, user_id).into_bytes()
    }
}

impl UnqliteEntity for PrivateChannelEntity {
    fn key(id: ChannelId) -> Vec<u8> {
        format!("cp:{}", id).into_bytes()
    }
}

impl UnqliteEntity for RoleEntity {
    fn key(id: RoleId) -> Vec<u8> {
        format!("r:{}", id).into_bytes()
    }
}

impl UnqliteEntity for TextChannelEntity {
    fn key(id: ChannelId) -> Vec<u8> {
        format!("ct:{}", id).into_bytes()
    }
}

impl UnqliteEntity for UserEntity {
    fn key(id: UserId) -> Vec<u8> {
        format!("u:{}", id).into_bytes()
    }
}

impl UnqliteEntity for VoiceChannelEntity {
    fn key(id: ChannelId) -> Vec<u8> {
        format!("cv:{}", id).into_bytes()
    }
}

impl UnqliteEntity for VoiceStateEntity {
    fn key((guild_id, user_id): (GuildId, UserId)) -> Vec<u8> {
        format!("v:{}:{}", guild_id, user_id).into_bytes()
    }
}

pub struct UnqliteRepository<T>(UnqliteBackend, PhantomData<T>);

impl<T> UnqliteRepository<T> {
    fn new(backend: UnqliteBackend) -> Self {
        Self(backend, PhantomData)
    }
}

impl<T: DeserializeOwned + Serialize + UnqliteEntity> Repository<T, UnqliteBackend>
    for UnqliteRepository<T>
{
    fn backend(&self) -> UnqliteBackend {
        self.0.clone()
    }

    fn get(&self, entity_id: T::Id) -> GetEntityFuture<'_, T, Error> {
        let bytes: Vec<u8> = (self.0).0.kv_fetch(T::key(entity_id)).unwrap();

        future::ok(Some(serde_cbor::from_slice::<T>(&bytes).unwrap())).boxed()
    }

    fn list(&self) -> ListEntitiesFuture<'_, T, Error> {
        unimplemented!("not implemented by this backend");
    }

    fn remove(&self, entity_id: T::Id) -> RemoveEntityFuture<'_, Error> {
        future::ready((self.0).0.kv_delete(T::key(entity_id))).boxed()
    }

    fn upsert(&self, entity: T) -> UpsertEntityFuture<'_, Error> {
        let bytes = serde_cbor::to_vec(&entity).unwrap();

        future::ready((self.0).0.kv_store(T::key(entity.id()), bytes)).boxed()
    }
}

impl AttachmentRepository<UnqliteBackend> for UnqliteRepository<AttachmentEntity> {}

impl CategoryChannelRepository<UnqliteBackend> for UnqliteRepository<CategoryChannelEntity> {}

impl EmojiRepository<UnqliteBackend> for UnqliteRepository<EmojiEntity> {}

impl GroupRepository<UnqliteBackend> for UnqliteRepository<GroupEntity> {}

impl GuildRepository<UnqliteBackend> for UnqliteRepository<GuildEntity> {
    fn channel_ids(
        &self,
        _: GuildId,
    ) -> rarity_cache::repository::ListEntityIdsFuture<'_, ChannelId, Error> {
        unimplemented!("not implemented by this backend");
    }

    fn channels(
        &self,
        _: GuildId,
    ) -> ListEntitiesFuture<'_, rarity_cache::entity::channel::GuildChannelEntity, Error> {
        unimplemented!("not implemented by this backend");
    }

    fn emoji_ids(
        &self,
        _: GuildId,
    ) -> rarity_cache::repository::ListEntityIdsFuture<'_, EmojiId, Error> {
        unimplemented!("not implemented by this backend");
    }

    fn member_ids(
        &self,
        _: GuildId,
    ) -> rarity_cache::repository::ListEntityIdsFuture<'_, UserId, Error> {
        unimplemented!("not implemented by this backend");
    }

    fn members(&self, _: GuildId) -> ListEntitiesFuture<'_, MemberEntity, Error> {
        unimplemented!("not implemented by this backend");
    }

    fn presence_ids(
        &self,
        _: GuildId,
    ) -> rarity_cache::repository::ListEntityIdsFuture<'_, UserId, Error> {
        unimplemented!("not implemented by this backend");
    }

    fn presences(&self, _: GuildId) -> ListEntitiesFuture<'_, PresenceEntity, Error> {
        unimplemented!("not implemented by this backend");
    }

    fn role_ids(
        &self,
        _: GuildId,
    ) -> rarity_cache::repository::ListEntityIdsFuture<'_, RoleId, Error> {
        unimplemented!("not implemented by this backend");
    }

    fn voice_state_ids(
        &self,
        _: GuildId,
    ) -> rarity_cache::repository::ListEntityIdsFuture<'_, UserId, Error> {
        unimplemented!("not implemented by this backend");
    }

    fn voice_states(&self, _: GuildId) -> ListEntitiesFuture<'_, VoiceStateEntity, Error> {
        unimplemented!("not implemented by this backend");
    }
}

impl MemberRepository<UnqliteBackend> for UnqliteRepository<MemberEntity> {}

impl MessageRepository<UnqliteBackend> for UnqliteRepository<MessageEntity> {}

impl PresenceRepository<UnqliteBackend> for UnqliteRepository<PresenceEntity> {}

impl PrivateChannelRepository<UnqliteBackend> for UnqliteRepository<PrivateChannelEntity> {}

impl RoleRepository<UnqliteBackend> for UnqliteRepository<RoleEntity> {}

impl TextChannelRepository<UnqliteBackend> for UnqliteRepository<TextChannelEntity> {}

impl VoiceChannelRepository<UnqliteBackend> for UnqliteRepository<VoiceChannelEntity> {}

impl VoiceStateRepository<UnqliteBackend> for UnqliteRepository<VoiceStateEntity> {}

impl UserRepository<UnqliteBackend> for UnqliteRepository<UserEntity> {
    fn guild_ids(
        &self,
        _: UserId,
    ) -> rarity_cache::repository::ListEntityIdsFuture<'_, GuildId, Error> {
        unimplemented!("not implemented by this backend")
    }
}

/// `rarity-cache` backend for the [UnQLite] database.
///
/// [UnQLite]: https://docs.rs/unqlite
#[derive(Clone)]
pub struct UnqliteBackend(Arc<UnQLite>);

impl UnqliteBackend {
    /// Create a new `rarity-cache` UnQLite backend with a provided instance.
    pub fn new(unqlite: UnQLite) -> Self {
        Self(Arc::new(unqlite))
    }

    /// Shortcut for `UnQLite::create` and [`new`].
    ///
    /// [`new`]: #method.new
    pub fn create(filename: impl AsRef<str>) -> UnQLite {
        UnQLite::create(filename)
    }

    /// Shortcut for `UnQLite::create_in_memory` and [`new`].
    ///
    /// [`new`]: #method.new
    pub fn create_in_memory() -> UnQLite {
        UnQLite::create_in_memory()
    }

    /// Shortcut for `UnQLite::create_temp` and [`new`].
    ///
    /// [`new`]: #method.new
    pub fn create_temp() -> UnQLite {
        UnQLite::create_temp()
    }

    /// Shortcut for `UnQLite::open_mmap` and [`new`].
    ///
    /// [`new`]: #method.new
    pub fn open_mmap(filename: impl AsRef<str>) -> UnQLite {
        UnQLite::open_mmap(filename)
    }

    /// Shortcut for `UnQLite::open_readonly` and [`new`].
    ///
    /// [`new`]: #method.new
    pub fn open_readonly(filename: impl AsRef<str>) -> UnQLite {
        UnQLite::open_readonly(filename)
    }

    fn repo<T>(&self) -> UnqliteRepository<T> {
        UnqliteRepository::new(self.clone())
    }
}

impl Backend for UnqliteBackend {
    type Error = Error;
    type AttachmentRepository = UnqliteRepository<AttachmentEntity>;
    type CategoryChannelRepository = UnqliteRepository<CategoryChannelEntity>;
    type EmojiRepository = UnqliteRepository<EmojiEntity>;
    type GroupRepository = UnqliteRepository<GroupEntity>;
    type GuildRepository = UnqliteRepository<GuildEntity>;
    type MemberRepository = UnqliteRepository<MemberEntity>;
    type MessageRepository = UnqliteRepository<MessageEntity>;
    type PresenceRepository = UnqliteRepository<PresenceEntity>;
    type PrivateChannelRepository = UnqliteRepository<PrivateChannelEntity>;
    type RoleRepository = UnqliteRepository<RoleEntity>;
    type TextChannelRepository = UnqliteRepository<TextChannelEntity>;
    type UserRepository = UnqliteRepository<UserEntity>;
    type VoiceChannelRepository = UnqliteRepository<VoiceChannelEntity>;
    type VoiceStateRepository = UnqliteRepository<VoiceStateEntity>;

    fn attachments(&self) -> Self::AttachmentRepository {
        self.repo()
    }

    fn category_channels(&self) -> Self::CategoryChannelRepository {
        self.repo()
    }

    fn emojis(&self) -> Self::EmojiRepository {
        self.repo()
    }

    fn groups(&self) -> Self::GroupRepository {
        self.repo()
    }

    fn guilds(&self) -> Self::GuildRepository {
        self.repo()
    }

    fn members(&self) -> Self::MemberRepository {
        self.repo()
    }

    fn messages(&self) -> Self::MessageRepository {
        self.repo()
    }

    fn presences(&self) -> Self::PresenceRepository {
        self.repo()
    }

    fn private_channels(&self) -> Self::PrivateChannelRepository {
        self.repo()
    }

    fn roles(&self) -> Self::RoleRepository {
        self.repo()
    }

    fn text_channels(&self) -> Self::TextChannelRepository {
        self.repo()
    }

    fn users(&self) -> Self::UserRepository {
        self.repo()
    }

    fn voice_channels(&self) -> Self::VoiceChannelRepository {
        self.repo()
    }

    fn voice_states(&self) -> Self::VoiceStateRepository {
        self.repo()
    }
}
