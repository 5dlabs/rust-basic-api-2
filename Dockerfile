# syntax=docker/dockerfile:1.6

FROM rust:1.80-slim AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install --no-install-recommends -y pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update \
    && apt-get install --no-install-recommends -y ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rust-basic-api /usr/local/bin/rust-basic-api

ENV RUST_LOG=info
EXPOSE 3000
CMD ["rust-basic-api"]
