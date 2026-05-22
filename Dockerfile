# syntax=docker/dockerfile:1.7

ARG NODE_VERSION=20
ARG RUST_VERSION=1.90

FROM node:${NODE_VERSION}-bookworm-slim AS frontend
RUN corepack enable
WORKDIR /app
COPY package.json pnpm-lock.yaml ./
RUN --mount=type=cache,id=marinara-pnpm,target=/root/.local/share/pnpm/store \
    pnpm install --frozen-lockfile
COPY index.html tsconfig.json tsconfig.node.json vite.config.ts ./
COPY public public
COPY src src
COPY src-tauri/capabilities src-tauri/capabilities
ENV VITE_TARGET=web
RUN pnpm build

FROM rust:${RUST_VERSION}-slim-bookworm AS rust-builder
RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /workspace
COPY Cargo.toml ./
COPY src-server src-server
COPY src-tauri/Cargo.toml src-tauri/Cargo.toml
COPY src-tauri/crates src-tauri/crates
RUN mkdir -p src-tauri/src && echo "fn main() {}" > src-tauri/src/main.rs && echo "" > src-tauri/src/lib.rs
# EasyPanel passes --build-arg GIT_SHA=<commit> on every deploy. Declaring it
# here makes the cargo-build RUN layer's cache key depend on the commit, so
# docker buildx can't reuse a stale binary across commits — even when the
# COPY layers' content-hash happens to match a prior build's. The cargo cache
# mount below still lets cargo do an incremental compile (only changed crates
# recompile, the registry stays warm), so this costs only the time the binary
# actually needs to relink — typically tens of seconds, not a full rebuild.
ARG GIT_SHA=unknown
# The cargo-target cache mount preserves /workspace/target across builds. That
# keeps upstream deps (axum, tokio, autoagents, image, etc.) compiled across
# deploys, which is great. But it ALSO preserves stale .rmeta/.rlib files for
# our own workspace crates, and cargo's incremental fingerprint sometimes
# fails to detect when a new module file lands in one of them — e.g. a fresh
# `src-tauri/crates/handlers/src/<new>.rs` plus a `pub mod <new>;` in lib.rs.
# When that happens, cargo links marinara-server against an out-of-date
# marinara_handlers.rmeta and either reuses an old binary (no rebuild) or
# emits "could not find <module> in marinara_handlers" errors (rebuild that
# trips over the stale cache).
#
# Fix: cargo clean --release -p <each crate we own> before the build. This
# drops our crates' .rmeta/.rlib/.d files so cargo regenerates them from
# current source. Upstream deps stay cached. The `|| true` accommodates the
# very first build, where `cargo clean -p <name>` errors because the package
# isn't compiled yet.
RUN --mount=type=cache,id=marinara-cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=marinara-cargo-target,target=/workspace/target \
    echo "Building marinara-server at commit ${GIT_SHA}" && \
    cargo clean --release \
        -p marinara-core \
        -p marinara-security \
        -p marinara-storage \
        -p marinara-assets \
        -p marinara-handlers \
        -p marinara-server 2>/dev/null || true && \
    cargo build --release --bin marinara-server && \
    cp target/release/marinara-server /usr/local/bin/marinara-server

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=frontend /app/dist /app/dist
COPY --from=rust-builder /usr/local/bin/marinara-server /usr/local/bin/marinara-server
ENV MARINARA_FRONTEND_DIR=/app/dist
ENV MARINARA_DATA_DIR=/data
ENV MARINARA_ADDR=0.0.0.0:8080
VOLUME ["/data"]
EXPOSE 8080
CMD ["marinara-server"]
