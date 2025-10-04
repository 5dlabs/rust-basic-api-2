# syntax=docker/dockerfile:1

FROM rust:1.80 AS builder
WORKDIR /app

# Create a dummy project to leverage Docker layer caching for dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo fetch

# Copy full source and build release binary
COPY src ./src
RUN cargo build --release

FROM debian:bullseye-slim AS runtime
WORKDIR /app

RUN apt-get update \
    && apt-get install --no-install-recommends -y ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rust-basic-api /usr/local/bin/rust-basic-api

ENV RUST_LOG=info \
    SERVER_PORT=3000

EXPOSE 3000

CMD ["rust-basic-api"]
