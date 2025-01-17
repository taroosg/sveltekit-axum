# backend/Dockerfile
FROM rust:1.84.0-slim-bookworm AS builder

# RUN apt-get update && apt-get install -y pkg-config libssl-dev

WORKDIR /app
COPY . .
RUN apt-get update && apt-get install -y pkg-config libssl-dev
RUN cargo build --release

# 実行フェーズ
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y libssl3

COPY --from=builder /app/target/release/backend /app/backend

EXPOSE 3000
CMD ["/app/backend"]
