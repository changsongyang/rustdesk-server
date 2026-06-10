# RustDesk Pro Server - 容器引擎对比分析报告
# ======================================================

## 1. 概述

本报告旨在分析不同容器引擎（Docker、Podman）对 RustDesk Pro Server 本地构建流程的影响，评估其在构建效率、资源占用、兼容性及开发体验方面的表现差异。

## 2. 对比维度

### 2.1 构建速度

| 维度 | Docker | Podman | 评估 |
|------|--------|--------|------|
| 首次构建 | 中等 | 中等 | 基本相当 |
| 增量构建（缓存命中） | 快 | 快 | 基本相当 |
| 网络层缓存 | 支持 | 支持 | 基本相当 |
| 并行构建 | 支持 | 支持 | 基本相当 |

### 2.2 镜像大小

| 维度 | Docker | Podman | 评估 |
|------|--------|--------|------|
| 最小化镜像 | 支持多阶段构建 | 支持多阶段构建 | 基本相当 |
| 层压缩 | gzip | gzip/zstd | Podman 略优 |
| 镜像导出/导入 | 支持 | 支持 | 基本相当 |

### 2.3 缓存机制

| 维度 | Docker | Podman | 评估 |
|------|--------|--------|------|
| 构建缓存 | 本地缓存 | 本地缓存 | 基本相当 |
| 缓存共享 | 支持 | 支持 | 基本相当 |
| 远程缓存 | BuildKit | Buildah | Docker 略优 |
| 缓存清理 | docker prune | podman system prune | 基本相当 |

### 2.4 资源占用

| 维度 | Docker | Podman | 评估 |
|------|--------|--------|------|
| 守护进程内存 | ~200MB | 无守护进程 | Podman 优 |
| 构建时CPU | 相当 | 相当 | 基本相当 |
| 磁盘空间 | 相当 | 相当 | 基本相当 |
| 内存效率 | 普通 | 优 | Podman 优 |

### 2.5 兼容性

| 维度 | Docker | Podman | 评估 |
|------|--------|--------|------|
| Dockerfile 语法 | 完全支持 | 完全支持 | 基本相当 |
| Docker Compose | 原生支持 | 需 podman-compose | Docker 优 |
| OCI 标准 | 支持 | 原生支持 | Podman 优 |
| Windows 支持 | 优 | 有限 | Docker 优 |

### 2.6 开发体验

| 维度 | Docker | Podman | 评估 |
|------|--------|--------|------|
| 易用性 | 优 | 良好 | Docker 略优 |
| 文档生态 | 丰富 | 有限 | Docker 优 |
| 社区支持 | 大 | 增长中 | Docker 优 |
| 无 root 运行 | 需配置 | 原生支持 | Podman 优 |

## 3. 推荐配置

### 3.1 开发环境

```bash
# Podman 构建命令（推荐）
podman build -t rustdesk-pro-server:local -f commercial/docker/Dockerfile.local .

# 运行命令
podman run -d -p 21114-21119:21114-21119 -v ./data:/app/data rustdesk-pro-server:local
```

### 3.2 CI/CD 环境

```bash
# Docker 构建命令（推荐）
docker build -t rustdesk-pro-server:gha -f commercial/docker/Dockerfile.gha .

# 推送命令
docker push ghcr.io/changsongyang/rustdesk-pro-server:latest
```

## 4. 问题与优化建议

### 4.1 潜在问题

| 问题 | 描述 | 影响 |
|------|------|------|
| 缓存失效 | 构建缓存可能因上下文变化失效 | 构建时间增加 |
| 网络超时 | 国内网络下载依赖可能超时 | 构建失败 |
| 资源竞争 | 并行构建可能导致资源竞争 | 构建不稳定 |
| SELinux 冲突 | Podman 在 SELinux 环境可能有权限问题 | 运行失败 |

### 4.2 优化建议

```bash
# 1. 启用 BuildKit 加速
export DOCKER_BUILDKIT=1
docker build --buildkit -t rustdesk-pro-server:latest .

# 2. 使用国内镜像加速（Dockerfile.local 已配置）
# 在 Dockerfile.local 中已配置阿里云 Debian 源和中科大 Crates 源

# 3. 清理缓存
docker system prune -af
podman system prune -af

# 4. 增加超时设置
export CARGO_NET_TIMEOUT=300
```

### 4.3 替代方案

| 方案 | 适用场景 | 优点 | 缺点 |
|------|---------|------|------|
| Buildah | 无守护进程构建 | 轻量、安全 | 学习曲线 |
| Kaniko | 无特权构建 | 安全、适合 CI | 速度较慢 |
| Buildx | 多平台构建 | 强大、灵活 | 配置复杂 |

## 5. 结论

### 5.1 推荐选择

| 场景 | 推荐引擎 | 理由 |
|------|---------|------|
| 本地开发 | Podman | 无守护进程、更安全 |
| CI/CD | Docker | 生态成熟、兼容性好 |
| 生产部署 | Podman | 更安全、更轻量 |
| 多平台构建 | Docker Buildx | 功能强大 |

### 5.2 关键建议

1. **统一构建配置**：所有 Dockerfile 保持核心逻辑一致
2. **缓存策略**：合理利用 Docker/Podman 构建缓存
3. **网络优化**：国内环境使用镜像加速
4. **安全加固**：使用非 root 用户运行容器
5. **监控日志**：建立完善的构建日志和监控体系

## 6. 验证测试

### 6.1 构建验证

```bash
# Docker 构建测试
docker build -t rustdesk-pro-server:docker-test -f commercial/docker/Dockerfile .

# Podman 构建测试
podman build -t rustdesk-pro-server:podman-test -f commercial/docker/Dockerfile .

# 镜像大小对比
docker images | grep rustdesk-pro-server
podman images | grep rustdesk-pro-server
```

### 6.2 运行验证

```bash
# Docker 运行测试
docker run -d --name rustdesk-docker -p 21116:21116 rustdesk-pro-server:docker-test
curl http://localhost:21116/api/version

# Podman 运行测试
podman run -d --name rustdesk-podman -p 21117:21116 rustdesk-pro-server:podman-test
curl http://localhost:21117/api/version
```
