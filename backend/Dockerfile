FROM rust:1.84.0-slim-bookworm AS builder

WORKDIR /app
RUN apt-get update && apt-get install -y libssl-dev pkg-config perl build-essential
RUN cargo install cargo-watch --version 8.5.3

COPY . .

# 依存ダウンロード兼ビルドキャッシュ用
RUN cargo build --release || true

CMD ["cargo", "watch", "-x", "run"]
