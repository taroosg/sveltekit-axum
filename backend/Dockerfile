# backend/Dockerfile
FROM rust:1.84.0-slim-bookworm AS builder

WORKDIR /app
COPY . .

# cargo-lambda 等使わない最小イメージなら下記でもOK
RUN cargo build --release

# 実行フェーズ
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/backend /app/backend

EXPOSE 3000
CMD ["/app/backend"]
