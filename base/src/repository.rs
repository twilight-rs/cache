use super::entity::Entity as EntityTrait;
use futures_util::{
    future::{self, FutureExt, TryFutureExt},
    stream::Stream,
};
use std::{future::Future, pin::Pin};

pub type GetEntityFuture<'a, T, E> =
    Pin<Box<dyn Future<Output = Result<Option<T>, E>> + Send + 'a>>;
pub type ListEntitiesFuture<'a, T, E> =
    Pin<Box<dyn Future<Output = Result<ListEntitiesStream<'a, T, E>, E>> + Send + 'a>>;
pub type ListEntitiesStream<'a, T, E> = Pin<Box<dyn Stream<Item = Result<T, E>> + Send + 'a>>;
pub type ListEntityIdsFuture<'a, T, E> =
    Pin<Box<dyn Future<Output = Result<ListEntityIdsStream<'a, T, E>, E>> + Send + 'a>>;
pub type ListEntityIdsStream<'a, T, E> = Pin<Box<dyn Stream<Item = Result<T, E>> + Send + 'a>>;
pub type RemoveEntityFuture<'a, E> = Pin<Box<dyn Future<Output = Result<(), E>> + Send + 'a>>;
pub type RemoveEntitiesFuture<'a, E> = Pin<Box<dyn Future<Output = Result<(), E>> + Send + 'a>>;
pub type UpsertEntityFuture<'a, E> = Pin<Box<dyn Future<Output = Result<(), E>> + Send + 'a>>;
pub type UpsertEntitiesFuture<'a, E> = Pin<Box<dyn Future<Output = Result<(), E>> + Send + 'a>>;

pub trait Repository<Entity: EntityTrait, Error: 'static> {
    /// Get an entity by its ID in the cache.
    fn get(&self, entity_id: Entity::Id) -> GetEntityFuture<'_, Entity, Error>;

    /// Stream a list of records of the entity.
    fn list(&self) -> ListEntitiesFuture<'_, Entity, Error>;

    /// Remove an entity by its ID from the cache.
    fn remove(&self, entity_id: Entity::Id) -> RemoveEntityFuture<'_, Error>;

    /// Bulk remove multiple entities from the cache.
    ///
    /// **Backend implementations**: a default implementation is provided that
    /// will concurrently await [`remove`] calls for all provided entity IDs.
    /// This may not be optimal for all implementations, so you may want to
    /// implement this manually.
    ///
    /// [`remove`]: #tymethod.remove
    fn remove_bulk<T: Iterator<Item = Entity::Id>>(
        &self,
        entity_ids: T,
    ) -> RemoveEntitiesFuture<'_, Error> {
        future::try_join_all(entity_ids.map(|id| self.remove(id)))
            .map_ok(|_| ())
            .boxed()
    }

    /// Upsert an entity into the cache.
    fn upsert(&self, entity: Entity) -> UpsertEntityFuture<'_, Error>;

    /// Bulk upsert multiple entities in the cache.
    ///
    /// **Backend implementations**: a default implementation is provided that
    /// will concurrently await [`upsert`] calls for all provided entity IDs.
    /// This may not be optimal for all implementations, so you may want to
    /// implement this manually.
    ///
    /// [`upsert`]: #tymethod.upsert
    fn upsert_bulk<T: Iterator<Item = Entity> + Send>(
        &self,
        entities: T,
    ) -> UpsertEntitiesFuture<'_, Error> {
        Box::pin(future::try_join_all(entities.map(|entity| self.upsert(entity))).map_ok(|_| ()))
    }
}
