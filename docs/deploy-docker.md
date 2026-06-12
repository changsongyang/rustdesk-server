# RustDesk Server - Docker 部署指南

## 概述

本文档提供 RustDesk Server 的 Docker 单容器部署指南，适用于社区版和商业版。

---

## 1. 环境准备

### 1.1 安装 Docker

**Debian/Ubuntu:**
```bash
# 更新系统
sudo apt update && sudo apt upgrade -y

# 安装依赖
sudo apt install -y apt-transport-https ca-certificates curl software-properties-common

# 添加 Docker GPG 密钥
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg

# 添加 Docker 仓库
echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

# 安装 Docker
sudo apt update && sudo apt install -y docker-ce docker-ce-cli containerd.io

# 启动 Docker 服务
sudo systemctl enable --now docker

# 添加用户到 docker 组（可选）
sudo usermod -aG docker $USER
newgrp docker
```

**Red Hat/CentOS:**
```bash
# 安装依赖
sudo dnf install -y yum-utils device-mapper-persistent-data lvm2

# 添加 Docker 仓库
sudo yum-config-manager --add-repo https://download.docker.com/linux/centos/docker-ce.repo

# 安装 Docker
sudo dnf install -y docker-ce docker-ce-cli containerd.io

# 启动 Docker 服务
sudo systemctl enable --now docker

# 添加用户到 docker 组（可选）
sudo usermod -aG docker $USER
newgrp docker
```

### 1.2 验证 Docker 安装

```bash
# 检查 Docker 版本
docker --version

# 运行测试容器
docker run --rm hello-world
```

---

## 2. 社区版部署

### 2.1 拉取官方镜像

```bash
# 拉取最新版本
docker pull rustdesk/rustdesk-server:latest

# 或者指定特定版本
docker pull rustdesk/rustdesk-server:1.2.3
```

### 2.2 创建数据目录

```bash
# 创建持久化数据目录
mkdir -p ~/rustdesk-server/data

# 设置权限
chown -R 1000:1000 ~/rustdesk-server/data
```

### 2.3 启动容器

```bash
# 启动 hbbs (Rendezvous Server)
docker run -d \
  --name rustdesk-hbbs \
  --restart unless-stopped \
  -v ~/rustdesk-server/data:/data \
  -p 21114:21114 \
  -p 21115:21115 \
  -p 21116:21116 \
  -p 21116:21116/udp \
  -p 21118:21118 \
  rustdesk/rustdesk-server:latest \
  hbbs -r your-domain.com:21116

# 启动 hbbr (Relay Server)
docker run -d \
  --name rustdesk-hbbr \
  --restart unless-stopped \
  -v ~/rustdesk-server/data:/data \
  -p 21117:21117 \
  -p 21119:21119 \
  rustdesk/rustdesk-server:latest \
  hbbr
```

**端口说明:**

| 端口 | 协议 | 用途 |
|------|------|------|
| 21114 | TCP | 心跳服务 |
| 21115 | TCP | API 服务 |
| 21116 | TCP/UDP | Rendezvous 服务 |
| 21117 | TCP | Relay 服务 |
| 21118 | TCP | 备用端口 |
| 21119 | TCP | 备用端口 |

---

## 3. 商业版部署

### 3.1 拉取商业版镜像

```bash
# 拉取商业版最新版本
docker pull rustdesk/rustdesk-pro-server:latest

# 或者指定特定版本
docker pull rustdesk/rustdesk-pro-server:1.2.3
```

### 3.2 创建数据目录

```bash
# 创建持久化数据目录
mkdir -p ~/rustdesk-pro/data

# 设置权限
chown -R 1000:1000 ~/rustdesk-pro/data
```

### 3.3 启动商业版容器

```bash
docker run -d \
  --name rustdesk-pro \
  --restart unless-stopped \
  -v ~/rustdesk-pro/data:/data \
  -p 21114:21114 \
  -p 21115:21115 \
  -p 21116:21116 \
  -p 21116:21116/udp \
  -p 21117:21117 \
  -p 21118:21118 \
  -p 21119:21119 \
  -p 8000:8000 \
  rustdesk/rustdesk-pro-server:latest
```

**商业版额外端口:**

| 端口 | 用途 |
|------|------|
| 8000 | Web 管理界面 |

---

## 4. 自定义配置

### 4.1 使用环境变量

```bash
docker run -d \
  --name rustdesk-hbbs \
  --restart unless-stopped \
  -v ~/rustdesk-server/data:/data \
  -p 21114-21119:21114-21119 \
  -p 21116:21116/udp \
  -e RUST_LOG=info \
  -e HBBS_RELAY_SERVER=your-relay-server.com:21117 \
  rustdesk/rustdesk-server:latest \
  hbbs -r your-domain.com:21116
```

### 4.2 自定义网络

```bash
# 创建自定义网络
docker network create rustdesk-net

# 在自定义网络中启动容器
docker run -d \
  --name rustdesk-hbbs \
  --network rustdesk-net \
  --restart unless-stopped \
  -v ~/rustdesk-server/data:/data \
  -p 21114-21119:21114-21119 \
  -p 21116:21116/udp \
  rustdesk/rustdesk-server:latest \
  hbbs -r your-domain.com:21116
```

---

## 5. 构建自定义镜像

### 5.1 克隆仓库

```bash
git clone https://github.com/rustdesk/rustdesk-server.git
cd rustdesk-server
```

### 5.2 构建社区版镜像

```bash
# 构建 Docker 镜像
docker build -t rustdesk-server:custom -f docker/Dockerfile .

# 验证镜像
docker images | grep rustdesk-server
```

### 5.3 构建商业版镜像

```bash
# 构建商业版 Docker 镜像
docker build -t rustdesk-pro-server:custom -f commercial/docker/Dockerfile.s6 .

# 验证镜像
docker images | grep rustdesk-pro-server
```

---

## 6. 验证部署

### 6.1 检查容器状态

```bash
# 查看运行中的容器
docker ps

# 查看容器日志
docker logs rustdesk-hbbs
docker logs rustdesk-hbbr

# 查看容器详细信息
docker inspect rustdesk-hbbs
```

### 6.2 验证服务端口

```bash
# 检查端口监听
netstat -tlnp | grep 2111

# 或者使用 ss
ss -tlnp | grep 2111
```

### 6.3 验证服务可用性

```bash
# 检查 hbbs 状态
curl http://localhost:21115/status

# 检查 hbbr 状态
curl http://localhost:21117/status
```

### 6.4 获取配置信息

```bash
# 获取公钥（客户端需要此信息）
docker exec rustdesk-hbbs cat /data/id_ed25519.pub
```

---

## 7. 容器管理

### 7.1 启动/停止容器

```bash
# 启动容器
docker start rustdesk-hbbs rustdesk-hbbr

# 停止容器
docker stop rustdesk-hbbs rustdesk-hbbr

# 重启容器
docker restart rustdesk-hbbs rustdesk-hbbr
```

### 7.2 更新镜像

```bash
# 停止容器
docker stop rustdesk-hbbs rustdesk-hbbr

# 拉取最新镜像
docker pull rustdesk/rustdesk-server:latest

# 删除旧容器
docker rm rustdesk-hbbs rustdesk-hbbr

# 重新启动容器（使用相同的命令）
docker run -d \
  --name rustdesk-hbbs \
  --restart unless-stopped \
  -v ~/rustdesk-server/data:/data \
  -p 21114-21119:21114-21119 \
  -p 21116:21116/udp \
  rustdesk/rustdesk-server:latest \
  hbbs -r your-domain.com:21116
```

### 7.3 备份数据

```bash
# 停止容器
docker stop rustdesk-hbbs rustdesk-hbbr

# 创建备份
tar -czvf rustdesk-server-backup-$(date +%Y%m%d).tar.gz ~/rustdesk-server/data

# 启动容器
docker start rustdesk-hbbs rustdesk-hbbr
```

---

## 8. 故障排除

### 8.1 常见问题

| 问题 | 原因 | 解决方案 |
|------|------|----------|
| 容器无法启动 | 端口被占用 | 使用 `netstat -tlnp` 检查并释放端口 |
| 客户端无法连接 | 防火墙阻止 | 确保防火墙允许 21114-21119 端口 |
| 数据未持久化 | 目录权限问题 | 确保数据目录权限正确 |
| 日志显示错误 | 配置错误 | 检查环境变量和启动参数 |

### 8.2 查看日志

```bash
# 实时查看日志
docker logs -f rustdesk-hbbs

# 查看最近 100 行日志
docker logs --tail 100 rustdesk-hbbs
```

---

## 9. 版本兼容性

| RustDesk Server 版本 | Docker 版本要求 | 状态 |
|----------------------|-----------------|------|
| 1.2.0+ | 20.10+ | 支持 |
| 1.1.0+ | 19.03+ | 支持 |

---

**文档版本**: v1.0  
**适用产品**: RustDesk Server Community & Pro  
**最后更新**: 2026-06-12
