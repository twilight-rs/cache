use crate::{config::EntityType, InMemoryBackend, InMemoryBackendError};
use futures_util::{
    future::{self, FutureExt},
    stream::{self, StreamExt},
};
use rarity_cache::{
    entity::{
        channel::{AttachmentEntity, AttachmentRepository, MessageEntity},
        Entity,
    },
    repository::{
        GetEntityFuture, ListEntitiesFuture, RemoveEntityFuture, Repository, UpsertEntityFuture,
    },
};
use twilight_model::id::AttachmentId;

/// Repository to retrieve and work with attachments and their related entities.
#[derive(Clone, Debug)]
pub struct InMemoryAttachmentRepository(pub(crate) InMemoryBackend);

impl Repository<AttachmentEntity, InMemoryBackend> for InMemoryAttachmentRepository {
    fn backend(&self) -> &InMemoryBackend {
        &self.0
    }

    fn get(
        &self,
        attachment_id: AttachmentId,
    ) -> GetEntityFuture<'_, AttachmentEntity, InMemoryBackendError> {
        future::ok(
            (self.0)
                .0
                .attachments
                .get(&attachment_id)
                .map(|r| r.value().clone()),
        )
        .boxed()
    }

    fn list(&self) -> ListEntitiesFuture<'_, AttachmentEntity, InMemoryBackendError> {
        let stream =
            stream::iter((self.0).0.attachments.iter().map(|r| Ok(r.value().clone()))).boxed();

        future::ok(stream).boxed()
    }

    fn remove(&self, attachment_id: AttachmentId) -> RemoveEntityFuture<'_, InMemoryBackendError> {
        if !self
            .0
             .0
            .config
            .entity_types()
            .contains(EntityType::ATTACHMENT)
        {
            return future::ok(()).boxed();
        }

        (self.0).0.attachments.remove(&attachment_id);

        future::ok(()).boxed()
    }

    fn upsert(
        &self,
        category_channel: AttachmentEntity,
    ) -> UpsertEntityFuture<'_, InMemoryBackendError> {
        if !self
            .0
             .0
            .config
            .entity_types()
            .contains(EntityType::ATTACHMENT)
        {
            return future::ok(()).boxed();
        }

        self.0
             .0
            .attachments
            .insert(category_channel.id(), category_channel);

        future::ok(()).boxed()
    }
}

impl AttachmentRepository<InMemoryBackend> for InMemoryAttachmentRepository {
    fn message(
        &self,
        attachment_id: AttachmentId,
    ) -> GetEntityFuture<'_, MessageEntity, InMemoryBackendError> {
        let message = self
            .0
             .0
            .attachments
            .get(&attachment_id)
            .map(|attachment| attachment.message_id)
            .and_then(|id| (self.0).0.messages.get(&id))
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
    pub fn message(
        &self,
        attachment_id: AttachmentId,
    ) -> GetEntityFuture<'_, MessageEntity, InMemoryBackendError> {
        AttachmentRepository::message(self, attachment_id)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AttachmentEntity, AttachmentRepository, InMemoryAttachmentRepository, InMemoryBackend,
        Repository,
    };
    use static_assertions::{assert_impl_all, assert_obj_safe};
    use std::fmt::Debug;

    assert_impl_all!(
        InMemoryAttachmentRepository:
        AttachmentRepository<InMemoryBackend>,
        Clone,
        Debug,
        Repository<AttachmentEntity, InMemoryBackend>,
        Send,
        Sync,
    );
    assert_obj_safe!(InMemoryAttachmentRepository);
}
