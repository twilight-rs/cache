//! # rarity-cache
//!
//! `rarity-cache` is an implementation of a generic cache to support any type
//! of datastore backend implementation. Implementations of backends can be
//! anything. To name a few examples: a backend to work with a database like
//! Cassandra or MongoDB, an in-memory database like Redis or memcached, or a
//! cache held in the process' memory.
//!
//! # Design
//!
//! The cache is built around the repository pattern. If you're not familiar
//! with entities and repositories,
//! [Microsoft has an article][docs:repo:microsoft] about it. The idea is that
//! entities - things like guilds, channels, or users - contain data about
//! themselves, while repositories are for retrieving, removing, and modifying
//! those entries at the data source.
//!
//! # Backends
//!
//! Here's a list of backends supported by Rarity:
//!
//! - [`rarity-cache-inmemory`]: datastore in the process's memory
//!
//! # Usage
//!
//! Library users shouldn't use too much of this library directly. This library
//! is a set oftraits for datastore backends to implement. Users should use the
//! [`Cache`] to update the cache with new event data and retrieve things like
//! users or emojis.
//!
//! Here's an example of using the cache with the [`rarity-cache-inmemory`]
//! backend:
//!
//! ```rust,no_run
//! // Import the cache and the Repository trait to work with the repositories
//! // that the backend implements.
//! use futures_util::stream::StreamExt;
//! use rarity_cache::{Cache, Repository};
//! use rarity_cache_inmemory::InMemoryBackend;
//! use twilight_model::id::{GuildId, MessageId};
//!
//! # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Here we create a cache that uses the in-memory backend. This can be
//! // swapped out with any backend.
//! let cache: Cache<InMemoryBackend> = Cache::new();
//!
//! // To update the cache, you pass events from the gateway into it. Say that
//! // we already have a gateway event to update the cache with:
//! # let event = twilight_model::gateway::event::Event::GatewayHeartbeatAck;
//! cache.update(&event).await?;
//!
//! // It's as easy as calling the `update` method. The cache will call the
//! // backend's repositories to insert, remove, and update data from events.
//! // Updates might error if the backend errors when doing something.
//! //
//! // Now let's get a message by its ID and print its content if it exists in
//! // the cache:
//! let message_id = MessageId(123_456_789);
//!
//! if let Some(message) = cache.messages.get(message_id).await? {
//!     println!("the message's content is: {}", message.content);
//! }
//!
//! // And now let's asynchronously iterate over the members in a guild:
//! let guild_id = GuildId(987_654_321);
//!
//! // Create an iterator over the members:
//! let mut members = cache.guilds.members(guild_id).await?;
//!
//! while let Some(member) = members.next().await {
//!     // The member is wrapped in a Result in case if there's a problem
//!     // retrieving it, such as due to a deserialization error.
//!     let member = member?;
//!
//!     println!("the member's user ID is {}", member.user_id);
//! }
//! # Ok(()) }
//! ```
//!
//! # Backend implementors
//!
//! **Note**: This section is for people interested in implementing a new
//! backend to work with a datastore.
//!
//! Backend implementations have to do two things: implement the [`Repository`]
//! trait for each entity, and implement the [`Backend`] trait which returns
//! instances of those repositories.
//!
//! For detailed information, read the documentation for both traits.
//!
//! # Features
//!
//! The `serde` feature can be disabled to remove the `Deserialize` and
//! `Serialize` implementations on entities. It is enabled by default.
//!
//! [`rarity-cache-inmemory`]: ../rarity_cache_inmemory/index.html
//! [docs:repo:microsoft]: https://docs.microsoft.com/en-us/dotnet/architecture/microservices/microservice-ddd-cqrs-patterns/infrastructure-persistence-layer-design

#![deny(
    clippy::all,
    clippy::pedantic,
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms,
    unused,
    warnings
)]
#![allow(
    clippy::doc_markdown,
    clippy::match_same_arms,
    clippy::module_name_repetitions,
    clippy::must_use_candidate
)]

pub mod entity;
pub mod repository;

mod backend;
mod cache;
mod utils;

pub use self::{backend::Backend, cache::Cache, entity::Entity, repository::Repository};
