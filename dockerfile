# syntax=docker/dockerfile:1

# 定義全域參數
ARG RUST_VERSION=1.77
ARG APP_NAME=notify_grpc_server

# 建立應用程式構建階段
FROM rust:${RUST_VERSION}-slim-bookworm AS chef

# 安裝 cargo-chef 和所需的依賴項
RUN apt-get update -y && \
    apt-get install -y libssl-dev pkg-config protobuf-compiler && \
    cargo install cargo-chef

WORKDIR /app

# 準備食譜（準備依賴項）
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# 烹調階段（構建依賴項）
FROM chef AS cacher
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# 構建應用程式階段
FROM chef AS builder
COPY . .
COPY --from=cacher /app/target target
RUN cargo build --release

# 運行階段鏡像
FROM debian:bookworm-slim AS runtime
ARG APP_NAME

# 創建非特權用戶（用於運行應用程式）
ARG UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    appuser
USER appuser

# 安裝必要的依賴項
USER root
RUN apt-get update -y && \
    apt-get install libssl-dev -y && \
    apt-get install ca-certificates -y

# 複製可執行文件
COPY --from=builder /app/target/release/${APP_NAME} /app/server

# 暴露端口
EXPOSE 1680

# 啟動時運行的命令
CMD ["/app/server"]
