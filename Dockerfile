# syntax=docker/dockerfile:1

FROM rust:1-slim-bookworm AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
        pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
        ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -m -u 10001 app
USER app
COPY --from=builder /app/target/release/web-push-testing-server /usr/local/bin/
EXPOSE 8080
CMD ["web-push-testing-server"]
