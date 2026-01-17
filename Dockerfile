# 构建阶段
FROM rust:1.75-slim AS builder

WORKDIR /usr/src/app
COPY . .

# 安装依赖并构建
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN cargo build --release -p api-server

# 运行阶段
FROM debian:bookworm-slim

WORKDIR /app
COPY --from=builder /usr/src/app/target/release/api-server /app/api-server
# 复制迁移文件 (假设需要)
COPY --from=builder /usr/src/app/db/migrations /app/migrations

# 暴露端口 (UDS 不占用端口，TCP 则需要)
EXPOSE 8080

CMD ["/app/api-server"]
