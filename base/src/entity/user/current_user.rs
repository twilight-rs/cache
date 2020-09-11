use crate::{repository::{guild::GuildEntity, ListEntityIdsFuture, ListEntitiesFuture, Repository}, Entity};
use twilight_model::id::{GuildId, UserId};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrentUserEntity {
    pub avatar: Option<String>,
    pub bot: bool,
    pub discriminator: String,
    pub email: Option<String>,
    pub id: UserId,
    pub mfa_enabled: bool,
    pub name: String,
    pub verified: bool,
}

impl Entity for CurrentUserEntity {
    type Id = UserId;

    /// Return the current user's ID.
    fn id(&self) -> Self::Id {
        self.id
    }
}

pub trait CurrentUserRepository<Error: 'static>: Repository<CurrentUserEntity, Error> {
    /// Retrieve a stream of guild IDs associated with the current user.
    fn guild_ids(&self) -> ListEntityIdsFuture<'_, GuildId, Error>;

    /// Retrieve a stream of guilds associated with the current user.
    fn guilds(&self) -> ListEntitiesFuture<'_, GuildEntity, Error>;
}
