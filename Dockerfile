# This is an example Dockerfile, demonstrating build process of rs-tenderdash-abci

# We use Debian base image, as Alpine has some segmentation fault issue
FROM rust:bullseye

RUN --mount=type=cache,sharing=locked,target=/var/lib/apt/lists \
    --mount=type=cache,sharing=locked,target=/var/cache/apt \
    rm -f /etc/apt/apt.conf.d/docker-clean && \
    apt-get update && \
    apt-get install --yes \
        build-essential \
        libclang-dev \
        libssl-dev \
        protobuf-compiler \
        git \
        bash

# Create a dummy package
RUN cargo init /usr/src/abci-app
WORKDIR /usr/src/abci-app
RUN cargo add --git https://github.com/dashpay/rs-tenderdash-abci --branch "proto-timestamp-milis" tenderdash-abci #a

# Build the app, using extensive caching of dependencies.
# See https://doc.rust-lang.org/cargo/guide/cargo-home.html#caching-the-cargo-home-in-ci
RUN --mount=type=cache,sharing=shared,target=${CARGO_HOME}/.crates.toml \
    --mount=type=cache,sharing=shared,target=${CARGO_HOME}/.crates2.json \
    --mount=type=cache,sharing=shared,target=${CARGO_HOME}/registry/index \
    --mount=type=cache,sharing=shared,target=${CARGO_HOME}/registry/cache \
    --mount=type=cache,sharing=shared,target=${CARGO_HOME}/git/db \
    cargo build
