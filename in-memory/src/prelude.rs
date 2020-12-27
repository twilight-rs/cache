//! Useful re-exports for working with the in memory cache.

#[doc(no_inline)]
pub use super::{InMemoryBackend, InMemoryBackendError, InMemoryCache};
#[doc(no_inline)]
pub use twilight_cache::{
    entity::{
        channel::{
            attachment::AttachmentRepository as _,
            category_channel::CategoryChannelRepository as _, group::GroupRepository as _,
            message::MessageRepository as _, private_channel::PrivateChannelRepository as _,
            text_channel::TextChannelRepository as _, voice_channel::VoiceChannelRepository as _,
            ChannelEntity, GuildChannelEntity,
        },
        gateway::presence::PresenceRepository as _,
        guild::{
            emoji::EmojiRepository as _, member::MemberRepository as _, role::RoleRepository as _,
            GuildRepository as _,
        },
        user::UserRepository as _,
        voice::VoiceStateRepository as _,
    },
    Backend as _, Cache, Repository as _,
};
