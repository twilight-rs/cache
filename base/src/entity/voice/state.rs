use super::super::channel::VoiceChannelEntity;
use crate::{
    repository::{GetEntityFuture, Repository},
    utils, Backend, Entity,
};
use twilight_model::id::{ChannelId, GuildId, UserId};

#[allow(clippy::struct_excessive_bools)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VoiceStateEntity {
    pub channel_id: Option<ChannelId>,
    pub deaf: bool,
    pub guild_id: GuildId,
    pub mute: bool,
    pub self_deaf: bool,
    pub self_mute: bool,
    pub self_stream: bool,
    pub session_id: String,
    pub suppress: bool,
    pub user_id: UserId,
}

impl Entity for VoiceStateEntity {
    type Id = (GuildId, UserId);

    /// Return an ID consisting of a tuple of the guild ID and user ID.
    fn id(&self) -> Self::Id {
        (self.guild_id, self.user_id)
    }
}

pub trait VoiceStateRepository<B: Backend>: Repository<VoiceStateEntity, B> {
    /// Retrieve the channel associated with a webhook.
    ///
    /// **Backend implementations**: if a voice state's channel ID is `None` or
    /// the channel does not exist in the cache then an `Ok(None)` should be
    /// returned.
    fn channel(
        &self,
        guild_id: GuildId,
        user_id: UserId,
    ) -> GetEntityFuture<'_, VoiceChannelEntity, B::Error> {
        utils::relation_and_then(
            self.backend().voice_states(),
            self.backend().voice_channels(),
            (guild_id, user_id),
            |state| state.channel_id,
        )
    }
}
