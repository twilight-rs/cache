use super::{super::user::UserEntity, MessageEntity};
use crate::{
    repository::{GetEntityFuture, ListEntitiesFuture, Repository},
    utils, Backend, Entity,
};
use twilight_model::{
    channel::{ChannelType, Group},
    id::{ApplicationId, ChannelId, MessageId, UserId},
};

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroupEntity {
    pub application_id: Option<ApplicationId>,
    pub icon: Option<String>,
    pub id: ChannelId,
    pub kind: ChannelType,
    pub last_message_id: Option<MessageId>,
    pub last_pin_timestamp: Option<String>,
    pub name: Option<String>,
    pub owner_id: UserId,
    pub recipient_ids: Vec<UserId>,
}

impl From<Group> for GroupEntity {
    fn from(group: Group) -> Self {
        let recipient_ids = group.recipients.into_iter().map(|user| user.id).collect();

        Self {
            application_id: group.application_id,
            icon: group.icon,
            id: group.id,
            kind: group.kind,
            last_message_id: group.last_message_id,
            last_pin_timestamp: group.last_pin_timestamp,
            name: group.name,
            owner_id: group.owner_id,
            recipient_ids,
        }
    }
}

impl Entity for GroupEntity {
    type Id = ChannelId;

    /// Return the group's ID.
    fn id(&self) -> Self::Id {
        self.id
    }
}

pub trait GroupRepository<B: Backend>: Repository<GroupEntity, B> {
    /// Retrieve the last message of a group.
    fn last_message(&self, group_id: ChannelId) -> GetEntityFuture<'_, MessageEntity, B::Error> {
        utils::relation_and_then(
            self.backend().groups(),
            self.backend().messages(),
            group_id,
            |group| group.last_message_id,
        )
    }

    /// Retrieve the owner of a group.
    fn owner(&self, group_id: ChannelId) -> GetEntityFuture<'_, UserEntity, B::Error> {
        utils::relation_map(
            self.backend().groups(),
            self.backend().users(),
            group_id,
            |group| group.owner_id,
        )
    }

    /// Retrieve a stream of recipients associated with a group.
    fn recipients(&self, group_id: ChannelId) -> ListEntitiesFuture<'_, UserEntity, B::Error> {
        utils::stream(
            self.backend().groups(),
            self.backend().users(),
            group_id,
            |group| group.recipient_ids.into_iter(),
        )
    }
}
