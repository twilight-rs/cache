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

pub trait Backend: Sized + 'static {
    type Error: Send + 'static;
    type AttachmentRepository: AttachmentRepository<Self> + Send + Sync;
    type CategoryChannelRepository: CategoryChannelRepository<Self> + Send + Sync;
    type EmojiRepository: EmojiRepository<Self> + Send + Sync;
    type GroupRepository: GroupRepository<Self> + Send + Sync;
    type GuildRepository: GuildRepository<Self> + Send + Sync;
    type MemberRepository: MemberRepository<Self> + Send + Sync;
    type MessageRepository: MessageRepository<Self> + Send + Sync;
    type PresenceRepository: PresenceRepository<Self> + Send + Sync;
    type PrivateChannelRepository: PrivateChannelRepository<Self> + Send + Sync;
    type RoleRepository: RoleRepository<Self> + Send + Sync;
    type TextChannelRepository: TextChannelRepository<Self> + Send + Sync;
    type UserRepository: UserRepository<Self> + Send + Sync;
    type VoiceChannelRepository: VoiceChannelRepository<Self> + Send + Sync;
    type VoiceStateRepository: VoiceStateRepository<Self> + Send + Sync;

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
