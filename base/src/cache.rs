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
use futures_util::future;
use std::sync::Arc;
use twilight_model::{
    channel::{Channel, GuildChannel},
    gateway::event::Event,
};

/// The cache, a container over a backend that allows you to retrieve and work
/// with entities.
#[derive(Clone, Debug, Default)]
pub struct Cache<T: Backend> {
    backend: Arc<T>,
    /// Repository for working with attachments.
    pub attachments: T::AttachmentRepository,
    /// Repository for working with category channels.
    pub category_channels: T::CategoryChannelRepository,
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
    /// use rarity_cache::Cache;
    /// use rarity_cache_inmemory::{InMemoryBackend, Repository};
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
    /// cache.update(&event).await?;
    /// # Ok(()) }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a backend error if a backend repository operation errors.
    pub async fn update(&self, event: &Event) -> Result<(), T::Error> {
        match event {
            Event::BanAdd(_) => {}
            Event::BanRemove(_) => {}
            Event::ChannelCreate(channel) => match &channel.0 {
                Channel::Group(group) => {
                    let entity = GroupEntity::from(group.clone());

                    self.groups.upsert(entity).await?;
                }
                Channel::Guild(GuildChannel::Category(c)) => {
                    let entity = CategoryChannelEntity::from(c.clone());

                    self.category_channels.upsert(entity).await?;
                }
                Channel::Guild(GuildChannel::Text(c)) => {
                    let entity = TextChannelEntity::from(c.clone());

                    self.text_channels.upsert(entity).await?;
                }
                Channel::Guild(GuildChannel::Voice(c)) => {
                    let entity = VoiceChannelEntity::from(c.clone());

                    self.voice_channels.upsert(entity).await?;
                }
                Channel::Private(c) => {
                    let entity = PrivateChannelEntity::from(c.clone());

                    self.private_channels.upsert(entity).await?;
                }
            },
            Event::ChannelDelete(channel) => match &channel.0 {
                Channel::Group(group) => {
                    self.groups.remove(group.id).await?;
                }
                Channel::Guild(GuildChannel::Category(c)) => {
                    self.category_channels.remove(c.id).await?;
                }
                Channel::Guild(GuildChannel::Text(c)) => {
                    self.text_channels.remove(c.id).await?;
                }
                Channel::Guild(GuildChannel::Voice(c)) => {
                    self.voice_channels.remove(c.id).await?;
                }
                Channel::Private(c) => {
                    self.private_channels.remove(c.id).await?;
                }
            },
            //     Event::ChannelPinsUpdate(pins) => {
            //         let id = EntityId::Primary(pins.channel_id.0);
            //         let record = Record::new(id, ResourceType::Channel);
            //         let field = Field::Message(MessageField::LastPinTimestamp(pins.last_pin_timestamp.as_deref()));

            //         self.backend.upsert_field(FieldUpdate::new(record, field)).await?;
            //     },
            //     Event::ChannelUpdate(channel) => {
            //         self.backend.upsert(ResourceUpsert::Channel(ChannelUpsert {
            //             inner: channel,
            //         })).await?;
            //     },
            // Ignore non-dispatch gateway events.
            Event::GatewayHeartbeat(_) => {}
            Event::GatewayHeartbeatAck => {}
            Event::GatewayHello(_) => {}
            Event::GatewayInvalidateSession(_) => {}
            Event::GatewayReconnect => {}
            Event::GiftCodeUpdate => {}
            Event::InviteCreate(_) => {}
            Event::InviteDelete(_) => {}
            Event::MemberAdd(member) => {
                let entity = MemberEntity::from(member.0.clone());

                self.members.upsert(entity);
            }
            Event::MemberChunk(chunk) => {
                future::try_join_all(chunk.members.values().map(|member| {
                    let entity = MemberEntity::from(member.clone());

                    self.members.upsert(entity)
                }))
                .await?;
            }
            // Event::MemberUpdate(member) => {
            //     let entity = MemberEntity::from(&member.0);

            //     self.members.upsert(entity);
            // },
            //     // Event::MessageCreate(message) => {
            //     //     self.backend.upsert(ResourceUpsert::Message(MessageUpsert {
            //     //         inner: message,
            //     //     })).await?;
            //     // },
            //     Event::MessageDelete(message) => {
            //         let id = EntityId::Primary(message.id.0);
            //         let record = Record::new(id, ResourceType::Message);

            //         self.backend.delete(record).await?;
            //     },
            //     Event::MessageDeleteBulk(bulk) => {
            //         self.backend.delete_bulk(bulk.ids.iter().map(|id| {
            //             Record::new(EntityId::Primary(id.0), ResourceType::Message)
            //         })).await?;
            //     },
            //     Event::MessageUpdate(message) => {
            //         self.backend.upsert(ResourceUpsert::Message(MessageUpsert {
            //             inner: message,
            //         })).await?;
            //     },
            Event::Ready(_) => {}
            Event::Resumed => {}
            // Ignore shard events.
            Event::ShardConnected(_) => {}
            Event::ShardConnecting(_) => {}
            Event::ShardDisconnected(_) => {}
            Event::ShardIdentifying(_) => {}
            Event::ShardPayload(_) => {}
            Event::ShardReconnecting(_) => {}
            Event::ShardResuming(_) => {}
            Event::TypingStart(_) => {}
            Event::UnavailableGuild(_) => {}
            //     Event::UserUpdate(current_user) => {
            //         self.backend.upsert(ResourceUpsert::UserCurrent(UserCurrentUpsert {
            //             inner: current_user,
            //         })).await?;
            //     },
            //     Event::VoiceServerUpdate(_) => {},
            //     Event::VoiceStateUpdate(voice_state) => {
            //         self.backend.upsert(ResourceUpsert::VoiceState(VoiceStateUpsert {
            //             inner: &voice_state.0,
            //         })).await?;
            //     }
            //     Event::WebhooksUpdate(_) => {},
            _ => {}
        }

        Ok(())
    }
}
