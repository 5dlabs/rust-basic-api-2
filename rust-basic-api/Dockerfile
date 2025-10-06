FROM rust:1.82 as builder

WORKDIR /app

# Cache dependencies
COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update \
    && apt-get install --no-install-recommends -y ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/rust-basic-api ./rust-basic-api

ENV RUST_LOG=info \
    SERVER_PORT=3000

EXPOSE 3000

CMD ["./rust-basic-api"]
