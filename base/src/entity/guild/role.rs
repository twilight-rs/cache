use super::GuildEntity;
use crate::{
    repository::{GetEntityFuture, Repository},
    utils, Backend, Entity,
};
use twilight_model::{
    guild::{Permissions, Role},
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

impl From<(Role, GuildId)> for RoleEntity {
    fn from((role, guild_id): (Role, GuildId)) -> Self {
        Self {
            color: role.color,
            guild_id,
            hoist: role.hoist,
            id: role.id,
            managed: role.managed,
            mentionable: role.mentionable,
            name: role.name,
            permissions: role.permissions,
            position: role.position,
        }
    }
}

impl Entity for RoleEntity {
    type Id = RoleId;

    /// Return the role's ID.
    fn id(&self) -> Self::Id {
        self.id
    }
}

pub trait RoleRepository<B: Backend>: Repository<RoleEntity, B> {
    /// Retrieve the guild associated with a role.
    fn guild(&self, role_id: RoleId) -> GetEntityFuture<'_, GuildEntity, B::Error> {
        utils::relation_map(
            self.backend().roles(),
            self.backend().guilds(),
            role_id,
            |role| role.guild_id,
        )
    }
}
