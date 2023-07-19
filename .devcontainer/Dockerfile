FROM rust:1.70.0

# target runtime
RUN rustup target add x86_64-unknown-linux-gnu

# linter and formatter
RUN rustup component add clippy
RUN rustup component add rustfmt

# for VS Code Remote Container
RUN rustup component add rust-analysis
RUN rustup component add rust-src
RUN rustup component add rls
RUN rustup install nightly-2023-03-03-x86_64-unknown-linux-gnu
RUN rustup target add wasm32-unknown-unknown

# utility for Cargo.toml
RUN cargo install cargo-edit

# for ink!
RUN cargo install cargo-contract --version 2.0.1 --force
RUN rustc --version

# nodejs
RUN apt update
RUN apt install -y curl
RUN curl -sL https://deb.nodesource.com/setup_18.x | bash
RUN apt install -y nodejs
RUN npm install -g yarn

# contracts-node
RUN apt install -y protobuf-compiler
RUN cargo install contracts-node --git https://github.com/paritytech/substrate-contracts-node.git --tag v0.25.0
