pub mod channel;
pub mod gateway;
pub mod guild;
pub mod user;
pub mod voice;

use std::hash::Hash;

/// Efficient cachable entities mapping to the models returned from Discord's
/// API.
///
/// For example, the [`EmojiEntity`] does not contain the user data within it,
/// but contains only the ID of the user. This can act similar to foreign keys
/// in a relational database.
///
/// [`EmojiEntity`]: emoji/struct.EmojiEntity.html
pub trait Entity: Send + Sync {
    type Id: Copy + Eq + Hash + Send + Sync;

    /// Return the ID of the entity.
    ///
    /// For entities like the [`EmojiEntity`] this will return an ID consisting
    /// of the emoji's ID, while for entities like the [`MemberEntity`] this
    /// will return a tuple pair of the member's guild ID and member's user ID.
    ///
    /// [`EmojiEntity`]: emoji/struct.EmojiEntity.html
    /// [`MemberEntity`]: member/struct.MemberEntity.html
    fn id(&self) -> Self::Id;
}
