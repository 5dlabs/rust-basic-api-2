# syntax=docker/dockerfile:1.7

FROM rust:1.90 AS builder
WORKDIR /usr/src/app

# Optimize build by leveraging dependency caching
COPY Cargo.toml .
COPY Cargo.lock .
RUN mkdir src \
    && echo "fn main() {}" > src/main.rs \
    && cargo build --release \
    && rm -rf target/release/deps/rust_basic_api*

# Copy the actual source code and recompile
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update \
    && apt-get install --no-install-recommends -y ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/rust-basic-api /usr/local/bin/rust-basic-api

ENV RUST_LOG=info
EXPOSE 3000

CMD ["rust-basic-api"]
