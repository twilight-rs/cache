use super::{
    entity::{
        channel::{
            CategoryChannelEntity, GroupEntity, PrivateChannelEntity, TextChannelEntity,
            VoiceChannelEntity,
        },
        guild::MemberEntity,
    },
    Backend, Repository,
};
use futures_util::future::{self, FutureExt, TryFutureExt};
use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use twilight_model::{
    channel::{Channel, GuildChannel},
    gateway::{
        event::Event,
        payload::{ChannelCreate, ChannelDelete, GuildCreate, MemberAdd, MemberChunk},
    },
};

pub trait CacheUpdate<T: Backend> {
    fn process<'a>(
        &'a self,
        cache: &'a Cache<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>>;
}

pub struct ProcessFuture<'a, T: Backend> {
    inner: Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>>,
}

impl<T: Backend> Future for ProcessFuture<'_, T> {
    type Output = Result<(), T::Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.inner.poll_unpin(cx)
    }
}

/// The cache, a container over a backend that allows you to retrieve and work
/// with entities.
#[derive(Clone, Debug, Default)]
pub struct Cache<T: Backend> {
    backend: Arc<T>,
    /// Repository for working with attachments.
    pub attachments: T::AttachmentRepository,
    /// Repository for working with category channels.
    pub category_channels: T::CategoryChannelRepository,
    /// Repository for working with the current user.
    pub current_user: T::CurrentUserRepository,
    /// Repository for working with emojis.
    pub emojis: T::EmojiRepository,
    /// Repository for working with groups.
    pub groups: T::GroupRepository,
    /// Repository for working with guilds.
    pub guilds: T::GuildRepository,
    /// Repository for working with members.
    pub members: T::MemberRepository,
    /// Repository for working with messages.
    pub messages: T::MessageRepository,
    /// Repository for working with presences.
    pub presences: T::PresenceRepository,
    /// Repository for working with private channels.
    pub private_channels: T::PrivateChannelRepository,
    /// Repository for working with roles.
    pub roles: T::RoleRepository,
    /// Repository for working with text channels.
    pub text_channels: T::TextChannelRepository,
    /// Repository for working with users.
    pub users: T::UserRepository,
    /// Repository for working with users.
    pub voice_channels: T::VoiceChannelRepository,
    /// Repository for working with voice state.
    pub voice_states: T::VoiceStateRepository,
}

impl<T: Backend + Default> Cache<T> {
    /// Create a new cache with a default instance of the backend.
    pub fn new() -> Self {
        Self::with_backend(T::default())
    }
}

impl<T: Backend> Cache<T> {
    /// Create a new cache with a provided instance of the backend.
    pub fn with_backend(backend: impl Into<Arc<T>>) -> Self {
        let backend = backend.into();
        let attachments = backend.attachments();
        let category_channels = backend.category_channels();
        let current_user = backend.current_user();
        let emojis = backend.emojis();
        let groups = backend.groups();
        let guilds = backend.guilds();
        let members = backend.members();
        let messages = backend.messages();
        let presences = backend.presences();
        let private_channels = backend.private_channels();
        let roles = backend.roles();
        let text_channels = backend.text_channels();
        let users = backend.users();
        let voice_channels = backend.voice_channels();
        let voice_states = backend.voice_states();

        Self {
            attachments,
            backend,
            category_channels,
            current_user,
            emojis,
            groups,
            guilds,
            members,
            messages,
            presences,
            private_channels,
            roles,
            text_channels,
            users,
            voice_channels,
            voice_states,
        }
    }

    /// Return an immutable reference to the backend.
    pub fn backend(&self) -> &Arc<T> {
        &self.backend
    }

    /// Update the cache with an event.
    ///
    /// # Examples
    ///
    /// Update the cache with a `RoleDelete` event, which will use the backend's
    /// role repository to delete the role from the datastore:
    ///
    /// ```no_run
    /// use twilight_cache::Cache;
    /// use twilight_cache_inmemory::{InMemoryBackend, Repository};
    /// use twilight_model::{
    ///     gateway::{event::Event, payload::RoleDelete},
    ///     id::{GuildId, RoleId},
    /// };
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let event = Event::RoleDelete(RoleDelete {
    ///     guild_id: GuildId(123),
    ///     role_id: RoleId(456),
    /// });
    ///
    /// let cache: Cache<InMemoryBackend> = Cache::new();
    ///
    /// // And now update the cache with the event:
    /// cache.process(&event).await?;
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a backend error if a backend repository operation errors.
    pub fn process<'a>(&'a self, event: &'a Event) -> ProcessFuture<'a, T> {
        ProcessFuture {
            inner: event.process(self),
        }
    }
}

impl<T: Backend> CacheUpdate<T> for Event {
    fn process<'a>(
        &'a self,
        cache: &'a Cache<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>> {
        match self {
            Event::BanAdd(_) => future::ok(()).boxed(),
            Event::BanRemove(_) => future::ok(()).boxed(),
            Event::ChannelCreate(event) => event.process(cache),
            Event::ChannelDelete(event) => event.process(cache),
            // Ignore non-dispatch gateway events.
            Event::GatewayHeartbeat(_) => future::ok(()).boxed(),
            Event::GatewayHeartbeatAck => future::ok(()).boxed(),
            Event::GatewayHello(_) => future::ok(()).boxed(),
            Event::GatewayInvalidateSession(_) => future::ok(()).boxed(),
            Event::GatewayReconnect => future::ok(()).boxed(),
            Event::GiftCodeUpdate => future::ok(()).boxed(),
            Event::InviteCreate(_) => future::ok(()).boxed(),
            Event::InviteDelete(_) => future::ok(()).boxed(),
            Event::MemberAdd(event) => event.process(cache),
            Event::MemberChunk(event) => event.process(cache),
            Event::Ready(_) => todo!(),
            Event::Resumed => future::ok(()).boxed(),
            // Ignore shard events.
            Event::ShardConnected(_) => future::ok(()).boxed(),
            Event::ShardConnecting(_) => future::ok(()).boxed(),
            Event::ShardDisconnected(_) => future::ok(()).boxed(),
            Event::ShardIdentifying(_) => future::ok(()).boxed(),
            Event::ShardPayload(_) => future::ok(()).boxed(),
            Event::ShardReconnecting(_) => future::ok(()).boxed(),
            Event::ShardResuming(_) => future::ok(()).boxed(),
            Event::TypingStart(_) => future::ok(()).boxed(),
            Event::UnavailableGuild(_) => todo!(),
            _ => todo!(),
        }
    }
}

impl<T: Backend> CacheUpdate<T> for ChannelCreate {
    fn process<'a>(
        &'a self,
        cache: &'a Cache<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>> {
        match &self.0 {
            Channel::Group(group) => {
                let entity = GroupEntity::from(group.clone());

                cache.groups.upsert(entity)
            }
            Channel::Guild(GuildChannel::Category(c)) => {
                let entity = CategoryChannelEntity::from(c.clone());

                cache.category_channels.upsert(entity)
            }
            Channel::Guild(GuildChannel::Text(c)) => {
                let entity = TextChannelEntity::from(c.clone());

                cache.text_channels.upsert(entity)
            }
            Channel::Guild(GuildChannel::Voice(c)) => {
                let entity = VoiceChannelEntity::from(c.clone());

                cache.voice_channels.upsert(entity)
            }
            Channel::Private(c) => {
                let entity = PrivateChannelEntity::from(c.clone());

                cache.private_channels.upsert(entity)
            }
        }
    }
}

impl<T: Backend> CacheUpdate<T> for ChannelDelete {
    fn process<'a>(
        &'a self,
        cache: &'a Cache<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>> {
        match &self.0 {
            Channel::Group(group) => cache.groups.remove(group.id),
            Channel::Guild(GuildChannel::Category(c)) => cache.category_channels.remove(c.id),
            Channel::Guild(GuildChannel::Text(c)) => cache.text_channels.remove(c.id),
            Channel::Guild(GuildChannel::Voice(c)) => cache.voice_channels.remove(c.id),
            Channel::Private(c) => cache.private_channels.remove(c.id),
        }
    }
}

impl<T: Backend> CacheUpdate<T> for GuildCreate {
    fn process<'a>(
        &'a self,
        _: &'a Cache<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>> {
        todo!();
    }
}

impl<T: Backend> CacheUpdate<T> for MemberAdd {
    fn process<'a>(
        &'a self,
        cache: &'a Cache<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>> {
        let entity = MemberEntity::from(self.0.clone());

        cache.members.upsert(entity)
    }
}

impl<T: Backend> CacheUpdate<T> for MemberChunk {
    fn process<'a>(
        &'a self,
        cache: &'a Cache<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>> {
        future::try_join_all(self.members.iter().map(|member| {
            let entity = MemberEntity::from(member.clone());

            cache.members.upsert(entity)
        }))
        .map_ok(|_| ())
        .boxed()
    }
}
