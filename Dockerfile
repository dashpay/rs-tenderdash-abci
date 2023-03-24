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
# Note: we add `--config net.git-fetch-with-cli=true` to address ARM build issue,
# see https://github.com/rust-lang/cargo/issues/10781#issuecomment-1441071052
RUN cargo add --config net.git-fetch-with-cli=true --git https://github.com/dashpay/rs-tenderdash-abci tenderdash-abci
RUN cargo build --config net.git-fetch-with-cli=true
