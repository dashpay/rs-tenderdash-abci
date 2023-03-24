# This is an example Dockerfile, demonstrating build process of rs-tenderdash-abci

# We use Debian base image, as Alpine has some segmentation fault issue
FROM rust:bullseye

RUN --mount=type=cache,sharing=locked,target=/var/lib/apt/lists \
    --mount=type=cache,sharing=locked,target=/var/cache/apt \
    rm -f /etc/apt/apt.conf.d/docker-clean && \
    apt-get update --quiet && \
    apt-get install --quiet --yes \
        protobuf-compiler \
        git \
        wget \
        bash


#
# Install sccache - build cache for Rust.
# This is optional, but it will speed up the build process 
#
ARG SCCACHE_URL="https://github.com/mozilla/sccache/releases/download/v0.4.0/sccache-v0.4.0-x86_64-unknown-linux-musl.tar.gz"
RUN wget -q -O /tmp/sccache.tar.gz ${SCCACHE_URL} \
    && mkdir -p /tmp/sccache \
    && tar -z -C /tmp/sccache -xf /tmp/sccache.tar.gz \
    && mv /tmp/sccache/sccache*/sccache /usr/bin/sccache \
    && rm -r /tmp/sccache.tar.gz /tmp/sccache
# Set RUSTC_WRAPPER=/usr/bin/sccache to enable `sccache` caching.
ARG RUSTC_WRAPPER=/usr/bin/sccache

# Create a dummy package
RUN cargo init /usr/src/abci-app
WORKDIR /usr/src/abci-app

# Add tenderdash-abci as a dependency and build the package
#
# Two notes here:
# 1. All these --mount... are to cache reusable info between runs.
# See https://doc.rust-lang.org/cargo/guide/cargo-home.html#caching-the-cargo-home-in-ci
# 2. We add `--config net.git-fetch-with-cli=true` to address ARM build issue,
# see https://github.com/rust-lang/cargo/issues/10781#issuecomment-1441071052
RUN --mount=type=cache,sharing=shared,target=/root/.cache/sccache \
    --mount=type=cache,sharing=shared,target=${CARGO_HOME}/.crates.toml \
    --mount=type=cache,sharing=shared,target=${CARGO_HOME}/.crates2.json \
    --mount=type=cache,sharing=shared,target=${CARGO_HOME}/registry/index \
    --mount=type=cache,sharing=shared,target=${CARGO_HOME}/registry/cache \
    --mount=type=cache,sharing=shared,target=${CARGO_HOME}/git/db \
    cargo add --config net.git-fetch-with-cli=true --git https://github.com/dashpay/rs-tenderdash-abci tenderdash-abci && \
    cargo build --config net.git-fetch-with-cli=true
