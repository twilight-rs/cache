use super::message::MessageEntity;
use crate::{
    repository::{GetEntityFuture, Repository},
    utils, Backend, Entity,
};
use twilight_model::{
    channel::Attachment,
    id::{AttachmentId, MessageId},
};

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

impl From<(MessageId, Attachment)> for AttachmentEntity {
    fn from((message_id, attachment): (MessageId, Attachment)) -> Self {
        Self {
            filename: attachment.filename,
            height: attachment.height,
            id: attachment.id,
            message_id,
            proxy_url: attachment.proxy_url,
            size: attachment.size,
            url: attachment.url,
            width: attachment.width,
        }
    }
}

impl Entity for AttachmentEntity {
    type Id = AttachmentId;

    /// Return the attachment's ID.
    fn id(&self) -> Self::Id {
        self.id
    }
}

pub trait AttachmentRepository<B: Backend>: Repository<AttachmentEntity, B> + Send {
    fn message(&self, attachment_id: AttachmentId) -> GetEntityFuture<'_, MessageEntity, B::Error> {
        utils::relation_map(
            self.backend().attachments(),
            self.backend().messages(),
            attachment_id,
            |attachment| attachment.message_id,
        )
    }
}
