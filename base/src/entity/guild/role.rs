use super::GuildEntity;
use crate::{
    repository::{GetEntityFuture, Repository},
    Entity,
};
use twilight_model::{
    guild::Permissions,
    id::{GuildId, RoleId},
};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleEntity {
    pub color: u32,
    pub guild_id: GuildId,
    pub hoist: bool,
    pub id: RoleId,
    pub managed: bool,
    pub mentionable: bool,
    pub name: String,
    pub permissions: Permissions,
    pub position: i64,
}

impl Entity for RoleEntity {
    type Id = RoleId;

    /// Return the role's ID.
    fn id(&self) -> Self::Id {
        self.id
    }
}

pub trait RoleRepository<Error: 'static>: Repository<RoleEntity, Error> {
    /// Retrieve the guild associated with a role.
    fn guild(&self, role_id: RoleId) -> GetEntityFuture<'_, GuildEntity, Error>;
}
