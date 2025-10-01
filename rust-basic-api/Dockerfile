FROM rust:1.89-bullseye AS builder

WORKDIR /app

# Pre-fetch dependencies for faster incremental builds.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo fetch --locked
RUN rm -rf src

# Build the release binary.
COPY src ./src
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/rust-basic-api /usr/local/bin/rust-basic-api

ENV RUST_LOG=info \
    SERVER_PORT=3000 \
    DATABASE_MAX_CONNECTIONS=5

EXPOSE 3000

CMD ["rust-basic-api"]