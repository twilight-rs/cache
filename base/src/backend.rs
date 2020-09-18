use super::entity::{
    channel::{
        AttachmentRepository, CategoryChannelRepository, GroupRepository, MessageRepository,
        PrivateChannelRepository, TextChannelRepository, VoiceChannelRepository,
    },
    gateway::PresenceRepository,
    guild::{EmojiRepository, GuildRepository, MemberRepository, RoleRepository},
    user::UserRepository,
    voice::VoiceStateRepository,
};

pub trait Backend: Sized {
    type Error: 'static;
    type AttachmentRepository: AttachmentRepository<Self>;
    type CategoryChannelRepository: CategoryChannelRepository<Self>;
    type EmojiRepository: EmojiRepository<Self>;
    type GroupRepository: GroupRepository<Self>;
    type GuildRepository: GuildRepository<Self>;
    type MemberRepository: MemberRepository<Self>;
    type MessageRepository: MessageRepository<Self>;
    type PresenceRepository: PresenceRepository<Self>;
    type PrivateChannelRepository: PrivateChannelRepository<Self>;
    type RoleRepository: RoleRepository<Self>;
    type TextChannelRepository: TextChannelRepository<Self>;
    type UserRepository: UserRepository<Self>;
    type VoiceChannelRepository: VoiceChannelRepository<Self>;
    type VoiceStateRepository: VoiceStateRepository<Self>;

    /// Return a new instance of the backend's attachment repository
    /// implementation.
    fn attachments(&self) -> Self::AttachmentRepository;

    /// Return a new instance of the backend's guild category channel repository
    /// implementation.
    fn category_channels(&self) -> Self::CategoryChannelRepository;

    /// Return a new instance of the backend's emoji repository implementation.
    fn emojis(&self) -> Self::EmojiRepository;

    /// Return a new instance of the backend's group repository implementation.
    fn groups(&self) -> Self::GroupRepository;

    /// Return a new instance of the backend's guild repository implementation.
    fn guilds(&self) -> Self::GuildRepository;

    /// Return a new instance of the backend's member repository implementation.
    fn members(&self) -> Self::MemberRepository;

    /// Return a new instance of the backend's message repository
    /// implementation.
    fn messages(&self) -> Self::MessageRepository;

    /// Return a new instance of the backend's presence repository
    /// implementation.
    fn presences(&self) -> Self::PresenceRepository;

    /// Return a new instance of the backend's guild private channel repository
    /// implementation.
    fn private_channels(&self) -> Self::PrivateChannelRepository;

    /// Return a new instance of the backend's role repository implementation.
    fn roles(&self) -> Self::RoleRepository;

    /// Return a new instance of the backend's guild text channel repository
    /// implementation.
    fn text_channels(&self) -> Self::TextChannelRepository;

    /// Return a new instance of the backend's user repository implementation.
    fn users(&self) -> Self::UserRepository;

    /// Return a new instance of the backend's voice channel repository
    /// implementation.
    fn voice_channels(&self) -> Self::VoiceChannelRepository;

    /// Return a new instance of the backend's voice state repository
    /// implementation.
    fn voice_states(&self) -> Self::VoiceStateRepository;
}
