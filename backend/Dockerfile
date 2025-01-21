FROM rust:1.84.0-slim-bookworm AS builder

WORKDIR /app
RUN apt-get update && apt-get install -y libssl-dev pkg-config perl build-essential
RUN cargo install cargo-watch --version 8.5.3
RUN cargo install sqlx-cli --features postgres
# COPY . .

# 依存ダウンロード兼ビルドキャッシュ用
# RUN cargo build --release || true

# FROM debian:bookworm-slim
# WORKDIR /app
# COPY --from=builder /app/target/release/backend /app/backend

# EXPOSE 3000
# CMD ["/app/backend"]

# CMD ["cargo", "watch", "-x", "run"]
