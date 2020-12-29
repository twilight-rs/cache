//! # Examples
//!
//! Knowing the ID of an emoji, get the user associated with the emoji in the
//! cache:
//!
//! > (note that, of course, both the emoji and the user must be in the cache)
//!
//! ```rust,no_run
//! use twilight_cache_inmemory::{prelude::*, InMemoryCache};
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
//! use twilight_cache_inmemory::{prelude::*, InMemoryCache, Repository};
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

pub extern crate twilight_cache as cache;

pub mod config;
pub mod prelude;
pub mod repository;

#[doc(no_inline)]
pub use twilight_cache::Repository;

use self::{
    config::{Config, EntityType},
    repository::{
        InMemoryAttachmentRepository, InMemoryCategoryChannelRepository,
        InMemoryCurrentUserRepository, InMemoryEmojiRepository, InMemoryGroupRepository,
        InMemoryGuildRepository, InMemoryMemberRepository, InMemoryMessageRepository,
        InMemoryPresenceRepository, InMemoryPrivateChannelRepository, InMemoryRepository,
        InMemoryRoleRepository, InMemoryTextChannelRepository, InMemoryUserRepository,
        InMemoryVoiceChannelRepository, InMemoryVoiceStateRepository,
    },
};
use dashmap::DashMap;
use std::{
    collections::{BTreeSet, HashSet},
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
    marker::PhantomData,
    sync::{Arc, Mutex},
};
use twilight_cache::{
    entity::{
        channel::{
            AttachmentEntity, CategoryChannelEntity, GroupEntity, MessageEntity,
            PrivateChannelEntity, TextChannelEntity, VoiceChannelEntity,
        },
        gateway::PresenceEntity,
        guild::{EmojiEntity, GuildEntity, MemberEntity, RoleEntity},
        user::{CurrentUserEntity, UserEntity},
        voice::VoiceStateEntity,
    },
    Backend, Cache,
};
use twilight_model::id::{AttachmentId, ChannelId, EmojiId, GuildId, MessageId, RoleId, UserId};

/// Alias over `twilight_cache::Cache` which uses the [`InMemoryBackend`].
///
/// This allows you to use the in-memory backend like:
///
/// ```
/// use twilight_cache_inmemory::{InMemoryCache, Repository};
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
    user_current: Mutex<Option<CurrentUserEntity>>,
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
    /// Create a new cache backend and then create a `twilight_cache::Cache`
    /// with it:
    ///
    /// ```
    /// use twilight_cache::Cache;
    /// use twilight_cache_inmemory::InMemoryBackend;
    ///
    /// let backend = InMemoryBackend::new();
    /// let cache = Cache::with_backend(backend);
    /// ```
    ///
    /// Create a new cache with this backend via [`InMemoryCache`], which is
    /// shorthand for above:
    ///
    /// ```
    /// use twilight_cache_inmemory::InMemoryCache;
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
    /// use twilight_cache_inmemory::{config::EntityType, InMemoryBackend};
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

    fn repo<T>(&self) -> InMemoryRepository<T> {
        InMemoryRepository(self.clone(), PhantomData)
    }
}

/// In memory implementation of a `twilight_cache` backend.
///
/// **Note**: you should probably not be using the trait's methods directly, and
/// should wrap a backend instance in `twilight_cache`'s `Cache` and use its
/// methods and fields instead.
impl Backend for InMemoryBackend {
    type Error = InMemoryBackendError;
    type AttachmentRepository = InMemoryAttachmentRepository;
    type CategoryChannelRepository = InMemoryCategoryChannelRepository;
    type CurrentUserRepository = InMemoryCurrentUserRepository;
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
        self.repo()
    }

    /// A new instance of a repository for working with guild category channels.
    fn category_channels(&self) -> Self::CategoryChannelRepository {
        self.repo()
    }

    /// A new instance of a repository for working with the current user.
    fn current_user(&self) -> Self::CurrentUserRepository {
        self.repo()
    }

    /// A new instance of a repository for working with emojis.
    fn emojis(&self) -> Self::EmojiRepository {
        self.repo()
    }

    /// A new instance of a repository for working with groups.
    fn groups(&self) -> Self::GroupRepository {
        self.repo()
    }

    /// A new instance of a repository for working with guilds.
    fn guilds(&self) -> Self::GuildRepository {
        self.repo()
    }

    /// A new instance of a repository for working with members.
    fn members(&self) -> Self::MemberRepository {
        self.repo()
    }

    /// A new instance of a repository for working with messages.
    fn messages(&self) -> Self::MessageRepository {
        self.repo()
    }

    /// A new instance of a repository for working with presences.
    fn presences(&self) -> Self::PresenceRepository {
        self.repo()
    }

    /// A new instance of a repository for working with private channels.
    fn private_channels(&self) -> Self::PrivateChannelRepository {
        self.repo()
    }

    /// A new instance of a repository for working with roles.
    fn roles(&self) -> Self::RoleRepository {
        self.repo()
    }

    /// A new instance of a repository for working with guild text channels.
    fn text_channels(&self) -> Self::TextChannelRepository {
        self.repo()
    }

    /// A new instance of a repository for working with users.
    fn users(&self) -> Self::UserRepository {
        self.repo()
    }

    /// A new instance of a repository for working with guild voice channels.
    fn voice_channels(&self) -> Self::VoiceChannelRepository {
        self.repo()
    }

    /// A new instance of a repository for working with voice states.
    fn voice_states(&self) -> Self::VoiceStateRepository {
        self.repo()
    }
}

#[cfg(test)]
mod tests {
    use super::{prelude::*, InMemoryBackendBuilder};
    use futures_util::stream::StreamExt;
    use static_assertions::{assert_impl_all, assert_obj_safe};
    use std::{collections::HashMap, error::Error, fmt::Debug};
    use twilight_cache::{
        entity::{
            channel::{
                CategoryChannelEntity, GroupEntity, PrivateChannelEntity, TextChannelEntity,
                VoiceChannelEntity,
            },
            guild::{GuildEntity, MemberEntity},
            user::{CurrentUserEntity, UserEntity},
        },
        repository::SingleEntityRepository,
        Backend,
    };
    use twilight_model::{
        channel::{
            message::{Message, MessageType},
            Attachment, CategoryChannel, Channel, ChannelType, Group, GuildChannel, PrivateChannel,
            TextChannel, VoiceChannel,
        },
        gateway::{
            event::Event,
            payload::{
                ChannelCreate, ChannelDelete, ChannelPinsUpdate, ChannelUpdate, GuildCreate,
                GuildDelete, GuildEmojisUpdate, GuildUpdate, MemberAdd, MemberChunk, MemberRemove,
                MemberUpdate, MessageCreate, MessageDelete, MessageDeleteBulk, MessageUpdate,
                PresenceUpdate, Ready, RoleCreate, RoleDelete, RoleUpdate, UserUpdate,
                VoiceStateUpdate,
            },
            presence::{ClientStatus, Presence, Status, UserOrId},
        },
        guild::{
            member::Member, DefaultMessageNotificationLevel, Emoji, ExplicitContentFilter, Guild,
            MfaLevel, PartialGuild, PartialMember, Permissions, PremiumTier, Role,
            SystemChannelFlags, VerificationLevel,
        },
        id::{AttachmentId, ChannelId, EmojiId, GuildId, MessageId, RoleId, UserId},
        user::{CurrentUser, User},
        voice::VoiceState,
    };

    assert_impl_all!(InMemoryBackendBuilder: Clone, Debug, Default, Send, Sync);
    assert_impl_all!(InMemoryBackendError: Clone, Debug, Error, Send, Sync);
    assert_impl_all!(InMemoryBackend: Backend, Clone, Debug, Send, Sync);
    assert_impl_all!(InMemoryCache: Clone, Debug, Send, Sync);
    assert_obj_safe!(
        InMemoryBackendBuilder,
        InMemoryBackendError,
        InMemoryBackend,
        InMemoryCache
    );

    fn user() -> User {
        User {
            avatar: None,
            bot: true,
            discriminator: String::from("0001"),
            email: None,
            flags: None,
            id: UserId(2),
            locale: Some(String::from("en-US")),
            mfa_enabled: None,
            name: String::from("user"),
            premium_type: None,
            public_flags: None,
            system: Some(false),
            verified: Some(false),
        }
    }

    fn user2() -> User {
        User {
            avatar: None,
            bot: true,
            discriminator: String::from("0002"),
            email: None,
            flags: None,
            id: UserId(9),
            locale: Some(String::from("en-US")),
            mfa_enabled: None,
            name: String::from("user2"),
            premium_type: None,
            public_flags: None,
            system: Some(false),
            verified: Some(false),
        }
    }

    fn current_user() -> CurrentUser {
        let user = user();

        CurrentUser {
            avatar: user.avatar,
            bot: user.bot,
            discriminator: user.discriminator,
            email: user.email,
            flags: user.flags,
            id: user.id,
            locale: user.locale,
            mfa_enabled: false,
            name: user.name,
            premium_type: user.premium_type,
            public_flags: user.public_flags,
            verified: user.verified,
        }
    }

    fn ready() -> Ready {
        Ready {
            guilds: HashMap::new(),
            session_id: String::from("session"),
            shard: Some([0, 1]),
            user: current_user(),
            version: 8,
        }
    }

    fn member() -> Member {
        Member {
            deaf: false,
            guild_id: GuildId(1),
            hoisted_role: None,
            joined_at: Some(String::from("2012-11-21T10:00:00.40000+00:00")),
            mute: false,
            nick: None,
            premium_since: None,
            roles: Vec::new(),
            user: user(),
        }
    }

    fn member2() -> Member {
        Member {
            deaf: false,
            guild_id: GuildId(1),
            hoisted_role: None,
            joined_at: Some(String::from("2012-11-21T11:00:00.40000+00:00")),
            mute: false,
            nick: None,
            premium_since: None,
            roles: Vec::new(),
            user: user2(),
        }
    }

    fn member_chunk() -> MemberChunk {
        let mut members = HashMap::new();
        let mut presences = HashMap::new();

        for i in 400u64..=410 {
            members.insert(
                UserId(i),
                Member {
                    deaf: false,
                    guild_id: GuildId(1),
                    hoisted_role: None,
                    joined_at: Some(String::from("2012-11-21T10:00:00.40000+00:00")),
                    mute: false,
                    nick: None,
                    premium_since: None,
                    roles: Vec::new(),
                    user: User {
                        avatar: None,
                        bot: true,
                        discriminator: format!("0{}", i),
                        email: None,
                        flags: None,
                        id: UserId(i),
                        locale: Some(String::from("en-US")),
                        mfa_enabled: None,
                        name: format!("user{}", i),
                        premium_type: None,
                        public_flags: None,
                        system: Some(false),
                        verified: Some(false),
                    },
                },
            );

            presences.insert(
                UserId(i),
                Presence {
                    activities: Vec::new(),
                    client_status: ClientStatus {
                        desktop: None,
                        mobile: None,
                        web: None,
                    },
                    guild_id: GuildId(1),
                    status: Status::Offline,
                    user: UserOrId::UserId { id: UserId(i) },
                },
            );
        }

        MemberChunk {
            chunk_count: 1,
            chunk_index: 0,
            guild_id: GuildId(1),
            members,
            nonce: None,
            not_found: Vec::new(),
            presences,
        }
    }

    fn presence_update() -> PresenceUpdate {
        PresenceUpdate {
            activities: Vec::new(),
            client_status: ClientStatus {
                desktop: None,
                mobile: None,
                web: None,
            },
            game: None,
            guild_id: GuildId(1),
            status: Status::Online,
            user: UserOrId::UserId { id: UserId(405) },
        }
    }

    fn emoji() -> Emoji {
        Emoji {
            animated: false,
            available: true,
            id: EmojiId(200),
            managed: false,
            name: String::from("emoji"),
            require_colons: true,
            roles: Vec::new(),
            user: Some(user()),
        }
    }

    fn voice_state() -> VoiceState {
        VoiceState {
            channel_id: Some(ChannelId(6)),
            deaf: false,
            guild_id: Some(GuildId(1)),
            member: Some(member()),
            mute: false,
            self_deaf: false,
            self_mute: false,
            self_stream: false,
            session_id: String::from("session"),
            suppress: false,
            token: Some(String::from("token")),
            user_id: UserId(2),
        }
    }

    fn role() -> Role {
        Role {
            color: 0x000000,
            hoist: false,
            id: RoleId(12),
            managed: false,
            mentionable: false,
            name: String::from("role"),
            permissions: Permissions::empty(),
            position: 1,
            tags: None,
        }
    }

    fn group() -> Group {
        Group {
            application_id: None,
            icon: None,
            id: ChannelId(3),
            kind: ChannelType::Group,
            last_message_id: None,
            last_pin_timestamp: None,
            name: Some(String::from("group")),
            owner_id: UserId(2),
            recipients: vec![user(), user2()],
        }
    }

    fn category() -> CategoryChannel {
        CategoryChannel {
            guild_id: Some(GuildId(1)),
            id: ChannelId(4),
            kind: ChannelType::GuildCategory,
            name: String::from("category"),
            permission_overwrites: Vec::new(),
            position: 1,
        }
    }

    fn text() -> TextChannel {
        TextChannel {
            guild_id: Some(GuildId(1)),
            id: ChannelId(5),
            kind: ChannelType::GuildText,
            last_message_id: None,
            last_pin_timestamp: None,
            name: String::from("text"),
            nsfw: false,
            permission_overwrites: Vec::new(),
            parent_id: Some(ChannelId(4)),
            position: 2,
            rate_limit_per_user: None,
            topic: None,
        }
    }

    fn voice() -> VoiceChannel {
        VoiceChannel {
            bitrate: 96_000,
            guild_id: Some(GuildId(1)),
            id: ChannelId(6),
            kind: ChannelType::GuildVoice,
            name: String::from("voice"),
            permission_overwrites: Vec::new(),
            parent_id: Some(ChannelId(4)),
            position: 3,
            user_limit: Some(3),
        }
    }

    fn private() -> PrivateChannel {
        PrivateChannel {
            id: ChannelId(7),
            last_message_id: None,
            last_pin_timestamp: None,
            kind: ChannelType::Private,
            recipients: vec![user2()],
        }
    }

    fn attachment(id: u64) -> Attachment {
        Attachment {
            filename: format!("filename{}.png", id),
            height: Some(600),
            id: AttachmentId(id),
            proxy_url: String::from("proxy url"),
            size: 123_456_789,
            url: String::from("url"),
            width: Some(400),
        }
    }

    fn messages() -> Vec<Message> {
        let mut messages = Vec::new();

        for i in 100u64..=110 {
            messages.push(Message {
                activity: None,
                application: None,
                attachments: vec![attachment(i + 100)],
                author: user(),
                channel_id: ChannelId(5),
                content: format!("test {}", i),
                edited_timestamp: None,
                embeds: Vec::new(),
                flags: None,
                guild_id: Some(GuildId(1)),
                id: MessageId(i),
                kind: MessageType::Regular,
                member: Some(PartialMember {
                    deaf: false,
                    joined_at: Some(String::from("2012-11-21T10:00:00.40000+00:00")),
                    mute: false,
                    nick: None,
                    roles: Vec::new(),
                }),
                mention_channels: Vec::new(),
                mention_everyone: false,
                mention_roles: Vec::new(),
                mentions: HashMap::new(),
                pinned: false,
                reactions: Vec::new(),
                reference: None,
                referenced_message: None,
                stickers: Vec::new(),
                timestamp: String::from("2012-11-21T12:00:00.40000+00:00"),
                tts: false,
                webhook_id: None,
            });
        }

        messages
    }

    fn guild() -> Guild {
        let mut members = HashMap::new();
        members.insert(UserId(2), member());

        let mut presences = HashMap::new();
        presences.insert(
            UserId(2),
            Presence {
                activities: Vec::new(),
                client_status: ClientStatus {
                    desktop: None,
                    mobile: None,
                    web: None,
                },
                guild_id: GuildId(1),
                status: Status::Offline,
                user: UserOrId::UserId { id: UserId(2) },
            },
        );

        Guild {
            afk_channel_id: None,
            afk_timeout: 0,
            application_id: None,
            approximate_member_count: Some(1),
            approximate_presence_count: Some(1),
            banner: None,
            channels: HashMap::new(),
            default_message_notifications: DefaultMessageNotificationLevel::All,
            description: Some(String::from("a")),
            discovery_splash: None,
            emojis: HashMap::new(),
            explicit_content_filter: ExplicitContentFilter::None,
            features: Vec::new(),
            icon: None,
            id: GuildId(1),
            joined_at: Some(String::from("2012-11-21T10:00:00.40000+00:00")),
            large: false,
            lazy: None,
            max_members: None,
            max_presences: None,
            max_video_channel_users: Some(100),
            member_count: Some(1),
            members,
            mfa_level: MfaLevel::None,
            name: String::from("guild"),
            owner_id: UserId(2),
            owner: Some(true),
            permissions: None,
            preferred_locale: String::from("en-US"),
            premium_subscription_count: Some(0),
            premium_tier: PremiumTier::None,
            presences,
            region: String::from("us-east"),
            roles: HashMap::new(),
            rules_channel_id: None,
            splash: None,
            system_channel_flags: SystemChannelFlags::empty(),
            system_channel_id: None,
            unavailable: false,
            vanity_url_code: None,
            verification_level: VerificationLevel::Low,
            voice_states: HashMap::new(),
            widget_channel_id: None,
            widget_enabled: None,
        }
    }

    fn partial_guild() -> PartialGuild {
        PartialGuild {
            afk_channel_id: None,
            afk_timeout: 0,
            application_id: None,
            banner: None,
            default_message_notifications: DefaultMessageNotificationLevel::All,
            description: None,
            discovery_splash: None,
            emojis: HashMap::new(),
            explicit_content_filter: ExplicitContentFilter::None,
            features: Vec::new(),
            icon: None,
            id: GuildId(1),
            max_members: None,
            max_presences: None,
            member_count: Some(1),
            mfa_level: MfaLevel::None,
            name: String::from("new guild"),
            owner_id: UserId(2),
            owner: Some(true),
            permissions: None,
            preferred_locale: String::from("en-US"),
            premium_subscription_count: Some(0),
            premium_tier: PremiumTier::None,
            region: String::from("us-east"),
            roles: HashMap::new(),
            rules_channel_id: None,
            splash: None,
            system_channel_flags: SystemChannelFlags::empty(),
            system_channel_id: None,
            vanity_url_code: None,
            verification_level: VerificationLevel::Low,
            widget_channel_id: None,
            widget_enabled: None,
        }
    }

    #[tokio::test]
    async fn test_inmemory_cache() {
        let cache = InMemoryCache::new();

        // ready
        let event = Event::Ready(Box::new(ready()));
        let _ = cache.process(&event).await;

        assert_eq!(
            cache.current_user.get().await.unwrap().unwrap(),
            CurrentUserEntity {
                avatar: None,
                bot: true,
                discriminator: String::from("0001"),
                email: None,
                flags: None,
                id: UserId(2),
                mfa_enabled: false,
                name: String::from("user"),
                premium_type: None,
                public_flags: None,
                verified: Some(false),
            }
        );

        // guild create
        let event = Event::GuildCreate(Box::new(GuildCreate(guild())));
        let _ = cache.process(&event).await;

        assert_eq!(
            cache.guilds.get(GuildId(1)).await.unwrap().unwrap(),
            GuildEntity {
                afk_channel_id: None,
                afk_timeout: 0,
                application_id: None,
                approximate_member_count: Some(1),
                approximate_presence_count: Some(1),
                banner: None,
                default_message_notifications: DefaultMessageNotificationLevel::All,
                description: Some(String::from("a")),
                discovery_splash: None,
                explicit_content_filter: ExplicitContentFilter::None,
                features: Vec::new(),
                icon: None,
                id: GuildId(1),
                joined_at: Some(String::from("2012-11-21T10:00:00.40000+00:00")),
                large: false,
                lazy: None,
                max_members: None,
                max_presences: None,
                max_video_channel_users: Some(100),
                member_count: Some(1),
                mfa_level: MfaLevel::None,
                name: String::from("guild"),
                owner_id: UserId(2),
                owner: Some(true),
                permissions: None,
                preferred_locale: String::from("en-US"),
                premium_subscription_count: Some(0),
                premium_tier: PremiumTier::None,
                region: String::from("us-east"),
                rules_channel_id: None,
                splash: None,
                system_channel_flags: SystemChannelFlags::empty(),
                system_channel_id: None,
                unavailable: false,
                vanity_url_code: None,
                verification_level: VerificationLevel::Low,
                widget_channel_id: None,
                widget_enabled: None,
            }
        );

        let mut member_ids = cache.members.list().await.unwrap();

        assert_eq!(
            member_ids.next().await.unwrap().unwrap(),
            MemberEntity {
                deaf: false,
                guild_id: GuildId(1),
                hoisted_role_id: None,
                joined_at: Some(String::from("2012-11-21T10:00:00.40000+00:00")),
                mute: false,
                nick: None,
                premium_since: None,
                role_ids: Vec::new(),
                user_id: UserId(2),
            }
        );

        assert!(member_ids.next().await.is_none());

        assert_eq!(
            cache.guilds.owner(GuildId(1)).await.unwrap().unwrap(),
            UserEntity {
                avatar: None,
                bot: true,
                discriminator: String::from("0001"),
                email: None,
                flags: None,
                id: UserId(2),
                locale: Some(String::from("en-US")),
                mfa_enabled: None,
                name: String::from("user"),
                premium_type: None,
                public_flags: None,
                system: Some(false),
                verified: Some(false),
            }
        );

        // guild update
        let event = Event::GuildUpdate(Box::new(GuildUpdate(partial_guild())));
        let _ = cache.process(&event).await;

        let cached_new_guild = cache.guilds.get(GuildId(1)).await.unwrap().unwrap();
        assert_eq!(cached_new_guild.name, String::from("new guild"));
        assert_eq!(cached_new_guild.description, Some(String::from("a")));

        // channel create
        let event = Event::ChannelCreate(ChannelCreate(Channel::Group(group())));
        let _ = cache.process(&event).await;

        assert_eq!(
            cache.groups.get(ChannelId(3)).await.unwrap().unwrap(),
            GroupEntity {
                application_id: None,
                icon: None,
                id: ChannelId(3),
                kind: ChannelType::Group,
                last_message_id: None,
                last_pin_timestamp: None,
                name: Some(String::from("group")),
                owner_id: UserId(2),
                recipient_ids: vec![UserId(2), UserId(9)],
            }
        );

        let event = Event::ChannelCreate(ChannelCreate(Channel::Guild(GuildChannel::Category(
            category(),
        ))));
        let _ = cache.process(&event).await;

        assert_eq!(
            cache
                .category_channels
                .get(ChannelId(4))
                .await
                .unwrap()
                .unwrap(),
            CategoryChannelEntity {
                guild_id: Some(GuildId(1)),
                id: ChannelId(4),
                kind: ChannelType::GuildCategory,
                name: String::from("category"),
                permission_overwrites: Vec::new(),
                position: 1,
            }
        );

        let event = Event::ChannelCreate(ChannelCreate(Channel::Guild(GuildChannel::Text(text()))));
        let _ = cache.process(&event).await;

        assert_eq!(
            cache
                .text_channels
                .get(ChannelId(5))
                .await
                .unwrap()
                .unwrap(),
            TextChannelEntity {
                guild_id: Some(GuildId(1)),
                id: ChannelId(5),
                kind: ChannelType::GuildText,
                last_message_id: None,
                last_pin_timestamp: None,
                name: String::from("text"),
                nsfw: false,
                permission_overwrites: Vec::new(),
                parent_id: Some(ChannelId(4)),
                position: 2,
                rate_limit_per_user: None,
                topic: None,
            }
        );

        let event =
            Event::ChannelCreate(ChannelCreate(Channel::Guild(GuildChannel::Voice(voice()))));
        let _ = cache.process(&event).await;

        assert_eq!(
            cache
                .voice_channels
                .get(ChannelId(6))
                .await
                .unwrap()
                .unwrap(),
            VoiceChannelEntity {
                bitrate: 96_000,
                guild_id: Some(GuildId(1)),
                id: ChannelId(6),
                kind: ChannelType::GuildVoice,
                name: String::from("voice"),
                permission_overwrites: Vec::new(),
                parent_id: Some(ChannelId(4)),
                position: 3,
                user_limit: Some(3),
            }
        );

        let event = Event::ChannelCreate(ChannelCreate(Channel::Private(private())));
        let _ = cache.process(&event).await;

        assert_eq!(
            cache
                .private_channels
                .get(ChannelId(7))
                .await
                .unwrap()
                .unwrap(),
            PrivateChannelEntity {
                id: ChannelId(7),
                last_message_id: None,
                last_pin_timestamp: None,
                kind: ChannelType::Private,
                recipient_id: Some(UserId(9)),
            }
        );

        assert_eq!(
            cache.users.get(UserId(2)).await,
            Ok(Some(UserEntity::from(user())))
        );

        // member add
        let event = Event::MemberAdd(Box::new(MemberAdd(member2())));
        let _ = cache.process(&event).await;

        assert_eq!(
            cache
                .members
                .get((GuildId(1), UserId(9)))
                .await
                .unwrap()
                .unwrap(),
            MemberEntity {
                deaf: false,
                guild_id: GuildId(1),
                hoisted_role_id: None,
                joined_at: Some(String::from("2012-11-21T11:00:00.40000+00:00")),
                mute: false,
                nick: None,
                premium_since: None,
                role_ids: Vec::new(),
                user_id: UserId(9),
            }
        );

        // channel update
        let event = Event::ChannelUpdate(ChannelUpdate(Channel::Group(Group {
            name: Some(String::from("new group name")),
            ..group()
        })));
        let _ = cache.process(&event).await;

        assert_eq!(
            cache.groups.get(ChannelId(3)).await.unwrap().unwrap().name,
            Some(String::from("new group name"))
        );

        let event = Event::ChannelUpdate(ChannelUpdate(Channel::Guild(GuildChannel::Category(
            CategoryChannel {
                name: String::from("new category name"),
                ..category()
            },
        ))));
        let _ = cache.process(&event).await;

        assert_eq!(
            cache
                .category_channels
                .get(ChannelId(4))
                .await
                .unwrap()
                .unwrap()
                .name,
            String::from("new category name")
        );

        let event = Event::ChannelUpdate(ChannelUpdate(Channel::Guild(GuildChannel::Text(
            TextChannel {
                nsfw: true,
                ..text()
            },
        ))));
        let _ = cache.process(&event).await;

        assert!(
            cache
                .text_channels
                .get(ChannelId(5))
                .await
                .unwrap()
                .unwrap()
                .nsfw
        );

        let event = Event::ChannelCreate(ChannelCreate(Channel::Guild(GuildChannel::Voice(
            VoiceChannel {
                user_limit: Some(4),
                ..voice()
            },
        ))));
        let _ = cache.process(&event).await;

        assert_eq!(
            cache
                .voice_channels
                .get(ChannelId(6))
                .await
                .unwrap()
                .unwrap()
                .user_limit,
            Some(4),
        );

        let event = Event::ChannelCreate(ChannelCreate(Channel::Private(PrivateChannel {
            last_message_id: Some(MessageId(100)),
            ..private()
        })));
        let _ = cache.process(&event).await;

        assert_eq!(
            cache
                .private_channels
                .get(ChannelId(7))
                .await
                .unwrap()
                .unwrap()
                .last_message_id,
            Some(MessageId(100))
        );

        assert_eq!(
            cache.users.get(UserId(2)).await.unwrap().unwrap(),
            UserEntity {
                avatar: None,
                bot: true,
                discriminator: String::from("0001"),
                email: None,
                flags: None,
                id: UserId(2),
                locale: Some(String::from("en-US")),
                mfa_enabled: None,
                name: String::from("user"),
                premium_type: None,
                public_flags: None,
                system: Some(false),
                verified: Some(false),
            }
        );

        // member update
        let member3 = MemberUpdate {
            guild_id: GuildId(1),
            joined_at: String::from("2012-11-21T11:00:00.40000+00:00"),
            nick: None,
            premium_since: None,
            roles: Vec::new(),
            user: User {
                name: String::from("new name"),
                ..user2()
            },
        };

        let event = Event::MemberUpdate(Box::new(member3.clone()));
        let _ = cache.process(&event).await;

        assert_eq!(
            cache.users.get(UserId(9)).await.unwrap().unwrap(),
            UserEntity {
                avatar: None,
                bot: true,
                discriminator: String::from("0002"),
                email: None,
                flags: None,
                id: UserId(9),
                locale: Some(String::from("en-US")),
                mfa_enabled: None,
                name: String::from("new name"),
                premium_type: None,
                public_flags: None,
                system: Some(false),
                verified: Some(false),
            }
        );

        // message create
        for message in messages() {
            let event = Event::MessageCreate(Box::new(MessageCreate(message)));
            let _ = cache.process(&event).await;
        }

        assert_eq!(
            cache
                .messages
                .get(MessageId(105))
                .await
                .unwrap()
                .unwrap()
                .content,
            String::from("test 105")
        );

        assert_eq!(
            cache
                .attachments
                .get(AttachmentId(205))
                .await
                .unwrap()
                .unwrap()
                .filename,
            String::from("filename205.png")
        );

        // channel pins update
        let event = Event::ChannelPinsUpdate(ChannelPinsUpdate {
            channel_id: ChannelId(5),
            guild_id: Some(GuildId(1)),
            last_pin_timestamp: Some(String::from("2012-11-21T11:01:00.40000+00:00")),
        });
        let _ = cache.process(&event).await;

        assert_eq!(
            cache
                .text_channels
                .get(ChannelId(5))
                .await
                .unwrap()
                .unwrap()
                .last_pin_timestamp,
            Some(String::from("2012-11-21T11:01:00.40000+00:00"))
        );

        // guild emojis update
        let mut emojis = HashMap::new();
        emojis.insert(EmojiId(200), emoji());

        let event = Event::GuildEmojisUpdate(GuildEmojisUpdate {
            emojis,
            guild_id: GuildId(1),
        });
        let _ = cache.process(&event).await;

        assert_eq!(
            cache.emojis.get(EmojiId(200)).await.unwrap().unwrap().name,
            String::from("emoji")
        );

        // message update
        let event = Event::MessageUpdate(Box::new(MessageUpdate {
            attachments: None,
            author: Some(user()),
            channel_id: ChannelId(5),
            content: Some(String::from("110 new content")),
            edited_timestamp: Some(String::from("2012-11-21T12:01:00.40000+00:00")),
            embeds: None,
            guild_id: Some(GuildId(1)),
            id: MessageId(110),
            kind: None,
            mention_everyone: None,
            mention_roles: None,
            mentions: None,
            pinned: None,
            timestamp: Some(String::from("2012-11-21T12:00:00.40000+00:00")),
            tts: None,
        }));
        let _ = cache.process(&event).await;

        assert_eq!(
            cache
                .messages
                .get(MessageId(110))
                .await
                .unwrap()
                .unwrap()
                .content,
            String::from("110 new content"),
        );

        // message delete
        let event = Event::MessageDelete(MessageDelete {
            channel_id: ChannelId(5),
            guild_id: Some(GuildId(1)),
            id: MessageId(110),
        });
        let _ = cache.process(&event).await;

        assert_eq!(cache.messages.get(MessageId(110)).await.unwrap(), None);

        // message delete bulk
        let event = Event::MessageDeleteBulk(MessageDeleteBulk {
            channel_id: ChannelId(5),
            guild_id: Some(GuildId(1)),
            ids: (100..=109).map(MessageId).collect(),
        });
        let _ = cache.process(&event).await;

        assert_eq!(cache.messages.get(MessageId(105)).await.unwrap(), None);
        assert_eq!(
            cache.attachments.get(AttachmentId(205)).await.unwrap(),
            None
        );

        // voice state update
        let event = Event::VoiceStateUpdate(Box::new(VoiceStateUpdate(voice_state())));
        let _ = cache.process(&event).await;

        assert_eq!(
            cache
                .voice_states
                .get((GuildId(1), UserId(2)))
                .await
                .unwrap()
                .unwrap()
                .session_id,
            String::from("session")
        );

        // channel delete
        let event =
            Event::ChannelDelete(ChannelDelete(Channel::Guild(GuildChannel::Voice(voice()))));
        let _ = cache.process(&event).await;

        assert_eq!(cache.voice_channels.get(ChannelId(6)).await.unwrap(), None);

        // role create
        let event = Event::RoleCreate(RoleCreate {
            guild_id: GuildId(1),
            role: role(),
        });
        let _ = cache.process(&event).await;

        assert_eq!(
            cache.roles.get(RoleId(12)).await.unwrap().unwrap().name,
            String::from("role")
        );

        // role update
        let event = Event::RoleUpdate(RoleUpdate {
            guild_id: GuildId(1),
            role: Role {
                name: String::from("role new name"),
                ..role()
            },
        });
        let _ = cache.process(&event).await;

        assert_eq!(
            cache.roles.get(RoleId(12)).await.unwrap().unwrap().name,
            String::from("role new name")
        );

        // role delete
        let event = Event::RoleDelete(RoleDelete {
            guild_id: GuildId(1),
            role_id: RoleId(12),
        });
        let _ = cache.process(&event).await;

        assert_eq!(cache.roles.get(RoleId(12)).await.unwrap(), None);

        // user update
        let event = Event::UserUpdate(UserUpdate(CurrentUser {
            name: String::from("new user name"),
            ..current_user()
        }));
        let _ = cache.process(&event).await;

        assert_eq!(
            cache.current_user.get().await.unwrap().unwrap().name,
            String::from("new user name")
        );

        // member remove
        let event = Event::MemberRemove(MemberRemove {
            guild_id: GuildId(1),
            user: member3.user.clone(),
        });
        let _ = cache.process(&event).await;

        assert_eq!(
            cache.members.get((GuildId(1), UserId(9))).await.unwrap(),
            None
        );

        // member chunk
        let event = Event::MemberChunk(member_chunk());
        let _ = cache.process(&event).await;

        assert_eq!(
            cache.users.get(UserId(405)).await.unwrap().unwrap().name,
            String::from("user405")
        );

        // presence update
        let event = Event::PresenceUpdate(Box::new(presence_update()));
        let _ = cache.process(&event).await;

        assert_eq!(
            cache.presences.get((GuildId(1), UserId(405))).await.unwrap().unwrap().status,
            Status::Online
        );

        // guild delete
        let event = Event::GuildDelete(Box::new(GuildDelete {
            id: GuildId(1),
            unavailable: true,
        }));
        let _ = cache.process(&event).await;

        assert!(
            cache
                .guilds
                .get(GuildId(1))
                .await
                .unwrap()
                .unwrap()
                .unavailable
        );

        let event = Event::GuildDelete(Box::new(GuildDelete {
            id: GuildId(1),
            unavailable: false,
        }));
        let _ = cache.process(&event).await;

        assert_eq!(cache.guilds.get(GuildId(1)).await.unwrap(), None);
    }
}
