FROM rust:1.70.0

# it makes faster cargo build
ENV CARGO_BUILD_TARGET_DIR=/tmp/target

# target runtime
RUN rustup target add x86_64-unknown-linux-gnu

# linter and formatter
RUN rustup component add clippy
RUN rustup component add rustfmt

# for VS Code Remote Container
RUN rustup component add rust-analysis
RUN rustup component add rust-src
RUN rustup component add rls

# utility for Cargo.toml
RUN cargo install cargo-edit

# for ink!
RUN cargo install cargo-contract --version 2.0.1 --force


# nodejs
RUN apt update
RUN apt install -y curl
RUN curl -sL https://deb.nodesource.com/setup_18.x | bash
RUN apt install -y nodejs
RUN npm install -g yarn
