# Rust-Redis

![Picture](/RR.png)

Rust-Redis is a lightweight Redis server implementation written in Rust. It's designed to help you understand how Redis works under the hood, while taking advantage of Rust's features for concurrency and performance. It is a learning tool that brings Redis to life, built from scratch with the power of Rust.

## Features

- Common Redis commands supported: `ECHO`, `SET`, `GET`
- Key expiration with the `PX` argument
- Command parsing that follows the Redis protocol
- RDB (Redis Database) persistence support
- `CONFIG GET` command for configuration management
- Ability to read and retrieve data from RDB files
- `KEYS` command for searching keys stored in the RDB file

## Installation

Getting started with Rust-Redis is easy! Just clone the repository and build it with Cargo:

```bash
# Clone the repository
git clone https://github.com/yourusername/rust-redis.git

# Navigate to the project directory
cd rust-redis

# Build using Cargo
cargo build --release
```

## Usage

To run the server, simply use:

```bash
cargo run --release
```

By default, Rust-Redis will start listening on port `6379`. You can connect to it using any Redis client you prefer.

## Supported Commands

- **ECHO <message>**: Returns the message you send as a RESP bulk string.
- **SET <key> <value> [PX <milliseconds>]**: Stores a key-value pair, optionally with an expiration time. Returns `+OK` if successful.
- **GET <key>**: Retrieves the value for the given key. If the key doesn't exist, it returns `$-1`.
- **CONFIG GET <parameter>**: Gets configuration parameters like `dir` or `dbfilename`.
- **KEYS <pattern>**: Searches for keys in the RDB file that match the given pattern.

## Roadmap

- Add more Redis commands (`DEL`, `INCR`, `DECR`, etc.)
- Enhance key expiration accuracy
- Implement advanced data structures (`Lists`, `Sets`, `Hashes`)
- Add AOF (Append-Only File) persistence for better durability
- Optimize performance for larger-scale scenarios

## Acknowledgments

- [Redis](https://redis.io) - The original inspiration for this project.
- [Tokio](https://tokio.rs) - The async runtime that helps manage all those connections.
