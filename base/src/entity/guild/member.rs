use super::role::RoleEntity;
use crate::{
    repository::{GetEntityFuture, ListEntitiesFuture, Repository},
    Entity,
};
use twilight_model::{
    guild::Member,
    id::{GuildId, RoleId, UserId},
};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemberEntity {
    pub deaf: bool,
    pub guild_id: GuildId,
    pub hoisted_role: Option<RoleId>,
    pub joined_at: Option<String>,
    pub mute: bool,
    pub nick: Option<String>,
    pub premium_since: Option<String>,
    pub role_ids: Vec<RoleId>,
    pub user_id: UserId,
}

impl From<Member> for MemberEntity {
    fn from(member: Member) -> Self {
        Self {
            deaf: member.deaf,
            guild_id: member.guild_id,
            hoisted_role: member.hoisted_role,
            joined_at: member.joined_at,
            mute: member.mute,
            nick: member.nick,
            premium_since: member.premium_since,
            role_ids: member.roles,
            user_id: member.user.id,
        }
    }
}

impl Entity for MemberEntity {
    type Id = (GuildId, UserId);

    /// Return an ID consisting of a tuple of the guild ID and user ID.
    fn id(&self) -> Self::Id {
        (self.guild_id, self.user_id)
    }
}

pub trait MemberRepository<Error: 'static>: Repository<MemberEntity, Error> {
    /// Retrieve the hoisted role associated with a role.
    fn hoisted_role(
        &self,
        guild_id: GuildId,
        user_id: UserId,
    ) -> GetEntityFuture<'_, RoleEntity, Error>;

    /// Retrieve a stream of roles associated with a member.
    ///
    /// Backend implementations aren't obligated to return roles in any
    /// particular order.
    fn roles(
        &self,
        guild_id: GuildId,
        user_id: UserId,
    ) -> ListEntitiesFuture<'_, RoleEntity, Error>;
}
