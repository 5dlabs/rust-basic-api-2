FROM rust:1.83 AS builder

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY src src

RUN cargo build --release --locked

FROM debian:bullseye-slim AS runtime

WORKDIR /app

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

ENV RUST_LOG=info

COPY --from=builder /app/target/release/rust-basic-api /usr/local/bin/rust-basic-api

EXPOSE 3000

CMD ["rust-basic-api"]
