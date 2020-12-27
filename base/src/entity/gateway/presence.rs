use crate::{Backend, Entity, Repository};
use twilight_model::{
    gateway::{
        presence::{Activity, ClientStatus, Presence, Status, UserOrId},
        payload::PresenceUpdate,
    },
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

impl From<Presence> for PresenceEntity {
    fn from(presence: Presence) -> Self {
        let user_id = match presence.user {
            UserOrId::User(user) => user.id,
            UserOrId::UserId { id } => id,
        };

        Self {
            activities: presence.activities,
            client_status: presence.client_status,
            guild_id: presence.guild_id,
            status: presence.status,
            user_id,
        }
    }
}

impl From<PresenceUpdate> for PresenceEntity {
    fn from(mut presence: PresenceUpdate) -> Self {
        let mut activities = Vec::new();

        if let Some(game) = presence.game {
            activities.push(game);
        }

        activities.append(&mut presence.activities);

        let user_id = match presence.user {
            UserOrId::User(user) => user.id,
            UserOrId::UserId { id } => id,
        };

        Self {
            activities,
            client_status: presence.client_status,
            guild_id: presence.guild_id,
            status: presence.status,
            user_id,
        }
    }
}

impl Entity for PresenceEntity {
    type Id = (GuildId, UserId);

    /// Return an ID consisting of a tuple of the guild ID and user ID.
    fn id(&self) -> Self::Id {
        (self.guild_id, self.user_id)
    }
}

pub trait PresenceRepository<B: Backend>: Repository<PresenceEntity, B> {}
