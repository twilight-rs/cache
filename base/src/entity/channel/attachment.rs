use super::message::MessageEntity;
use crate::{
    repository::{GetEntityFuture, Repository},
    Backend, Entity,
};
use twilight_model::id::{AttachmentId, MessageId};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AttachmentEntity {
    pub filename: String,
    pub height: Option<u64>,
    pub id: AttachmentId,
    pub message_id: MessageId,
    pub proxy_url: String,
    pub size: u64,
    pub url: String,
    pub width: Option<u64>,
}

impl Entity for AttachmentEntity {
    type Id = AttachmentId;

    /// Return the attachment's ID.
    fn id(&self) -> Self::Id {
        self.id
    }
}

pub trait AttachmentRepository<B: Backend>: Repository<AttachmentEntity, B> {
    fn message(&self, attachment_id: AttachmentId) -> GetEntityFuture<'_, MessageEntity, B::Error>;
}
