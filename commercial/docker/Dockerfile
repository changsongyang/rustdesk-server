# RustDesk Pro Server - Dockerfile (标准版本)
# ==============================================
# 适用场景：通用部署、生产环境、手动构建
# 
# 特性：
# - 完整的多阶段构建流程
# - 非 root 用户运行
# - 健康检查支持
# - 标准端口暴露
# 
# 构建命令：
#   docker build -t rustdesk-pro-server:latest -f commercial/docker/Dockerfile .
# 
# 运行命令：
#   docker run -d -p 21114:21114 -p 21115:21115 -p 21116:21116 -p 21117:21117 \
#              -p 21118:21118 -p 21119:21119 -v ./data:/app/data \
#              rustdesk-pro-server:latest

# ==================== 构建阶段 ====================
FROM rust:1.96 AS builder

LABEL stage=builder
LABEL maintainer="RustDesk Team <info@rustdesk.com>"
LABEL description="RustDesk Pro Server Builder Image"

# 设置工作目录
WORKDIR /build

# 安装构建依赖
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
        git \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# 创建目录结构
RUN mkdir -p /build/commercial /build/libs

# 复制子模块（依赖层缓存优化）
COPY libs/hbb_common /build/libs/hbb_common

# 复制 Cargo 配置文件（依赖层缓存优化）
COPY commercial/Cargo.toml /build/commercial/Cargo.toml
COPY commercial/Cargo.lock /build/commercial/Cargo.lock

# 复制源代码
COPY commercial/src /build/commercial/src

# 构建应用
WORKDIR /build/commercial
RUN cargo build --release

# ==================== 运行阶段 ====================
FROM debian:bookworm-slim

LABEL maintainer="RustDesk Team <info@rustdesk.com>"
LABEL description="RustDesk Pro Server - Enterprise Remote Control Solution"
LABEL version="1.0.0"

# 设置工作目录
WORKDIR /app

# 安装运行时依赖
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl3 \
        sqlite3 \
        curl \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# 创建非 root 用户（安全最佳实践）
RUN useradd -m -s /bin/bash rustdesk

# 创建必要目录
RUN mkdir -p /app/data /app/logs /app/keys && \
    chown -R rustdesk:rustdesk /app

# 从构建阶段复制二进制文件
COPY --from=builder /build/commercial/target/release/rustdesk-pro /app/rustdesk-pro

# 设置权限
RUN chown -R rustdesk:rustdesk /app && \
    chmod +x /app/rustdesk-pro

# 切换到非 root 用户
USER rustdesk

# 暴露端口
# 21114: TCP 控制端口
# 21115: TCP 数据端口
# 21116: HTTP API 端口
# 21117: HTTPS API 端口
# 21118: WebSocket 端口
# 21119: Relay 端口
EXPOSE 21114 21115 21116 21117 21118 21119

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:21116/api/version || exit 1

# 环境变量（默认值）
ENV SERVER_PORT=21116 \
    SERVER_HOST=0.0.0.0 \
    DATABASE_URL=sqlite:./data/rustdesk_pro.db \
    PRO_DB_URL=./data/rustdesk_pro.db \
    LOG_LEVEL=info

# 启动命令
CMD ["./rustdesk-pro"]
