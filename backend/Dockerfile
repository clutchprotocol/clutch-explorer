ARG RUST_VERSION=1.89
FROM rust:${RUST_VERSION}-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/clutch-explorer-backend
COPY Cargo.toml ./

RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -f target/release/clutch-explorer-backend target/release/clutch-explorer-backend.d && \
    rm -rf target/release/deps/clutch_explorer_backend* target/release/.fingerprint/clutch-explorer-backend-* && \
    rm -rf src

COPY src ./src
COPY config ./config
COPY migrations ./migrations
RUN cargo build --release

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
