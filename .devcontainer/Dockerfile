FROM rust:1.70.0

# it makes faster cargo build
ENV CARGO_BUILD_TARGET_DIR=/tmp/target

# target runtime
RUN rustup target add x86_64-unknown-linux-gnu
RUN rustup toolchain install nightly-2023-03-05-aarch64-unknown-linux-gnu
RUN rustup component add rust-src --toolchain nightly-2023-03-05-aarch64-unknown-linux-gnu
RUN rustup target add wasm32-unknown-unknown

# linter and formatter
RUN rustup component add clippy
RUN rustup component add rustfmt

# for VS Code Remote Container
RUN rustup component add rust-analysis
RUN rustup component add rust-src
RUN rustup component add rls


# nodejs
RUN apt update
RUN apt install -y curl
RUN curl -sL https://deb.nodesource.com/setup_18.x | bash
RUN apt install -y nodejs
RUN npm install -g yarn

#protoc
RUN apt install -y unzip
RUN curl -Lo protoc.zip "https://github.com/protocolbuffers/protobuf/releases/latest/download/protoc-23.3-linux-x86_64.zip"
RUN unzip -q protoc.zip bin/protoc -d /usr/local
RUN chmod a+x /usr/local/bin/protoc

# for ink!
RUN cargo install cargo-contract --version 2.0.1 --force

# for ink! e2e-tests
RUN cargo install contracts-node --git https://github.com/paritytech/substrate-contracts-node.git --tag v0.26.0
