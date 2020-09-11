//! Repository implementations for working with the in memory cache.

mod attachment;
mod category_channel;
mod emoji;
mod group;
mod guild;
mod member;
mod message;
mod presence;
mod private_channel;
mod role;
mod text_channel;
mod user;
mod voice_channel;
mod voice_state;

pub use self::{
    attachment::InMemoryAttachmentRepository, category_channel::InMemoryCategoryChannelRepository,
    emoji::InMemoryEmojiRepository, group::InMemoryGroupRepository, guild::InMemoryGuildRepository,
    member::InMemoryMemberRepository, message::InMemoryMessageRepository,
    presence::InMemoryPresenceRepository, private_channel::InMemoryPrivateChannelRepository,
    role::InMemoryRoleRepository, text_channel::InMemoryTextChannelRepository,
    user::InMemoryUserRepository, voice_channel::InMemoryVoiceChannelRepository,
    voice_state::InMemoryVoiceStateRepository,
};
