use crate::{config::EntityType, InMemoryBackend, InMemoryBackendError};
use futures_util::{
    future::{self, FutureExt},
    stream::{self, StreamExt},
};
use rarity_cache::{
    entity::{
        guild::{MemberEntity, MemberRepository, RoleEntity},
        Entity,
    },
    repository::{
        GetEntityFuture, ListEntitiesFuture, RemoveEntityFuture, Repository, UpsertEntityFuture,
    },
};
use twilight_model::id::{GuildId, UserId};

/// Repository to retrieve and work with members and their related entities.
#[derive(Clone, Debug)]
pub struct InMemoryMemberRepository(pub(crate) InMemoryBackend);

impl Repository<MemberEntity, InMemoryBackend> for InMemoryMemberRepository {
    fn backend(&self) -> &InMemoryBackend {
        &self.0
    }

    fn get(
        &self,
        id: (GuildId, UserId),
    ) -> GetEntityFuture<'_, MemberEntity, InMemoryBackendError> {
        future::ok((self.0).0.members.get(&id).map(|r| r.value().clone())).boxed()
    }

    fn list(&self) -> ListEntitiesFuture<'_, MemberEntity, InMemoryBackendError> {
        let stream = stream::iter((self.0).0.members.iter().map(|r| Ok(r.value().clone()))).boxed();

        future::ok(stream).boxed()
    }

    fn remove(&self, id: (GuildId, UserId)) -> RemoveEntityFuture<'_, InMemoryBackendError> {
        if !(self.0)
            .0
            .config
            .entity_types()
            .contains(EntityType::MEMBER)
        {
            return future::ok(()).boxed();
        }

        (self.0).0.members.remove(&id);

        future::ok(()).boxed()
    }

    fn upsert(&self, entity: MemberEntity) -> UpsertEntityFuture<'_, InMemoryBackendError> {
        if !(self.0)
            .0
            .config
            .entity_types()
            .contains(EntityType::MEMBER)
        {
            return future::ok(()).boxed();
        }

        (self.0).0.members.insert(entity.id(), entity);

        future::ok(()).boxed()
    }
}

impl MemberRepository<InMemoryBackend> for InMemoryMemberRepository {
    fn hoisted_role(
        &self,
        guild_id: GuildId,
        user_id: UserId,
    ) -> GetEntityFuture<'_, RoleEntity, InMemoryBackendError> {
        let role = self
            .0
             .0
            .members
            .get(&(guild_id, user_id))
            .and_then(|member| member.hoisted_role_id)
            .and_then(|id| (self.0).0.roles.get(&id))
            .map(|r| r.value().clone());

        future::ok(role).boxed()
    }

    fn roles(
        &self,
        guild_id: GuildId,
        user_id: UserId,
    ) -> ListEntitiesFuture<'_, RoleEntity, InMemoryBackendError> {
        let role_ids = match (self.0).0.members.get(&(guild_id, user_id)) {
            Some(member) => member.role_ids.clone(),
            None => return future::ok(stream::empty().boxed()).boxed(),
        };

        let iter = role_ids
            .into_iter()
            .filter_map(move |id| (self.0).0.roles.get(&id).map(|r| Ok(r.value().clone())));
        let stream = stream::iter(iter).boxed();

        future::ok(stream).boxed()
    }
}

impl InMemoryMemberRepository {
    /// Retrieve the hoisted role of a member.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rarity_cache_inmemory::InMemoryCache;
    /// use twilight_model::id::{GuildId, UserId};
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let cache = InMemoryCache::new();
    ///
    /// if let Some(role) = cache.members.hoisted_role(GuildId(1), UserId(2)).await? {
    ///     println!("the hoisted role's name is {}", role.name);
    /// }
    /// # Ok(()) }
    /// ```
    pub fn hoisted_role(
        &self,
        guild_id: GuildId,
        user_id: UserId,
    ) -> GetEntityFuture<'_, RoleEntity, InMemoryBackendError> {
        MemberRepository::hoisted_role(self, guild_id, user_id)
    }

    pub fn roles(
        &self,
        guild_id: GuildId,
        user_id: UserId,
    ) -> ListEntitiesFuture<'_, RoleEntity, InMemoryBackendError> {
        MemberRepository::roles(self, guild_id, user_id)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        InMemoryBackend, InMemoryMemberRepository, MemberEntity, MemberRepository, Repository,
    };
    use static_assertions::{assert_impl_all, assert_obj_safe};
    use std::fmt::Debug;

    assert_impl_all!(
        InMemoryMemberRepository:
        MemberRepository<InMemoryBackend>,
        Clone,
        Debug,
        Repository<MemberEntity, InMemoryBackend>,
        Send,
        Sync,
    );
    assert_obj_safe!(InMemoryMemberRepository);
}
