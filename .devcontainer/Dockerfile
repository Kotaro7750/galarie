# syntax=docker/dockerfile:1

ARG NODE_MAJOR=24
ARG NODE_FALLBACK=22
ARG USERNAME=vscode

FROM mcr.microsoft.com/devcontainers/base:ubuntu-24.04

ARG NODE_MAJOR
ARG NODE_FALLBACK
ARG USERNAME

ENV DEBIAN_FRONTEND=noninteractive

# Base dependencies + multimedia tooling
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      ca-certificates \
      curl \
      git \
      build-essential \
      pkg-config \
      libssl-dev \
      unzip \
      jq \
      ffmpeg \
      gifsicle \
      ripgrep && \
    rm -rf /var/lib/apt/lists/*

# Install Node.js (attempt Node ${NODE_MAJOR}, fallback to ${NODE_FALLBACK})
RUN set -eux; \
    if curl -fsSL "https://deb.nodesource.com/node_${NODE_MAJOR}.x/dists/nodistro/main/binary-amd64/Packages" -o /tmp/nodepkgs; then \
        curl -fsSL https://deb.nodesource.com/setup_${NODE_MAJOR}.x | bash -; \
    else \
        echo "Node ${NODE_MAJOR}.x not published yet. Falling back to ${NODE_FALLBACK}.x until release." >&2; \
        curl -fsSL https://deb.nodesource.com/setup_${NODE_FALLBACK}.x | bash -; \
    fi && \
    apt-get install -y nodejs && \
    npm install -g corepack && corepack enable && \
    rm -rf /var/lib/apt/lists/* /tmp/nodepkgs

# Install Rust toolchain for the non-root user
USER ${USERNAME}
ENV PATH="/home/${USERNAME}/.cargo/bin:${PATH}"
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable --profile default && \
    rustup component add rustfmt clippy && \
    echo 'source "$HOME/.cargo/env"' >> /home/${USERNAME}/.bashrc

USER root
RUN chown -R ${USERNAME}:${USERNAME} /home/${USERNAME}/.cargo /home/${USERNAME}/.rustup

WORKDIR /workspaces/galarie
