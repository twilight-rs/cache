//! Entities related to and within channels.

pub mod attachment;
pub mod category_channel;
pub mod group;
pub mod message;
pub mod private_channel;
pub mod text_channel;
pub mod voice_channel;

pub use self::{
    attachment::{AttachmentEntity, AttachmentRepository},
    category_channel::{CategoryChannelEntity, CategoryChannelRepository},
    group::{GroupEntity, GroupRepository},
    message::{MessageEntity, MessageRepository},
    private_channel::{PrivateChannelEntity, PrivateChannelRepository},
    text_channel::{TextChannelEntity, TextChannelRepository},
    voice_channel::{VoiceChannelEntity, VoiceChannelRepository},
};

#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(untagged)
)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChannelEntity {
    Group(GroupEntity),
    Guild(GuildChannelEntity),
    Private(PrivateChannelEntity),
}

#[cfg_attr(
    feature = "serde",
    derive(serde::Deserialize, serde::Serialize),
    serde(untagged)
)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GuildChannelEntity {
    Category(CategoryChannelEntity),
    Text(TextChannelEntity),
    Voice(VoiceChannelEntity),
}
