use super::{
    entity::{
        channel::{
            AttachmentEntity, CategoryChannelEntity, GroupEntity, GuildChannelEntity,
            MessageEntity, MessageRepository, PrivateChannelEntity, TextChannelEntity,
            VoiceChannelEntity,
        },
        gateway::PresenceEntity,
        guild::{EmojiEntity, GuildEntity, GuildRepository, MemberEntity, RoleEntity},
        user::CurrentUserEntity,
        voice::VoiceStateEntity,
    },
    repository::SingleEntityRepository,
    Backend, Repository,
};
use futures_util::{
    future::{self, FutureExt, TryFutureExt},
    stream::StreamExt,
};
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
        payload::{
            ChannelCreate, ChannelDelete, ChannelPinsUpdate, ChannelUpdate, GuildCreate,
            GuildDelete, GuildEmojisUpdate, GuildUpdate, MemberAdd, MemberChunk, MemberRemove,
            MemberUpdate, MessageCreate, MessageDelete, MessageDeleteBulk, MessageUpdate,
            PresenceUpdate, Ready, RoleCreate, RoleDelete, RoleUpdate, UserUpdate,
            VoiceStateUpdate,
        },
    },
};

pub trait CacheUpdate<T: Backend + Send + Sync> {
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

impl<T: Backend + Send + Sync + Default> Cache<T> {
    /// Create a new cache with a default instance of the backend.
    pub fn new() -> Self {
        Self::with_backend(T::default())
    }
}

impl<T: Backend + Send + Sync> Cache<T> {
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

impl<T: Backend + Send + Sync> CacheUpdate<T> for Event {
    fn process<'a>(
        &'a self,
        cache: &'a Cache<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>> {
        match self {
            Event::BanAdd(_) => future::ok(()).boxed(),
            Event::BanRemove(_) => future::ok(()).boxed(),
            Event::ChannelCreate(event) => event.process(cache),
            Event::ChannelDelete(event) => event.process(cache),
            Event::ChannelPinsUpdate(event) => event.process(cache),
            Event::ChannelUpdate(event) => event.process(cache),
            // Ignore non-dispatch gateway events.
            Event::GatewayHeartbeat(_) => future::ok(()).boxed(),
            Event::GatewayHeartbeatAck => future::ok(()).boxed(),
            Event::GatewayHello(_) => future::ok(()).boxed(),
            Event::GatewayInvalidateSession(_) => future::ok(()).boxed(),
            Event::GatewayReconnect => future::ok(()).boxed(),
            Event::GiftCodeUpdate => future::ok(()).boxed(),
            Event::GuildCreate(event) => event.process(cache),
            Event::GuildDelete(event) => event.process(cache),
            Event::GuildEmojisUpdate(event) => event.process(cache),
            Event::GuildIntegrationsUpdate(_) => future::ok(()).boxed(),
            Event::GuildUpdate(event) => event.process(cache),
            Event::InviteCreate(_) => future::ok(()).boxed(),
            Event::InviteDelete(_) => future::ok(()).boxed(),
            Event::MemberAdd(event) => event.process(cache),
            Event::MemberRemove(event) => event.process(cache),
            Event::MemberUpdate(event) => event.process(cache),
            Event::MemberChunk(event) => event.process(cache),
            Event::MessageCreate(event) => event.process(cache),
            Event::MessageDelete(event) => event.process(cache),
            Event::MessageDeleteBulk(event) => event.process(cache),
            Event::MessageUpdate(event) => event.process(cache),
            Event::PresenceUpdate(event) => event.process(cache),
            Event::PresencesReplace => future::ok(()).boxed(),
            Event::ReactionAdd(_) => future::ok(()).boxed(),
            Event::ReactionRemove(_) => future::ok(()).boxed(),
            Event::ReactionRemoveAll(_) => future::ok(()).boxed(),
            Event::ReactionRemoveEmoji(_) => future::ok(()).boxed(),
            Event::Ready(event) => event.process(cache),
            Event::Resumed => future::ok(()).boxed(),
            Event::RoleCreate(event) => event.process(cache),
            Event::RoleDelete(event) => event.process(cache),
            Event::RoleUpdate(event) => event.process(cache),
            // Ignore shard events.
            Event::ShardConnected(_) => future::ok(()).boxed(),
            Event::ShardConnecting(_) => future::ok(()).boxed(),
            Event::ShardDisconnected(_) => future::ok(()).boxed(),
            Event::ShardIdentifying(_) => future::ok(()).boxed(),
            Event::ShardPayload(_) => future::ok(()).boxed(),
            Event::ShardReconnecting(_) => future::ok(()).boxed(),
            Event::ShardResuming(_) => future::ok(()).boxed(),
            Event::TypingStart(_) => future::ok(()).boxed(),
            Event::UnavailableGuild(_) => future::ok(()).boxed(),
            Event::UserUpdate(event) => event.process(cache),
            Event::VoiceServerUpdate(_) => future::ok(()).boxed(),
            Event::VoiceStateUpdate(event) => event.process(cache),
            Event::WebhooksUpdate(_) => future::ok(()).boxed(),
        }
    }
}

impl<T: Backend + Send + Sync> CacheUpdate<T> for ChannelCreate {
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

impl<T: Backend + Send + Sync> CacheUpdate<T> for ChannelDelete {
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

impl<T: Backend + Send + Sync> CacheUpdate<T> for ChannelPinsUpdate {
    fn process<'a>(
        &'a self,
        cache: &'a Cache<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>> {
        Box::pin(async move {
            if let Some(group) = cache.groups.get(self.channel_id).await? {
                return cache
                    .groups
                    .upsert(GroupEntity {
                        last_pin_timestamp: self.last_pin_timestamp.clone(),
                        ..group
                    })
                    .await;
            }

            if let Some(text_channel) = cache.text_channels.get(self.channel_id).await? {
                return cache
                    .text_channels
                    .upsert(TextChannelEntity {
                        last_pin_timestamp: self.last_pin_timestamp.clone(),
                        ..text_channel
                    })
                    .await;
            }

            if let Some(private_channel) = cache.private_channels.get(self.channel_id).await? {
                return cache
                    .private_channels
                    .upsert(PrivateChannelEntity {
                        last_pin_timestamp: self.last_pin_timestamp.clone(),
                        ..private_channel
                    })
                    .await;
            }

            Ok(())
        })
    }
}

impl<T: Backend + Send + Sync> CacheUpdate<T> for ChannelUpdate {
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

impl<T: Backend + Send + Sync> CacheUpdate<T> for GuildCreate {
    fn process<'a>(
        &'a self,
        cache: &'a Cache<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>> {
        let mut futures = Vec::new();

        for channel in self.channels.values() {
            match channel {
                GuildChannel::Category(c) => {
                    let entity = CategoryChannelEntity::from(c.clone());
                    futures.push(cache.category_channels.upsert(entity));
                }
                GuildChannel::Text(c) => {
                    let entity = TextChannelEntity::from(c.clone());
                    futures.push(cache.text_channels.upsert(entity));
                }
                GuildChannel::Voice(c) => {
                    let entity = VoiceChannelEntity::from(c.clone());
                    futures.push(cache.voice_channels.upsert(entity));
                }
            }
        }

        for emoji in self.emojis.values() {
            let entity = EmojiEntity::from((self.id, emoji.clone()));
            futures.push(cache.emojis.upsert(entity));
        }

        for member in self.members.values() {
            let entity = MemberEntity::from(member.clone());
            futures.push(cache.members.upsert(entity));
        }

        for presence in self.presences.values() {
            let entity = PresenceEntity::from(presence.clone());
            futures.push(cache.presences.upsert(entity));
        }

        for role in self.roles.values() {
            let entity = RoleEntity::from((role.clone(), self.id));
            futures.push(cache.roles.upsert(entity));
        }

        for voice_state in self.voice_states.values() {
            let entity = VoiceStateEntity::from((voice_state.clone(), self.id));
            futures.push(cache.voice_states.upsert(entity));
        }

        let entity = GuildEntity::from(self.0.clone());
        futures.push(cache.guilds.upsert(entity));

        future::try_join_all(futures).map_ok(|_| ()).boxed()
    }
}

impl<T: Backend + Send + Sync> CacheUpdate<T> for GuildDelete {
    fn process<'a>(
        &'a self,
        cache: &'a Cache<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>> {
        Box::pin(async move {
            if self.unavailable {
                if let Some(guild) = cache.guilds.get(self.id).await? {
                    return cache
                        .guilds
                        .upsert(GuildEntity {
                            unavailable: self.unavailable,
                            ..guild
                        })
                        .await;
                }
            }

            let mut futures = Vec::new();

            let mut channels = cache.guilds.channels(self.id).await?;
            while let Some(Ok(c)) = channels.next().await {
                match c {
                    GuildChannelEntity::Category(c) => {
                        futures.push(cache.category_channels.remove(c.id));
                    }
                    GuildChannelEntity::Text(c) => futures.push(cache.text_channels.remove(c.id)),
                    GuildChannelEntity::Voice(c) => futures.push(cache.voice_channels.remove(c.id)),
                }
            }

            let mut emojis = cache.guilds.emoji_ids(self.id).await?;
            while let Some(Ok(id)) = emojis.next().await {
                futures.push(cache.emojis.remove(id));
            }

            let mut members = cache.guilds.member_ids(self.id).await?;
            while let Some(Ok(id)) = members.next().await {
                futures.push(cache.members.remove((self.id, id)));
            }

            let mut presences = cache.guilds.presence_ids(self.id).await?;
            while let Some(Ok(id)) = presences.next().await {
                futures.push(cache.presences.remove((self.id, id)))
            }

            let mut roles = cache.guilds.role_ids(self.id).await?;
            while let Some(Ok(id)) = roles.next().await {
                futures.push(cache.roles.remove(id))
            }

            let mut voice_states = cache.guilds.voice_state_ids(self.id).await?;
            while let Some(Ok(id)) = voice_states.next().await {
                futures.push(cache.voice_states.remove((self.id, id)))
            }

            future::try_join_all(futures).map_ok(|_| ()).await
        })
    }
}

impl<T: Backend + Send + Sync> CacheUpdate<T> for GuildEmojisUpdate {
    fn process<'a>(
        &'a self,
        cache: &'a Cache<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>> {
        let mut futures = Vec::new();

        for emoji in self.emojis.values() {
            let entity = EmojiEntity::from((self.guild_id, emoji.clone()));
            futures.push(cache.emojis.upsert(entity));
        }

        future::try_join_all(futures).map_ok(|_| ()).boxed()
    }
}

impl<T: Backend + Send + Sync> CacheUpdate<T> for GuildUpdate {
    fn process<'a>(
        &'a self,
        cache: &'a Cache<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>> {
        Box::pin(async move {
            let guild = cache.guilds.get(self.id).await?.unwrap();
            let entity = GuildEntity::from((self.0.clone(), guild));

            cache.guilds.upsert(entity).await
        })
    }
}

impl<T: Backend + Send + Sync> CacheUpdate<T> for MemberAdd {
    fn process<'a>(
        &'a self,
        cache: &'a Cache<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>> {
        let entity = MemberEntity::from(self.0.clone());

        cache.members.upsert(entity)
    }
}

impl<T: Backend + Send + Sync> CacheUpdate<T> for MemberRemove {
    fn process<'a>(
        &'a self,
        cache: &'a Cache<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>> {
        cache.members.remove((self.guild_id, self.user.id))
    }
}

impl<T: Backend + Send + Sync> CacheUpdate<T> for MemberUpdate {
    fn process<'a>(
        &'a self,
        cache: &'a Cache<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>> {
        Box::pin(async move {
            let member = cache
                .members
                .get((self.guild_id, self.user.id))
                .await?
                .unwrap();

            let entity = MemberEntity::from((self.clone(), member));

            cache.members.upsert(entity).await
        })
    }
}

impl<T: Backend + Send + Sync> CacheUpdate<T> for MemberChunk {
    fn process<'a>(
        &'a self,
        cache: &'a Cache<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>> {
        future::try_join_all(self.members.values().map(|member| {
            let entity = MemberEntity::from(member.clone());

            cache.members.upsert(entity)
        }))
        .map_ok(|_| ())
        .boxed()
    }
}

impl<T: Backend + Send + Sync> CacheUpdate<T> for MessageCreate {
    fn process<'a>(
        &'a self,
        cache: &'a Cache<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>> {
        Box::pin(async move {
            let mut futures = Vec::new();

            if let Some(group) = cache.groups.get(self.channel_id).await? {
                futures.push(cache.groups.upsert(GroupEntity {
                    last_message_id: Some(self.id),
                    ..group
                }));
            }

            if let Some(text_channel) = cache.text_channels.get(self.channel_id).await? {
                futures.push(cache.text_channels.upsert(TextChannelEntity {
                    last_message_id: Some(self.id),
                    ..text_channel
                }));
            }

            if let Some(private_channel) = cache.private_channels.get(self.channel_id).await? {
                futures.push(cache.private_channels.upsert(PrivateChannelEntity {
                    last_message_id: Some(self.id),
                    ..private_channel
                }));
            }

            for attachment in self.0.attachments.iter().cloned() {
                let entity = AttachmentEntity::from((self.id, attachment));
                futures.push(cache.attachments.upsert(entity));
            }

            let entity = MessageEntity::from(self.0.clone());
            futures.push(cache.messages.upsert(entity));

            future::try_join_all(futures).map_ok(|_| ()).await
        })
    }
}

impl<T: Backend + Send + Sync> CacheUpdate<T> for MessageDelete {
    fn process<'a>(
        &'a self,
        cache: &'a Cache<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>> {
        Box::pin(async move {
            let mut futures = Vec::new();

            let mut attachments = cache.messages.attachments(self.id).await?;
            while let Some(Ok(attachment)) = attachments.next().await {
                futures.push(cache.attachments.remove(attachment.id));
            }

            futures.push(cache.messages.remove(self.id));

            future::try_join_all(futures).map_ok(|_| ()).await
        })
    }
}

impl<T: Backend + Send + Sync> CacheUpdate<T> for MessageDeleteBulk {
    fn process<'a>(
        &'a self,
        cache: &'a Cache<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>> {
        Box::pin(async move {
            let mut futures = Vec::new();

            for id in self.ids.iter().copied() {
                let mut attachments = cache.messages.attachments(id).await?;
                while let Some(Ok(attachment)) = attachments.next().await {
                    futures.push(cache.attachments.remove(attachment.id));
                }

                futures.push(cache.messages.remove(id));
            }

            future::try_join_all(futures).map_ok(|_| ()).await
        })
    }
}

impl<T: Backend + Send + Sync> CacheUpdate<T> for MessageUpdate {
    fn process<'a>(
        &'a self,
        cache: &'a Cache<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>> {
        Box::pin(async move {
            let mut futures = Vec::new();

            if let Some(attachments) = &self.attachments {
                for attachment in attachments.iter().cloned() {
                    let entity = AttachmentEntity::from((self.id, attachment));
                    futures.push(cache.attachments.upsert(entity));
                }
            }

            let message = cache.messages.get(self.id).await?.unwrap();
            let entity = MessageEntity::from((self.clone(), message));
            futures.push(cache.messages.upsert(entity));

            future::try_join_all(futures).map_ok(|_| ()).await
        })
    }
}

impl<T: Backend + Send + Sync> CacheUpdate<T> for PresenceUpdate {
    fn process<'a>(
        &'a self,
        cache: &'a Cache<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>> {
        let entity = PresenceEntity::from(self.clone());

        cache.presences.upsert(entity)
    }
}

impl<T: Backend + Send + Sync> CacheUpdate<T> for Ready {
    fn process<'a>(
        &'a self,
        cache: &'a Cache<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>> {
        let entity = CurrentUserEntity::from(self.user.clone());

        cache.current_user.upsert(entity)
    }
}

impl<T: Backend + Send + Sync> CacheUpdate<T> for RoleCreate {
    fn process<'a>(
        &'a self,
        cache: &'a Cache<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>> {
        let entity = RoleEntity::from((self.role.clone(), self.guild_id));

        cache.roles.upsert(entity)
    }
}

impl<T: Backend + Send + Sync> CacheUpdate<T> for RoleDelete {
    fn process<'a>(
        &'a self,
        cache: &'a Cache<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>> {
        cache.roles.remove(self.role_id)
    }
}

impl<T: Backend + Send + Sync> CacheUpdate<T> for RoleUpdate {
    fn process<'a>(
        &'a self,
        cache: &'a Cache<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>> {
        let entity = RoleEntity::from((self.role.clone(), self.guild_id));

        cache.roles.upsert(entity)
    }
}

impl<T: Backend + Send + Sync> CacheUpdate<T> for UserUpdate {
    fn process<'a>(
        &'a self,
        cache: &'a Cache<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>> {
        let entity = CurrentUserEntity::from(self.0.clone());

        cache.current_user.upsert(entity)
    }
}

impl<T: Backend + Send + Sync> CacheUpdate<T> for VoiceStateUpdate {
    fn process<'a>(
        &'a self,
        cache: &'a Cache<T>,
    ) -> Pin<Box<dyn Future<Output = Result<(), T::Error>> + Send + 'a>> {
        self.0.guild_id.map_or_else(
            || future::ok(()).boxed(),
            |guild_id| {
                let entity = VoiceStateEntity::from((self.0.clone(), guild_id));

                cache.voice_states.upsert(entity)
            },
        )
    }
}
