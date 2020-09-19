use super::{
    backend::Backend,
    entity::Entity,
    repository::{GetEntityFuture, Repository},
};

pub fn relation_and_then<
    B: Backend + 'static,
    F: FnOnce(M1) -> Option<M2::Id> + Send + 'static,
    M1: Entity + 'static,
    M2: Entity + Send + 'static,
    R1: Repository<M1, B> + Send + 'static,
    R2: Repository<M2, B> + Send + Sync + 'static,
>(
    repo: R1,
    foreign: R2,
    id: M1::Id,
    f: F,
) -> GetEntityFuture<'static, M2, B::Error>
where
    B::Error: Send,
{
    Box::pin(async move {
        let fut = repo.get(id);

        let foreign_id = if let Some(foreign_id) = fut.await?.and_then(|e| f(e)) {
            foreign_id
        } else {
            return Ok(None);
        };

        foreign.get(foreign_id).await
    })
}

pub fn relation_map<
    B: Backend + 'static,
    F: FnOnce(M1) -> M2::Id + Send + 'static,
    M1: Entity + 'static,
    M2: Entity + Send + 'static,
    R1: Repository<M1, B> + Send + 'static,
    R2: Repository<M2, B> + Send + Sync + 'static,
>(
    repo: R1,
    foreign: R2,
    id: M1::Id,
    f: F,
) -> GetEntityFuture<'static, M2, B::Error>
where
    B::Error: Send,
{
    Box::pin(async move {
        let fut = repo.get(id);

        let foreign_id = if let Some(entity) = fut.await? {
            f(entity)
        } else {
            return Ok(None);
        };

        foreign.get(foreign_id).await
    })
}

// pub fn stream<'a, E: Send + 'a, K: Eq + Hash + Send, V: Clone + Send>(
//     map: &'a DashMap<K, V>,
// ) -> ListEntitiesFuture<'a, V, E> {
//     let iter = stream::iter(map.iter().map(|r| Ok(r.value().clone()))).boxed();

//     future::ok(iter).boxed()
// }
