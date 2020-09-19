use bitflags::bitflags;

bitflags! {
    /// Flags to enable which entities to operate on.
    ///
    /// Disabled entities will have their repositories skip upsert and remove
    /// operations, which means that all entity retrievals will result in
    /// `None`.
    pub struct EntityType: u64 {
        const ATTACHMENT = 1 << 0;
        const CHANNEL_CATEGORY = 1 << 1;
        const CHANNEL_GROUP = 1 << 2;
        const CHANNEL_PRIVATE = 1 << 3;
        const CHANNEL_TEXT = 1 << 4;
        const CHANNEL_VOICE = 1 << 5;
        const EMOJI = 1 << 6;
        const GUILD = 1 << 7;
        const MEMBER = 1 << 8;
        const MESSAGE = 1 << 9;
        const PRESENCE = 1 << 10;
        const ROLE = 1 << 11;
        const USER = 1 << 12;
        const USER_CURRENT = 1 << 13;
        const VOICE_STATE = 1 << 14;
    }
}

/// Configuration for the in memory backend.
///
/// Refer to each setter method to know the default value.
#[derive(Clone, Debug)]
pub struct Config {
    entity_types: EntityType,
    message_cache_size: usize,
}

impl Config {
    /// Returns an immutable reference to the entity types enabled.
    pub fn entity_types(&self) -> EntityType {
        self.entity_types
    }

    /// Returns a mutable reference to the entity types enabled.
    ///
    /// Disabled entities will have their repositories skip upsert and remove
    /// operations, which means that all entity retrievals will result in
    /// `None`.
    ///
    /// Defaults to all entity types.
    pub fn entity_types_mut(&mut self) -> &mut EntityType {
        &mut self.entity_types
    }

    /// Returns an immutable reference to the message cache size.
    pub fn message_cache_size(&self) -> usize {
        self.message_cache_size
    }

    /// Returns a mutable reference to the message cache size per channel.
    ///
    /// Defaults to 100.
    pub fn message_cache_size_mut(&mut self) -> &mut usize {
        &mut self.message_cache_size
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            entity_types: EntityType::all(),
            message_cache_size: 100,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, EntityType};
    use static_assertions::{assert_impl_all, assert_obj_safe};
    use std::fmt::Debug;

    assert_impl_all!(Config: Clone, Debug, Send, Sync);
    assert_impl_all!(EntityType: Clone, Copy, Debug, Eq, PartialEq, Send, Sync);
    assert_obj_safe!(Config, EntityType);

    #[test]
    fn test_event_type_const_values() {
        assert_eq!(1 << 0, EntityType::ATTACHMENT.bits());
        assert_eq!(1 << 1, EntityType::CHANNEL_CATEGORY.bits());
        assert_eq!(1 << 2, EntityType::CHANNEL_GROUP.bits());
        assert_eq!(1 << 3, EntityType::CHANNEL_PRIVATE.bits());
        assert_eq!(1 << 4, EntityType::CHANNEL_TEXT.bits());
        assert_eq!(1 << 5, EntityType::CHANNEL_VOICE.bits());
        assert_eq!(1 << 6, EntityType::EMOJI.bits());
        assert_eq!(1 << 7, EntityType::GUILD.bits());
        assert_eq!(1 << 8, EntityType::MEMBER.bits());
        assert_eq!(1 << 9, EntityType::MESSAGE.bits());
        assert_eq!(1 << 10, EntityType::PRESENCE.bits());
        assert_eq!(1 << 11, EntityType::ROLE.bits());
        assert_eq!(1 << 12, EntityType::USER.bits());
        assert_eq!(1 << 13, EntityType::USER_CURRENT.bits());
        assert_eq!(1 << 14, EntityType::VOICE_STATE.bits());
    }

    #[test]
    fn test_defaults() {
        let conf = Config {
            entity_types: EntityType::all(),
            message_cache_size: 100,
        };
        let default = Config::default();
        assert_eq!(conf.entity_types, default.entity_types);
        assert_eq!(conf.message_cache_size, default.message_cache_size);
    }

    #[test]
    fn test_config_fields() {
        static_assertions::assert_fields!(Config: entity_types, message_cache_size);
    }
}
