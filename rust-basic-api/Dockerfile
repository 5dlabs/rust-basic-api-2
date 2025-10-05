# syntax=docker/dockerfile:1.7

FROM rust:1.83 as builder
WORKDIR /usr/src/app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Build application
COPY src ./src
RUN cargo build --release

FROM debian:bullseye-slim as runtime
WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

ENV RUST_LOG=info
COPY --from=builder /usr/src/app/target/release/rust-basic-api /usr/local/bin/rust-basic-api

EXPOSE 3000

CMD ["rust-basic-api"]
