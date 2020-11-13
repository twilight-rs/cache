use super::super::{guild::GuildEntity, Entity};
use crate::{
    repository::{ListEntitiesFuture, ListEntityIdsFuture, SingleEntityRepository},
    utils, Backend,
};
use twilight_model::{
    id::{GuildId, UserId},
    user::{PremiumType, UserFlags},
};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrentUserEntity {
    pub avatar: Option<String>,
    pub bot: bool,
    pub discriminator: String,
    pub email: Option<String>,
    pub flags: Option<UserFlags>,
    pub id: UserId,
    pub mfa_enabled: bool,
    pub name: String,
    pub premium_type: Option<PremiumType>,
    pub public_flags: Option<UserFlags>,
    pub verified: bool,
}

impl Entity for CurrentUserEntity {
    type Id = UserId;

    /// Return the current user's ID.
    fn id(&self) -> Self::Id {
        self.id
    }
}

pub trait CurrentUserRepository<B: Backend>: SingleEntityRepository<CurrentUserEntity, B> {
    /// Retrieve a stream of guild IDs associated with the current user.
    fn guild_ids(&self) -> ListEntityIdsFuture<'_, GuildId, B::Error>;

    /// Retrieve a stream of guilds associated with the current user.
    fn guilds(&self) -> ListEntitiesFuture<'_, GuildEntity, B::Error> {
        utils::stream_ids(self.guild_ids(), self.backend().guilds())
    }
}
