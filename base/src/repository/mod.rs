mod r#impl;
mod noop;

pub use self::{noop::NoopRepository, r#impl::Repository};

use futures_util::stream::Stream;
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
