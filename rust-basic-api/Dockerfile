FROM rust:1.80 AS builder
WORKDIR /usr/src/app

# Reduce incremental compilation costs by caching dependencies
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bullseye-slim AS runtime
WORKDIR /app

RUN apt-get update \
    && apt-get install --no-install-recommends -y ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/rust-basic-api /app/rust-basic-api

ENV RUST_LOG=info
EXPOSE 3000

CMD ["/app/rust-basic-api"]
