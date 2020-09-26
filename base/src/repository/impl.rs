use super::{
    super::{backend::Backend, entity::Entity},
    GetEntityFuture, ListEntitiesFuture, RemoveEntitiesFuture, RemoveEntityFuture,
    UpsertEntitiesFuture, UpsertEntityFuture,
};
use futures_util::future::{self, FutureExt, TryFutureExt};

pub trait Repository<E: Entity, B: Backend> {
    /// Retrieve an immutable reference to the backend that the repository is
    /// tied to.
    fn backend(&self) -> B;

    /// Get an entity by its ID in the cache.
    fn get(&self, entity_id: E::Id) -> GetEntityFuture<'_, E, B::Error>;

    /// Stream a list of records of the entity.
    fn list(&self) -> ListEntitiesFuture<'_, E, B::Error>;

    /// Remove an entity by its ID from the cache.
    fn remove(&self, entity_id: E::Id) -> RemoveEntityFuture<'_, B::Error>;

    /// Bulk remove multiple entities from the cache.
    ///
    /// **B implementations**: a default implementation is provided that
    /// will concurrently await [`remove`] calls for all provided entity IDs.
    /// This may not be optimal for all implementations, so you may want to
    /// implement this manually.
    ///
    /// [`remove`]: #tymethod.remove
    fn remove_bulk<T: Iterator<Item = E::Id>>(
        &self,
        entity_ids: T,
    ) -> RemoveEntitiesFuture<'_, B::Error> {
        future::try_join_all(entity_ids.map(|id| self.remove(id)))
            .map_ok(|_| ())
            .boxed()
    }

    /// Upsert an entity into the cache.
    fn upsert(&self, entity: E) -> UpsertEntityFuture<'_, B::Error>;

    /// Bulk upsert multiple entities in the cache.
    ///
    /// **B implementations**: a default implementation is provided that
    /// will concurrently await [`upsert`] calls for all provided entity IDs.
    /// This may not be optimal for all implementations, so you may want to
    /// implement this manually.
    ///
    /// [`upsert`]: #tymethod.upsert
    fn upsert_bulk<T: Iterator<Item = E> + Send>(
        &self,
        entities: T,
    ) -> UpsertEntitiesFuture<'_, B::Error> {
        Box::pin(future::try_join_all(entities.map(|entity| self.upsert(entity))).map_ok(|_| ()))
    }
}

pub trait SingleEntityRepository<E: Entity, B: Backend> {
    /// Retrieve an immutable reference to the backend that the repository is
    /// tied to.
    fn backend(&self) -> B;

    /// Get the entity in the cache.
    fn get(&self) -> GetEntityFuture<'_, E, B::Error>;

    /// Remove the entity from the cache.
    fn remove(&self) -> RemoveEntityFuture<'_, B::Error>;

    /// Upsert the entity into the cache.
    fn upsert(&self, entity: E) -> UpsertEntityFuture<'_, B::Error>;
}
