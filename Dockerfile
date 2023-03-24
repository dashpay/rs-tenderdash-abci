# This is an example Dockerfile, demonstrating build process of rs-tenderdash-abci

# We use Debian base image, as Alpine has some segmentation fault issue
FROM rust:bullseye

RUN apt-get --quiet update \
    &&  apt-get --quiet install --yes \
        protobuf-compiler \
        git \
        bash \
    && apt-get --quiet clean

RUN rustup install stable

# Create a dummy package
RUN cargo init /usr/src/abci-app
WORKDIR /usr/src/abci-app
RUN cargo add --git https://github.com/dashpay/rs-tenderdash-abci tenderdash-abci
RUN cargo build
