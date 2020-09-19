//! # Examples
//!
//! Knowing the ID of an emoji, get the user associated with the emoji in the
//! cache:
//!
//! > (note that, of course, both the emoji and the user must be in the cache)
//!
//! ```rust,no_run
//! use rarity_cache_inmemory::InMemoryCache;
//! use twilight_model::id::EmojiId;
//!
//! # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // the ID of the emoji. we're going to get its user
//! let emoji_id = EmojiId(123456789012345678);
//!
//! let cache = InMemoryCache::new();
//!
//! if let Some(user) = cache.emojis.user(emoji_id).await? {
//!     println!("the person who made the emoji is {}", user.name);
//! } else {
//!     println!("the emoji or its user isn't in the cache");
//! }
//! # Ok(()) }
//! ```
//!
//! Get information about a guild, and then iterate over an asynchronous stream
//! of its members:
//!
//! > You don't need to do either of these to do the other, this is just an
//! > example of doing both things!
//!
//! ```rust,no_run
//! use futures::StreamExt;
//! use rarity_cache_inmemory::{InMemoryCache, Repository};
//! use twilight_model::id::GuildId;
//!
//! # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let guild_id = GuildId(123456789012345678);
//!
//! let cache = InMemoryCache::new();
//!
//! if let Some(guild) = cache.guilds.get(guild_id).await? {
//!     println!("the guild's name is {}", guild.name);
//! }
//!
//! // now asynchronously iterate over its members
//! let mut members = cache.guilds.members(guild_id).await?;
//!
//! while let Some(member) = members.next().await {
//!     // the member is wrapped in an error since there can be an error from
//!     // the cache backend, such as during deserialisation
//!     let member = member?;
//!
//!     println!("the member's user id is {}", member.user_id);
//! }
//! # Ok(()) }
//! ```

#![deny(
    clippy::all,
    clippy::pedantic,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms,
    unused,
    warnings
)]
#![allow(clippy::module_name_repetitions, clippy::must_use_candidate)]

pub extern crate rarity_cache as cache;

pub mod config;
pub mod prelude;
pub mod repository;

#[doc(no_inline)]
pub use rarity_cache::Repository;

use self::{
    config::{Config, EntityType},
    repository::{
        InMemoryAttachmentRepository, InMemoryCategoryChannelRepository, InMemoryEmojiRepository,
        InMemoryGroupRepository, InMemoryGuildRepository, InMemoryMemberRepository,
        InMemoryMessageRepository, InMemoryPresenceRepository, InMemoryPrivateChannelRepository,
        InMemoryRoleRepository, InMemoryTextChannelRepository, InMemoryUserRepository,
        InMemoryVoiceChannelRepository, InMemoryVoiceStateRepository,
    },
};
use dashmap::DashMap;
use rarity_cache::{
    entity::{
        channel::{
            AttachmentEntity, CategoryChannelEntity, GroupEntity, MessageEntity,
            PrivateChannelEntity, TextChannelEntity, VoiceChannelEntity,
        },
        gateway::PresenceEntity,
        guild::{EmojiEntity, GuildEntity, MemberEntity, RoleEntity},
        user::UserEntity,
        voice::VoiceStateEntity,
    },
    Backend, Cache,
};
use std::{
    collections::{BTreeSet, HashSet},
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    sync::Arc,
};
use twilight_model::id::{AttachmentId, ChannelId, EmojiId, GuildId, MessageId, RoleId, UserId};

/// Alias over `rarity_cache::Cache` which uses the [`InMemoryBackend`].
///
/// This allows you to use the in-memory backend like:
///
/// ```
/// use rarity_cache_inmemory::{InMemoryCache, Repository};
/// use twilight_model::id::UserId;
///
/// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // notably this line:
/// let cache = InMemoryCache::new();
///
/// if let Some(user) = cache.users.get(UserId(123)).await? {
///     println!("username: {}", user.name);
/// }
/// # Ok(()) }
/// ```
///
/// [`InMemoryBackend`]: struct.InMemoryBackend.html
pub type InMemoryCache = Cache<InMemoryBackend>;

/// Error returned from backend operations.
///
/// This error type has no variants and will never occur. It currently only
/// exists to satisfy the constraints of cache repositories.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct InMemoryBackendError;

impl Display for InMemoryBackendError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("this can't be constructed")
    }
}

impl Error for InMemoryBackendError {}

#[derive(Debug, Default)]
struct InMemoryBackendRef {
    attachments: DashMap<AttachmentId, AttachmentEntity>,
    channels_category: DashMap<ChannelId, CategoryChannelEntity>,
    channels_private: DashMap<ChannelId, PrivateChannelEntity>,
    channels_text: DashMap<ChannelId, TextChannelEntity>,
    channels_voice: DashMap<ChannelId, VoiceChannelEntity>,
    channel_messages: DashMap<ChannelId, BTreeSet<MessageId>>,
    config: Config,
    emojis: DashMap<EmojiId, EmojiEntity>,
    groups: DashMap<ChannelId, GroupEntity>,
    guilds: DashMap<GuildId, GuildEntity>,
    guild_channels: DashMap<GuildId, HashSet<ChannelId>>,
    guild_emojis: DashMap<GuildId, HashSet<EmojiId>>,
    guild_members: DashMap<GuildId, HashSet<UserId>>,
    guild_presences: DashMap<GuildId, HashSet<UserId>>,
    guild_roles: DashMap<GuildId, HashSet<RoleId>>,
    guild_voice_states: DashMap<GuildId, HashSet<UserId>>,
    members: DashMap<(GuildId, UserId), MemberEntity>,
    messages: DashMap<MessageId, MessageEntity>,
    presences: DashMap<(GuildId, UserId), PresenceEntity>,
    roles: DashMap<RoleId, RoleEntity>,
    users: DashMap<UserId, UserEntity>,
    user_guilds: DashMap<UserId, Vec<GuildId>>,
    voice_states: DashMap<(GuildId, UserId), VoiceStateEntity>,
}

/// Builder to create a configured [`InMemoryBackend`].
///
/// [`InMemoryBackend`]: struct.InMemoryBackend.html
#[derive(Clone, Debug, Default)]
pub struct InMemoryBackendBuilder(Config);

impl InMemoryBackendBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(self) -> InMemoryBackend {
        InMemoryBackend(Arc::new(InMemoryBackendRef {
            config: self.0,
            ..InMemoryBackendRef::default()
        }))
    }

    pub fn entity_types(&mut self, entity_types: EntityType) -> &mut Self {
        *self.0.entity_types_mut() = entity_types;

        self
    }

    pub fn message_cache_size(&mut self, message_cache_size: usize) -> &mut Self {
        *self.0.message_cache_size_mut() = message_cache_size;

        self
    }
}

/// Backend implementation to cache entities in the process's memory.
#[derive(Clone, Debug, Default)]
pub struct InMemoryBackend(Arc<InMemoryBackendRef>);

impl InMemoryBackend {
    /// Create a new default backend.
    ///
    /// Read [`Config`] to know the default configuration.
    ///
    /// Use [`InMemoryBackend::builder`] to create a backend from a
    /// configuration.
    ///
    /// # Examples
    ///
    /// Create a new cache backend and then create a `rarity_cache::Cache`
    /// with it:
    ///
    /// ```
    /// use rarity_cache::Cache;
    /// use rarity_cache_inmemory::InMemoryBackend;
    ///
    /// let backend = InMemoryBackend::new();
    /// let cache = Cache::with_backend(backend);
    /// ```
    ///
    /// Create a new cache with this backend via [`InMemoryCache`], which is
    /// shorthand for above:
    ///
    /// ```
    /// use rarity_cache_inmemory::InMemoryCache;
    ///
    /// let cache = InMemoryCache::new();
    /// ```
    ///
    /// [`Config`]: config/struct.Config.html
    /// [`InMemoryBackend::builder`]: #method.builder
    /// [from]: #impl-From<T>
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a builder to create a configured in memory backend.
    ///
    /// Refer to its documentation for more information.
    ///
    /// # Examples
    ///
    /// Create a backend which caches up to the latest 50 messages per channel
    /// and only caches messages and users:
    ///
    /// ```
    /// use rarity_cache_inmemory::{config::EntityType, InMemoryBackend};
    ///
    /// let mut builder = InMemoryBackend::builder();
    /// builder
    ///     .entity_types(EntityType::MESSAGE | EntityType::USER)
    ///     .message_cache_size(50);
    ///
    /// let cache = builder.build();
    /// ```
    pub fn builder() -> InMemoryBackendBuilder {
        InMemoryBackendBuilder::new()
    }

    /// Return a copy of the cache configuration.
    pub fn config(&self) -> Config {
        self.0.config.clone()
    }
}

/// In memory implementation of a `rarity_cache` backend.
///
/// **Note**: you should probably not be using the trait's methods directly, and
/// should wrap a backend instance in `rarity_cache`'s `Cache` and use its
/// methods and fields instead.
impl Backend for InMemoryBackend {
    type Error = InMemoryBackendError;
    type AttachmentRepository = InMemoryAttachmentRepository;
    type CategoryChannelRepository = InMemoryCategoryChannelRepository;
    type EmojiRepository = InMemoryEmojiRepository;
    type GroupRepository = InMemoryGroupRepository;
    type GuildRepository = InMemoryGuildRepository;
    type MemberRepository = InMemoryMemberRepository;
    type MessageRepository = InMemoryMessageRepository;
    type PresenceRepository = InMemoryPresenceRepository;
    type PrivateChannelRepository = InMemoryPrivateChannelRepository;
    type RoleRepository = InMemoryRoleRepository;
    type TextChannelRepository = InMemoryTextChannelRepository;
    type UserRepository = InMemoryUserRepository;
    type VoiceChannelRepository = InMemoryVoiceChannelRepository;
    type VoiceStateRepository = InMemoryVoiceStateRepository;

    /// A new instance of a repository for working with attachments.
    fn attachments(&self) -> Self::AttachmentRepository {
        InMemoryAttachmentRepository(self.clone())
    }

    /// A new instance of a repository for working with guild category channels.
    fn category_channels(&self) -> Self::CategoryChannelRepository {
        InMemoryCategoryChannelRepository(self.clone())
    }

    /// A new instance of a repository for working with emojis.
    fn emojis(&self) -> Self::EmojiRepository {
        InMemoryEmojiRepository(self.clone())
    }

    /// A new instance of a repository for working with groups.
    fn groups(&self) -> Self::GroupRepository {
        InMemoryGroupRepository(self.clone())
    }

    /// A new instance of a repository for working with guilds.
    fn guilds(&self) -> Self::GuildRepository {
        InMemoryGuildRepository(self.clone())
    }

    /// A new instance of a repository for working with members.
    fn members(&self) -> Self::MemberRepository {
        InMemoryMemberRepository(self.clone())
    }

    /// A new instance of a repository for working with messages.
    fn messages(&self) -> Self::MessageRepository {
        InMemoryMessageRepository(self.clone())
    }

    /// A new instance of a repository for working with presences.
    fn presences(&self) -> Self::PresenceRepository {
        InMemoryPresenceRepository(self.clone())
    }

    /// A new instance of a repository for working with private channels.
    fn private_channels(&self) -> Self::PrivateChannelRepository {
        InMemoryPrivateChannelRepository(self.clone())
    }

    /// A new instance of a repository for working with roles.
    fn roles(&self) -> Self::RoleRepository {
        InMemoryRoleRepository(self.clone())
    }

    /// A new instance of a repository for working with guild text channels.
    fn text_channels(&self) -> Self::TextChannelRepository {
        InMemoryTextChannelRepository(self.clone())
    }

    /// A new instance of a repository for working with users.
    fn users(&self) -> Self::UserRepository {
        InMemoryUserRepository(self.clone())
    }

    /// A new instance of a repository for working with guild voice channels.
    fn voice_channels(&self) -> Self::VoiceChannelRepository {
        InMemoryVoiceChannelRepository(self.clone())
    }

    /// A new instance of a repository for working with voice states.
    fn voice_states(&self) -> Self::VoiceStateRepository {
        InMemoryVoiceStateRepository(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::{InMemoryBackendBuilder, InMemoryBackendError, InMemoryBackend, InMemoryCache};
    use rarity_cache::Backend;
    use static_assertions::{assert_impl_all, assert_obj_safe};
    use std::{error::Error, fmt::Debug};

    assert_impl_all!(InMemoryBackendBuilder: Clone, Debug, Default, Send, Sync);
    assert_impl_all!(InMemoryBackendError: Clone, Debug, Error, Send, Sync);
    assert_impl_all!(InMemoryBackend: Backend, Clone, Debug, Send, Sync);
    assert_impl_all!(InMemoryCache: Clone, Debug, Send, Sync);
    assert_obj_safe!(InMemoryBackendBuilder, InMemoryBackendError, InMemoryBackend, InMemoryCache);
}
