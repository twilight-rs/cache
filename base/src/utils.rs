use super::{
    backend::Backend,
    entity::Entity,
    repository::{GetEntityFuture, ListEntitiesFuture, Repository},
};
use futures_util::stream::{self, StreamExt};

pub fn relation_and_then<
    'a,
    B: Backend + 'a,
    F: FnOnce(M1) -> Option<M2::Id> + Send + 'a,
    M1: Entity + 'a,
    M2: Entity + 'a,
    R1: Repository<M1, B> + Send + 'a,
    R2: Repository<M2, B> + Send + Sync + 'a,
>(
    repo: R1,
    foreign: R2,
    id: M1::Id,
    f: F,
) -> GetEntityFuture<'a, M2, B::Error>
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
    'a,
    B: Backend + 'a,
    F: FnOnce(M1) -> M2::Id + Send + 'a,
    M1: Entity + 'a,
    M2: Entity + 'a,
    R1: Repository<M1, B> + Send + 'a,
    R2: Repository<M2, B> + Send + Sync + 'a,
>(
    repo: R1,
    foreign: R2,
    id: M1::Id,
    f: F,
) -> GetEntityFuture<'a, M2, B::Error>
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

pub fn stream<
    'a,
    B: Backend + 'a,
    F: FnOnce(M1) -> I + Send + 'a,
    I: Iterator<Item = M2::Id> + Send + 'a,
    M1: Entity + 'a,
    M2: Entity + 'a,
    R1: Repository<M1, B> + Send + 'a,
    R2: Repository<M2, B> + Send + 'a,
>(
    repo: R1,
    foreign: R2,
    id: M1::Id,
    f: F,
) -> ListEntitiesFuture<'a, M2, B::Error> {
    struct StreamState<I, R2> {
        foreign: R2,
        ids: I,
    }

    Box::pin(async move {
        let fut = repo.get(id);

        let foreign_ids = if let Some(entity) = fut.await? {
            f(entity)
        } else {
            return Ok(stream::empty().boxed());
        };

        let state = StreamState {
            foreign,
            ids: foreign_ids,
        };

        Ok(stream::unfold(state, |mut state| async move {
            loop {
                let id = state.ids.next()?;

                let fut = state.foreign.get(id);

                match fut.await {
                    Ok(Some(e)) => return Some((Ok(e), state)),
                    Ok(None) => continue,
                    Err(why) => return Some((Err(why), state)),
                }
            }
        })
        .boxed())
    })
}
