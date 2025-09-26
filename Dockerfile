FROM rust:1.70 as builder
WORKDIR /app
COPY Cargo.* ./
COPY src ./src
RUN cargo build --release

FROM debian:bullseye-slim
WORKDIR /app
COPY --from=builder /app/target/release/rust-basic-api /app/
EXPOSE 3000
CMD ["./rust-basic-api"]