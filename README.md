# Twilight Cache

Twilight Cache is an entity/repository-based cache for the [`twilight-rs`]
ecosystem. If you're not familiar with entities and repositories,
[Microsoft has an article][docs:repo:microsoft] about it. The idea is that
entities - things like guilds, channels, or users - contain data about
themselves, while repositories are for retrieving, removing, and modifying those
entries at the data source.

## Structure

### Base Crate

The primary crate is `twilight-cache`, which is a generic abstraction over any
datastore backend. It can be built on top of to write backends for things like
Redis, CouchDB, SQLite, in-process-memory, or anything else.

This is largely a set of 3 traits and a collection of models. The models define
efficient data structures for storage (utilising IDs where possible for
optimised memory and query performance, similar to a foreign key).

The `Entity` trait is for common operations - such as getting an entity's unique
ID - on resources such as users, guilds, or members.

The `Repository` trait defines common CRUD operations on entities: upserting,
retrieving, and deleting entities (as well as bulk operations). There is also a
repository for each entity for unique operations relevant to that entity, such
as `MessageRepository::author` to get the author of a message by the message ID.
This avoids having to query a backend twice: getting the message by ID, and then
getting an author by the message's `author_id` field.

Lastly, the `Backend` trait includes methods and types for creating each
backend's repository implementations.

### Implementations

Provided is the `twilight-cache-inmemory` implementation, which caches entities in
the memory of the process. A Redis implementation is planned.

## Examples

Get a message by its ID, and then get a different message's author, knowing only
the ID of both messages:

```rust
use twilight_cache_inmemory::InMemoryCache;
use twilight_model::id::MessageId;

let cache = InMemoryCache::new();

if let Some(message) = cache.messages.get(MessageId(123)) {
    println!("the content of the message is: {}", message.content);
} else {
    println!("the message isn't in the cache");
}

if let Some(author) = cache.messages.author(MessageId(456)).await? {
    println!("the author of the message is {}#{}", author.name, author.discriminator);
} else {
    println!("the message or its author isn't in the cache");
}
```

## Installation

Add the following to your `Cargo.toml` to install the `twilight-cache-inmemory`
crate:

```toml
[dependencies]
twilight-cache-inmemory = { branch = "main", git = "https://github.com/twilight-rs/cache" }
```

## License

ISC.

[`twilight-rs`]: https://twilight.rs
[docs:repo:microsoft]: https://docs.microsoft.com/en-us/dotnet/architecture/microservices/microservice-ddd-cqrs-patterns/infrastructure-persistence-layer-design
