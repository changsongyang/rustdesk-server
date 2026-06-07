# RustDesk Pro Server - Docker 构建说明

## 文件说明

| 文件 | 用途 | 适用场景 |
|------|------|----------|
| `Dockerfile` | 默认版本（官方源） | 通用场景 |
| `Dockerfile.local` | 本地构建版本（国内加速） | 中国大陆网络环境 |
| `Dockerfile.gha` | GitHub Actions 版本 | CI/CD 自动化构建 |
| `Dockerfile.windows` | Windows 容器版本 | Windows 容器环境 |

## 快速使用

### 本地构建（国内加速）

```bash
# 使用国内镜像源加速
cd commercial/docker
podman build -t rustdesk-pro-server:local -f Dockerfile.local ..

# 或使用 Docker Compose
docker-compose -f docker-compose.local.yml up -d
```

### GitHub Actions 构建

```bash
# 使用官方源
cd commercial/docker
podman build -t rustdesk-pro-server:latest -f Dockerfile.gha ..

# 或使用 Docker Compose
docker-compose -f docker-compose.gha.yml up -d
```

### 默认构建

```bash
# 使用默认 Dockerfile（官方源）
cd commercial/docker
podman build -t rustdesk-pro-server:latest -f Dockerfile ..

# 或使用默认 Docker Compose
docker-compose up -d
```

## 配置说明

### Dockerfile.local（国内加速）

- 使用中国科学技术大学镜像源（USTC）
- 配置 `git-fetch-with-cli` 加速 Git 依赖下载
- 可选：配置 ghproxy 代理 GitHub 访问

### Dockerfile.gha（GitHub Actions）

- 使用官方 crates.io 源
- 无额外镜像配置
- 适用于 GitHub Actions 等国外 CI 环境

## 注意事项

1. **网络环境**：根据网络环境选择合适的 Dockerfile
2. **缓存利用**：首次构建较慢，后续会利用缓存层
3. **多阶段构建**：所有版本均使用多阶段构建优化镜像大小
