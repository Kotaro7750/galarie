# syntax=docker/dockerfile:1
ARG APP_USER=galarie

########################################
# Base builder image
########################################
FROM rust:1.90-bullseye AS backend-builder
ARG APP_USER
WORKDIR /tmp/build-backend

# Cache dependencies
COPY backend/Cargo.toml backend/Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Build application
COPY backend/src ./src
RUN ["cargo", "build", "--release"]

########################################
# Frontend builder image for frontend
########################################
FROM node:24-alpine AS frontend-builder
WORKDIR /tmp/build-frontend

COPY frontend/package.json frontend/package-lock.json ./
RUN ["npm", "ci"]

COPY frontend/ ./
ENV VITE_BASE_PATH=/ui
RUN ["npm", "run", "build"]

########################################
# Production runtime image
########################################
FROM debian:bookworm-slim AS prod-runtime
ARG APP_USER

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates ffmpeg gifsicle \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -m -s /usr/sbin/nologin "$APP_USER"

ENV GALARIE_MEDIA_ROOT=/data/media \
    GALARIE_CACHE_DIR=/data/cache \
    RUST_LOG=info \
    OTEL_SERVICE_NAME=galarie-backend \
    GALARIE_ENV=production

WORKDIR /app
RUN mkdir -p "$GALARIE_MEDIA_ROOT" "$GALARIE_CACHE_DIR" && chown -R "$APP_USER:$APP_USER" /data

COPY ./startup.sh /app/startup.sh
RUN ["chmod", "+x", "/app/startup.sh"]
COPY --from=backend-builder /tmp/build-backend/target/release/galarie-backend galarie-backend
COPY --from=frontend-builder /tmp/build-frontend/dist /app/frontend

RUN chown -R $APP_USER:$APP_USER /app

USER $APP_USER
EXPOSE 8080
ENTRYPOINT ["/app/startup.sh"]
CMD []

########################################
# Devcontainer image (extends prod runtime)
########################################
FROM backend-builder AS devcontainer
ARG APP_USER

RUN rm -rf /tmp/build-backend

USER root
RUN apt-get update \
    && apt-get install -y --no-install-recommends bash git sudo curl unzip supervisor ffmpeg gifsicle \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -m -s /bin/bash "$APP_USER" \
    && echo "$APP_USER ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/$APP_USER \
    && chown -R $APP_USER:$APP_USER /usr/local/cargo /usr/local/rustup

# Install Node (matching devcontainer requirements)
ARG NODE_VERSION=24
RUN curl -fsSL https://deb.nodesource.com/setup_${NODE_VERSION}.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /galarie-content \
    && chown -R $APP_USER:$APP_USER /galarie-content

USER $APP_USER
ENV CARGO_HOME=/usr/local/cargo RUSTUP_HOME=/usr/local/rustup PATH=/usr/local/cargo/bin:$PATH
WORKDIR /workspace
ENTRYPOINT ["sudo", "-E", "supervisord", "-c", "/etc/supervisor/supervisord.conf"]
