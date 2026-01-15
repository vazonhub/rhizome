# Rhizome - P2P protocol

<img src="docs\icon.png" width="250">

[![Crates.io](https://img.shields.io/crates/v/rhizome-p2p.svg)](https://crates.io/crates/rhizome-p2p)
[![Documentation](https://docs.rs/rhizome-p2p/badge.svg)](https://docs.rs/rhizome-p2p)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Rust Edition](https://img.shields.io/badge/Rust-2024-orange.svg)](https://doc.rust-lang.org/edition-guide/rust-2024/index.html)
[![Platform](https://img.shields.io/badge/Platform-Native%20%7C%20WASM-brightgreen.svg)](#)

Rhizome is a high—performance, decentralized P2P messaging library implemented on Rust. It is based on the Kademlia DHT protocol with custom data replication and content ranking mechanisms.

## ✨ Features
- 🦀 `Rust Core`: Maximum performance and memory security without GC.
- 🔒 `Anonymity`: DHT-based routing hides direct connections between network participants.
- ⚡ `Async First`: A fully asynchronous stack based on tokio and futures.
- 🔄 `Smart replication`: Automatic distribution of data to k-nearest nodes.
- 📈 `Popularity system`: Content in demand gets storage priority and a higher TTL.
- 📦 `Modularity`: You can use it as a ready-made CLI node, or connect it as a library (cargo lib) to your project.

## 🛠 Technology stack
- `Runtime & Async`: Fully asynchronous architecture based on tokio (full) and futures. Using async-trait for flexible component design.
- `Persistence (Storage)`: heed is a high—performance embedded database (a wrapper over LMDB) that provides ACID transactions and instant access to data.
- `Cryptography & Security`:
- `RSA (with SHA-2 support)` for key management and digital signatures.
    - `sha1, sha2, digest` — a set of cryptographic hash functions for data integrity and identification in DHT.
- `Serialization`:
    - `rmp-serde (MessagePack)` is the main binary protocol for minimizing traffic in a P2P network.
    - `serde_json & serde_yaml` — for configuration and external `APIs'.
- `Observability (Logging)`: An advanced system based on `tracing`. Support for structured logging (JSON), filtering via env-filter, and log file rotation via tracing-appender.
- `Portability (WASM)`: Support for compilation to `WebAssembly' (wasm-bindgen) for use in browser environments, including integration with getrandom/js.
- `Development & Quality`:
    - Automatic style and linting control via `cargo-husky` (pre-commit hooks for fmt and clippy).
    - The use of `thiserror` for strict and understandable error typing.

## 📂 Project structure
```
rhizome/
├── examples/            # Examples of the system operation
├── src/                 # The main project code
│   ├── config.rs        # Configuration Module
│   ├── logger.rs        # The logging module
│   ├── api.rs           # API module for external operation
│   ├── exception.rs     # Error management module
│   ├── dht/             # Kademlia DHT Module
│   ├── network/         # Network operation module
│   ├── node/            # Node Module
│   ├── popularity/      # A module for the operation of the reputation system
│   ├── replication/     # Data replication
│   ├── storage/         # Storage System Module
│   ├── utils/           # Auxiliary functions module
│   └── security/        # The security module
```

## 🛠 Setup and develop
For project build you need Rust version 1.85+ (because we will use Edition 2024).
```code
rustup update stable
```

### Clone and build
```code
Bash
git clone https://github.com/vazonhub/rhizome.git
cd rhizome

cargo build
```

### Run tests
For running tests you can use:
```code
Bash
# Run all tests
cargo test

# Run tests with logs in console
RUST_LOG=debug cargo test -- --nocapture
```

### Static analyze and formating
In project, we have some feature for code analyze and formating:
- Formating by (`cargo fmt`)
- Analyze linter by (`cargo clippy`)

## 🤝 Participation in the development
We are happy to see your Pull Requests!
1. Create fork from `develop` branch;
2. Create branch: `git checkout -b feature/amazing-feature`;
3. Commit changes: `git commit -m 'Add amazing feature'`;
4. Create push in your branch: `git push origin feature/amazing-feature`;
5. Check [pre-commit](./.github/hooks/pre-commit) result. If you have any troubles you can't push anything.
6. Open `Pull Request`.

> We use git flow in branch architecture.</br>
> Create your pull request in `develop` branch.

## 📄 License
Distributed under the Apache 2.0 license. Details in the file [LICENSE](./LICENSE.txt).

## 👥 Author
Rhizome Dev Team - [GitHub](https://github.com/orgs/vazonhub/people).

---

_Inspired by the resilience of nature. Built for the freedom of speech._
