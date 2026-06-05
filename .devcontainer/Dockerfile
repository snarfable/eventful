FROM rust:1.78-bullseye

# Install common build tools and Rust components used in CI and local development.
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates git \
    && rustup component add rustfmt clippy \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace
