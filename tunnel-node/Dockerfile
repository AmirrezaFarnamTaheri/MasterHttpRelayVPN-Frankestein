# syntax=docker/dockerfile:1

FROM rust:1.85-slim AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release --bin tunnel-node && \
    cp /app/target/release/tunnel-node /usr/local/bin/tunnel-node

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 1000 --no-create-home --shell /usr/sbin/nologin tunnel
COPY --from=builder /usr/local/bin/tunnel-node /usr/local/bin/tunnel-node
USER tunnel
ENV PORT=8080
EXPOSE 8080
ENTRYPOINT ["tunnel-node"]
