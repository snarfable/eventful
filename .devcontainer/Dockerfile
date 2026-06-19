# Devcontainer image with tools for building and configuring Rust projects
FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
	&& apt-get install -y --no-install-recommends \
		build-essential \
		curl \
		ca-certificates \
		git \
		pkg-config \
		libssl-dev \
		openssl \
		cmake \
		clang \
		llvm \
		make \
		musl-tools \
		zlib1g-dev \
	&& rm -rf /var/lib/apt/lists/*

# Install rustup (stable) and common components
ENV RUSTUP_HOME=/usr/local/rustup \
	CARGO_HOME=/usr/local/cargo \
	PATH=/usr/local/cargo/bin:${PATH}

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --no-modify-path && \
	rustup default stable && \
	rustup component add rustfmt clippy rust-src && \
	cargo install cargo-edit cargo-watch

# Make cargo and rustup available system-wide for all users
RUN chmod -R a+rX ${CARGO_HOME} ${RUSTUP_HOME}

# Set a non-root user for development (optional)
ARG USERNAME=dev
ARG USER_UID=1000
ARG USER_GID=$USER_UID
RUN groupadd --gid $USER_GID $USERNAME || true \
	&& useradd -s /bin/bash --uid $USER_UID --gid $USER_GID -m $USERNAME || true

USER ${USERNAME}

WORKDIR /workspace

CMD ["/bin/bash"]
