use super::super::{guild::GuildEntity, Entity};
use crate::{
    repository::{ListEntitiesFuture, ListEntityIdsFuture, SingleEntityRepository},
    utils, Backend,
};
use twilight_model::{
    id::{GuildId, UserId},
    user::{CurrentUser, PremiumType, UserFlags},
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
    pub verified: Option<bool>,
}

impl From<CurrentUser> for CurrentUserEntity {
    fn from(current_user: CurrentUser) -> Self {
        Self {
            avatar: current_user.avatar,
            bot: current_user.bot,
            discriminator: current_user.discriminator,
            email: current_user.email,
            flags: current_user.flags,
            id: current_user.id,
            mfa_enabled: current_user.mfa_enabled,
            name: current_user.name,
            premium_type: current_user.premium_type,
            public_flags: current_user.public_flags,
            verified: current_user.verified,
        }
    }
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
