use crate::{Backend, Entity, Repository};
use twilight_model::{
    gateway::presence::{Activity, ClientStatus, Status},
    id::{GuildId, UserId},
};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PresenceEntity {
    pub activities: Vec<Activity>,
    pub client_status: ClientStatus,
    pub guild_id: GuildId,
    pub status: Status,
    pub user_id: UserId,
}

impl Entity for PresenceEntity {
    type Id = (GuildId, UserId);

    /// Return an ID consisting of a tuple of the guild ID and user ID.
    fn id(&self) -> Self::Id {
        (self.guild_id, self.user_id)
    }
}

pub trait PresenceRepository<B: Backend>: Repository<PresenceEntity, B> {}
