# This is an example Dockerfile, demonstrating build process of rs-tenderdash-abci

# We use Debian base image, as Alpine has some segmentation fault issue
FROM rust:bullseye

RUN apt-get update \
    &&  apt-get install --yes \
        protobuf-compiler \
        git \
        bash \
    && apt-get clean

# Create a dummy package
RUN cargo init /usr/src/abci-app
WORKDIR /usr/src/abci-app
RUN 

# Build the app, using extensive caching of dependencies.
# See https://doc.rust-lang.org/cargo/guide/cargo-home.html#caching-the-cargo-home-in-ci
RUN cargo add --git https://github.com/dashpay/rs-tenderdash-abci tenderdash-abci \
    && cargo build
