//! Entities related to users.

pub mod current_user;

pub use self::current_user::{CurrentUserEntity, CurrentUserRepository};

use crate::{
    entity::{guild::GuildEntity, Entity},
    repository::{ListEntitiesFuture, ListEntityIdsFuture, Repository},
    utils, Backend,
};
use twilight_model::{
    id::{GuildId, UserId},
    user::{PremiumType, UserFlags},
};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct UserEntity {
    pub avatar: Option<String>,
    pub bot: bool,
    pub discriminator: String,
    pub email: Option<String>,
    pub flags: Option<UserFlags>,
    pub id: UserId,
    pub locale: Option<String>,
    pub mfa_enabled: Option<bool>,
    pub name: String,
    pub premium_type: Option<PremiumType>,
    pub public_flags: Option<UserFlags>,
    pub system: Option<bool>,
    pub verified: Option<bool>,
}

impl Entity for UserEntity {
    type Id = UserId;

    /// Return the user's ID.
    fn id(&self) -> Self::Id {
        self.id
    }
}

pub trait UserRepository<B: Backend>: Repository<UserEntity, B> {
    /// Retrieve a stream of guild IDs associated with a user.
    fn guild_ids(&self, user_id: UserId) -> ListEntityIdsFuture<'_, GuildId, B::Error>;

    /// Retrieve a stream of guilds associated with a user.
    fn guilds(&self, user_id: UserId) -> ListEntitiesFuture<'_, GuildEntity, B::Error> {
        utils::stream_ids(self.guild_ids(user_id), self.backend().guilds())
    }
}
