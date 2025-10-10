# syntax=docker/dockerfile:1

FROM rust:1.82 as builder
WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --locked && rm -rf src

# Copy migrations directory (required by sqlx::migrate! macro at compile time)
COPY migrations ./migrations

# Build application
COPY src ./src
RUN cargo build --release --locked

FROM debian:bookworm-slim as runtime
WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rust-basic-api /usr/local/bin/rust-basic-api

ENV RUST_LOG=info
EXPOSE 3000

CMD ["rust-basic-api"]
