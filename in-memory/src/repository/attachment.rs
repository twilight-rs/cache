use crate::{config::EntityType, InMemoryBackendError, InMemoryBackendRef};
use futures_util::{
    future::{self, FutureExt},
    stream::{self, StreamExt},
};
use rarity_cache::{
    entity::{
        channel::{AttachmentEntity, AttachmentRepository, MessageEntity},
        Entity,
    },
    repository::{GetEntityFuture, ListEntitiesFuture, RemoveEntityFuture, Repository},
};
use std::{future::Future, pin::Pin, sync::Arc};
use twilight_model::id::AttachmentId;

/// Repository to retrieve and work with attachments and their related entities.
#[derive(Clone, Debug)]
pub struct InMemoryAttachmentRepository(pub(crate) Arc<InMemoryBackendRef>);

impl Repository<AttachmentEntity, InMemoryBackendError> for InMemoryAttachmentRepository {
    fn get(
        &self,
        attachment_id: AttachmentId,
    ) -> GetEntityFuture<'_, AttachmentEntity, InMemoryBackendError> {
        future::ok(
            self.0
                .attachments
                .get(&attachment_id)
                .map(|r| r.value().clone()),
        )
        .boxed()
    }

    fn list(&self) -> ListEntitiesFuture<'_, AttachmentEntity, InMemoryBackendError> {
        let stream = stream::iter(self.0.attachments.iter().map(|r| Ok(r.value().clone()))).boxed();

        future::ok(stream).boxed()
    }

    fn remove(&self, attachment_id: AttachmentId) -> RemoveEntityFuture<'_, InMemoryBackendError> {
        if !self
            .0
            .config
            .entity_types()
            .contains(EntityType::ATTACHMENT)
        {
            return future::ok(()).boxed();
        }

        self.0.attachments.remove(&attachment_id);

        future::ok(()).boxed()
    }

    fn upsert(
        &self,
        category_channel: AttachmentEntity,
    ) -> Pin<Box<dyn Future<Output = Result<(), InMemoryBackendError>> + Send>> {
        if !self
            .0
            .config
            .entity_types()
            .contains(EntityType::ATTACHMENT)
        {
            return future::ok(()).boxed();
        }

        self.0
            .attachments
            .insert(category_channel.id(), category_channel);

        future::ok(()).boxed()
    }
}

impl AttachmentRepository<InMemoryBackendError> for InMemoryAttachmentRepository {
    fn message<'a>(
        &'a self,
        attachment_id: AttachmentId,
    ) -> Pin<
        Box<dyn Future<Output = Result<Option<MessageEntity>, InMemoryBackendError>> + Send + 'a>,
    > {
        let message = self
            .0
            .attachments
            .get(&attachment_id)
            .map(|attachment| attachment.message_id)
            .and_then(|id| self.0.messages.get(&id))
            .map(|r| r.value().clone());

        future::ok(message).boxed()
    }
}

impl InMemoryAttachmentRepository {
    /// Retrieve the message of an attachment.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use rarity_cache_inmemory::InMemoryCache;
    /// use twilight_model::id::AttachmentId;
    ///
    /// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let cache = InMemoryCache::new();
    ///
    /// if let Some(message) = cache.attachments.message(AttachmentId(123456)).await? {
    ///     println!("the message's content is {}", message.content);
    /// }
    /// # Ok(()) }
    /// ```
    pub fn message<'a>(
        &'a self,
        attachment_id: AttachmentId,
    ) -> Pin<
        Box<dyn Future<Output = Result<Option<MessageEntity>, InMemoryBackendError>> + Send + 'a>,
    > {
        AttachmentRepository::message(self, attachment_id)
    }
}
