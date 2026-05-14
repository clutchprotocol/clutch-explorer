ARG RUST_VERSION=1.89
FROM rust:${RUST_VERSION}-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Slow / flaky links to crates.io (common in Docker Desktop): more retries and a longer HTTP timeout.
ENV CARGO_NET_RETRY=10
ENV CARGO_HTTP_TIMEOUT=300

WORKDIR /usr/src/clutch-explorer-backend
COPY Cargo.toml ./

# Warm the dependency cache: stub lib + both binaries match the real layout ([lib] + src/main.rs + src/bin/indexer.rs).
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    mkdir -p src/bin && \
    printf '%s\n' '//! Dependency cache layer stub.' '#[allow(dead_code)]' 'pub mod explorer {}' > src/lib.rs && \
    echo 'fn main() {}' > src/main.rs && \
    echo 'fn main() {}' > src/bin/indexer.rs && \
    cargo build --release && \
    rm -f target/release/clutch-explorer-backend target/release/clutch-explorer-backend.d \
        target/release/indexer target/release/indexer.d && \
    rm -rf target/release/deps/clutch_explorer_backend* target/release/deps/indexer-* \
        target/release/.fingerprint/clutch-explorer-backend-* target/release/.fingerprint/indexer-* && \
    rm -rf src

COPY src ./src
COPY config ./config
COPY migrations ./migrations
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -g 1000 clutch && \
    useradd -r -u 1000 -g clutch -s /bin/sh clutch

WORKDIR /app
COPY --from=builder /usr/src/clutch-explorer-backend/target/release/clutch-explorer-backend /usr/local/bin/clutch-explorer-backend
COPY --from=builder /usr/src/clutch-explorer-backend/target/release/indexer /usr/local/bin/indexer
COPY --from=builder /usr/src/clutch-explorer-backend/config /app/config
RUN chmod +x /usr/local/bin/clutch-explorer-backend /usr/local/bin/indexer && chown -R clutch:clutch /app

USER clutch
EXPOSE 8088
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://127.0.0.1:8088/health || exit 1

CMD ["clutch-explorer-backend", "--env", "default"]
