FROM rust:1.85 AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM ubuntu:24.04

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/mongodb-init /app/mongodb-init

CMD ["./mongodb-init"]