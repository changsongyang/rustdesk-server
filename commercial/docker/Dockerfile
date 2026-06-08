# RustDesk Pro Server - Dockerfile
# 默认使用 GitHub Actions 版本（官方源）
# 如需国内加速，请使用 Dockerfile.local
# Build: podman build -t rustdesk-pro-server:latest -f commercial/docker/Dockerfile .

# ==================== Build Stage ====================
# Use mirror for faster download in China
# FROM registry.cn-hangzhou.aliyuncs.com/mirror_rust/rust:1.88 as builder
FROM rust:1.96 as builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    git \
    && rm -rf /var/lib/apt/lists/*

# Create directory structure
RUN mkdir -p /build/commercial /build/libs

# Copy hbb_common submodule from root directory
COPY libs/hbb_common /build/libs/hbb_common

# Copy commercial Cargo files first for better caching
COPY commercial/Cargo.toml /build/commercial/Cargo.toml
COPY commercial/Cargo.lock /build/commercial/Cargo.lock

# Copy actual commercial source code
COPY commercial/src /build/commercial/src

# Build the application
WORKDIR /build/commercial
RUN cargo build --release

# ==================== Runtime Stage ====================
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    sqlite3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN useradd -m -s /bin/bash rustdesk

# Create necessary directories
RUN mkdir -p /app/data /app/logs /app/keys && \
    chown -R rustdesk:rustdesk /app

# Copy binary from builder
COPY --from=builder /build/commercial/target/release/rustdesk-pro /app/rustdesk-pro

# Copy default configuration
COPY commercial/.env.example /app/.env.example

# Set ownership
RUN chown -R rustdesk:rustdesk /app && \
    chmod +x /app/rustdesk-pro

# Switch to non-root user
USER rustdesk

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Set environment variables
ENV SERVER_PORT=8080 \
    SERVER_HOST=0.0.0.0 \
    DATABASE_URL=sqlite:./data/rustdesk_pro.db \
    PRO_DB_URL=./data/rustdesk_pro.db \
    LOG_LEVEL=info

# Run the application
CMD ["./rustdesk-pro", "serve"]
