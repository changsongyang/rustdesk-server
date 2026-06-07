# RustDesk Pro Server - Podman 部署指南

## 当前状态总结

| 项目 | 状态 |
|------|------|
| ✅ 本地编译（Windows） | **成功** - `./target/release/rustdesk-pro.exe` |
| ✅ 本地服务运行 | **正常** - API 健康检查通过 |
| ⚠️ Podman 完整构建 | **受限** - WSL Linux 环境 + GitHub 网络限制 |
| ✅ Podman 环境 | **正常** - Podman 5.8.1 + WSL (Linux 模式) |

### 🚫 当前环境限制

1. **Podman 运行模式**：WSL Linux 模式（无法使用 Windows 容器）
2. **网络限制**：无法访问 GitHub 下载 Git 依赖
3. **编译产物**：仅生成 Windows 二进制（无法在 Linux 容器中运行）

---

## 推荐方案（当前环境）

### 方案 1：直接使用本地编译的二进制（推荐 ⭐⭐⭐⭐⭐）

**最简单、最快的方案！**

```bash
# 1. 进入 commercial 目录
cd commercial

# 2. 直接运行服务
.\target\release\rustdesk-pro.exe serve

# 3. 浏览器访问
# http://localhost:8080/health
# http://localhost:8080/swagger
```

**特点**：
- ✅ 无需容器
- ✅ 已编译好，立即可用
- ✅ 性能最优

---

## Podman 构建方案（需要解决网络问题）

### 前置条件

1. **网络环境**：需要能访问 GitHub 和 crates.io
2. **关闭代理**：DevSidecar 等代理可能干扰容器网络

### 方案 2：在网络正常环境下构建

如果你有网络正常的环境（比如有 VPN）：

```bash
cd commercial/docker

# 使用完整 Dockerfile 构建
podman build -t rustdesk-pro-server:latest -f Dockerfile ..

# 运行容器
podman run -d \
  --name rustdesk-pro \
  -p 8080:8080 \
  -v rustdesk-data:/app/data \
  -v rustdesk-logs:/app/logs \
  -v rustdesk-keys:/app/keys \
  rustdesk-pro-server:latest
```

### 方案 3：使用 Podman Compose

```bash
cd commercial/docker

# 启动所有服务
podman-compose up -d

# 查看日志
podman-compose logs -f

# 停止服务
podman-compose down
```

---

## 完整构建流程（网络正常环境）

### 步骤 1：检查项目结构

```
commercial/
├── src/              # 源代码
├── libs/             # hbb_common 依赖
├── Cargo.toml        # 项目配置
├── target/           # 编译输出
├── docker/           # Docker 配置
│   ├── Dockerfile
│   ├── docker-compose.yml
│   └── podman-compose.yml
└── docs/             # 文档
```

### 步骤 2：构建镜像

```bash
cd commercial/docker

# 构建（确保网络正常）
podman build --no-cache -t rustdesk-pro-server:latest -f Dockerfile ..

# 验证镜像
podman images | grep rustdesk
```

### 步骤 3：运行容器

```bash
# 创建数据卷
podman volume create rustdesk-data
podman volume create rustdesk-logs
podman volume create rustdesk-keys

# 运行
podman run -d \
  --name rustdesk-pro \
  --restart unless-stopped \
  -p 8080:8080 \
  -v rustdesk-data:/app/data \
  -v rustdesk-logs:/app/logs \
  -v rustdesk-keys:/app/keys \
  rustdesk-pro-server:latest
```

### 步骤 4：验证部署

```bash
# 检查容器状态
podman ps

# 查看日志
podman logs -f rustdesk-pro

# 健康检查
curl http://localhost:8080/health

# 访问 API 文档
# 浏览器打开: http://localhost:8080/swagger
```

---

## 故障排查

### 问题 1：网络超时 / GitHub 连接失败

**解决方案**：

1. 使用 VPN 或代理
2. 配置 Git 代理：
   ```bash
   git config --global http.proxy http://proxy:port
   git config --global https.proxy http://proxy:port
   ```

### 问题 2：Cargo 依赖下载失败

**解决方案**：使用国内镜像源（已在 Dockerfile 中配置）

### 问题 3：容器启动失败

**排查步骤**：
```bash
# 查看日志
podman logs rustdesk-pro

# 检查端口占用
netstat -ano | findstr :8080

# 检查数据卷
podman volume ls
```

---

## 生产环境部署建议

### 使用 Podman Compose

```yaml
# podman-compose.yml 已包含完整配置
# 包含：
# - rustdesk-pro 服务
# - 数据持久化
# - 健康检查
# - 自动重启
```

### 安全加固

1. 使用非 root 用户（已配置）
2. 配置 HTTPS
3. 设置防火墙规则
4. 定期备份数据卷

---

## 文件说明

| 文件 | 说明 |
|------|------|
| `Dockerfile` | 标准多阶段构建（需要网络） |
| `Dockerfile.simple` | 简化版（使用预编译二进制） |
| `docker-compose.yml` | Docker Compose 配置 |
| `podman-compose.yml` | Podman Compose 配置 |

---

## 总结

| 方案 | 难度 | 速度 | 推荐度 |
|------|------|------|--------|
| 本地二进制直接运行 | ⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Podman 完整构建 | ⭐⭐⭐ | ⭐⭐ | ⭐⭐ |
| Podman Compose | ⭐⭐ | ⭐⭐ | ⭐⭐⭐ |

**当前最佳实践**：在 Windows 本地直接使用已编译的 `rustdesk-pro.exe`！
