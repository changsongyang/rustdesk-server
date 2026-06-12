# RustDesk Server - Podman 部署指南

## 概述

本文档提供 RustDesk Server 的 Podman 部署指南，重点介绍 rootless 模式部署，适用于社区版和商业版。

---

## 1. 环境准备

### 1.1 安装 Podman

**Debian/Ubuntu:**
```bash
# 更新系统
sudo apt update && sudo apt upgrade -y

# 安装 Podman
sudo apt install -y podman

# 验证安装
podman --version
```

**Red Hat/CentOS:**
```bash
# 安装 Podman
sudo dnf install -y podman

# 验证安装
podman --version
```

### 1.2 配置 Rootless 模式

```bash
# 安装 slirp4netns（用于 rootless 网络）
sudo apt install -y slirp4netns uidmap fuse-overlayfs

# 配置用户 ID 映射
echo "user.max_user_namespaces=28633" | sudo tee -a /etc/sysctl.conf
sudo sysctl --system

# 为用户启用 rootless
loginctl enable-linger $USER

# 验证 rootless 配置
podman info | grep -i rootless
```

---

## 2. 社区版部署（Rootless 模式）

### 2.1 创建数据目录

```bash
# 创建持久化数据目录
mkdir -p ~/rustdesk-server/data

# 设置权限
chmod 700 ~/rustdesk-server/data
```

### 2.2 启动 hbbs（Rendezvous Server）

```bash
# 启动 hbbs
podman run -d \
  --name rustdesk-hbbs \
  --restart always \
  -v ~/rustdesk-server/data:/data:Z \
  -p 21114:21114 \
  -p 21115:21115 \
  -p 21116:21116 \
  -p 21116:21116/udp \
  -p 21118:21118 \
  docker.io/rustdesk/rustdesk-server:latest \
  hbbs -r your-domain.com:21116

# 验证启动
podman ps
```

### 2.3 启动 hbbr（Relay Server）

```bash
# 启动 hbbr
podman run -d \
  --name rustdesk-hbbr \
  --restart always \
  -v ~/rustdesk-server/data:/data:Z \
  -p 21117:21117 \
  -p 21119:21119 \
  docker.io/rustdesk/rustdesk-server:latest \
  hbbr

# 验证启动
podman ps
```

---

## 3. 商业版部署（Rootless 模式）

### 3.1 创建数据目录

```bash
# 创建持久化数据目录
mkdir -p ~/rustdesk-pro/data

# 设置权限
chmod 700 ~/rustdesk-pro/data
```

### 3.2 启动商业版容器

```bash
podman run -d \
  --name rustdesk-pro \
  --restart always \
  -v ~/rustdesk-pro/data:/data:Z \
  -p 21114:21114 \
  -p 21115:21115 \
  -p 21116:21116 \
  -p 21116:21116/udp \
  -p 21117:21117 \
  -p 21118:21118 \
  -p 21119:21119 \
  -p 8000:8000 \
  docker.io/rustdesk/rustdesk-pro-server:latest

# 验证启动
podman ps
```

---

## 4. Podman 网络配置

### 4.1 创建自定义网络

```bash
# 创建自定义网络
podman network create rustdesk-net

# 查看网络列表
podman network ls

# 在自定义网络中启动容器
podman run -d \
  --name rustdesk-hbbs \
  --network rustdesk-net \
  --restart always \
  -v ~/rustdesk-server/data:/data:Z \
  -p 21114-21116:21114-21116 \
  -p 21116:21116/udp \
  -p 21118:21118 \
  docker.io/rustdesk/rustdesk-server:latest \
  hbbs -r your-domain.com:21116
```

### 4.2 端口转发配置

```bash
# 使用 podman port 查看端口映射
podman port rustdesk-hbbs

# 添加防火墙规则（root 权限）
sudo firewall-cmd --add-port=21114-21119/tcp --permanent
sudo firewall-cmd --add-port=21116/udp --permanent
sudo firewall-cmd --reload
```

---

## 5. 容器生命周期管理

### 5.1 启动/停止容器

```bash
# 启动容器
podman start rustdesk-hbbs rustdesk-hbbr

# 停止容器
podman stop rustdesk-hbbs rustdesk-hbbr

# 重启容器
podman restart rustdesk-hbbs rustdesk-hbbr

# 删除容器
podman rm rustdesk-hbbs rustdesk-hbbr
```

### 5.2 更新镜像

```bash
# 停止容器
podman stop rustdesk-hbbs rustdesk-hbbr

# 删除旧容器
podman rm rustdesk-hbbs rustdesk-hbbr

# 拉取最新镜像
podman pull docker.io/rustdesk/rustdesk-server:latest

# 重新启动容器
podman run -d \
  --name rustdesk-hbbs \
  --restart always \
  -v ~/rustdesk-server/data:/data:Z \
  -p 21114-21116:21114-21116 \
  -p 21116:21116/udp \
  -p 21118:21118 \
  docker.io/rustdesk/rustdesk-server:latest \
  hbbs -r your-domain.com:21116
```

### 5.3 自动更新

```bash
# 使用 podman auto-update
# 首先创建 systemd 服务

# 生成 systemd 服务文件
podman generate systemd --name rustdesk-hbbs > ~/.config/systemd/user/rustdesk-hbbs.service

# 启用服务
systemctl --user enable --now rustdesk-hbbs

# 配置自动更新（在容器标签中添加 io.containers.autoupdate=registry）
podman run -d \
  --name rustdesk-hbbs \
  --label io.containers.autoupdate=registry \
  -v ~/rustdesk-server/data:/data:Z \
  -p 21114-21116:21114-21116 \
  docker.io/rustdesk/rustdesk-server:latest \
  hbbs -r your-domain.com:21116
```

---

## 6. 健康检查配置

### 6.1 添加健康检查

```bash
# 使用 --health-cmd 添加健康检查
podman run -d \
  --name rustdesk-hbbs \
  --restart always \
  --health-cmd "curl -f http://localhost:21115/status || exit 1" \
  --health-interval 30s \
  --health-timeout 10s \
  --health-retries 3 \
  --health-start-period 40s \
  -v ~/rustdesk-server/data:/data:Z \
  -p 21114-21116:21114-21116 \
  -p 21116:21116/udp \
  docker.io/rustdesk/rustdesk-server:latest \
  hbbs -r your-domain.com:21116
```

### 6.2 检查健康状态

```bash
# 查看健康状态
podman inspect rustdesk-hbbs | grep -A 10 "Health"

# 或者使用 podman ps
podman ps --format "{{.Names}} {{.HealthStatus}}"
```

---

## 7. 构建自定义镜像

### 7.1 构建社区版镜像

```bash
# 克隆仓库
git clone https://github.com/rustdesk/rustdesk-server.git
cd rustdesk-server

# 构建镜像
podman build -t rustdesk-server:custom -f docker/Dockerfile .

# 验证镜像
podman images | grep rustdesk-server
```

### 7.2 构建商业版镜像

```bash
# 构建商业版镜像
podman build -t rustdesk-pro-server:custom -f commercial/docker/Dockerfile.s6 .

# 验证镜像
podman images | grep rustdesk-pro-server
```

---

## 8. 验证部署

### 8.1 检查容器状态

```bash
# 查看运行中的容器
podman ps

# 查看所有容器
podman ps -a

# 查看容器日志
podman logs rustdesk-hbbs

# 查看容器详细信息
podman inspect rustdesk-hbbs
```

### 8.2 验证服务端口

```bash
# 检查端口监听
ss -tlnp | grep 2111

# 使用 podman port
podman port rustdesk-hbbs
```

### 8.3 验证服务可用性

```bash
# 检查 hbbs 状态
curl http://localhost:21115/status

# 检查 hbbr 状态
curl http://localhost:21117/status
```

### 8.4 获取配置信息

```bash
# 获取公钥
podman exec rustdesk-hbbs cat /data/id_ed25519.pub
```

---

## 9. 备份和恢复

### 9.1 备份数据

```bash
# 停止容器
podman stop rustdesk-hbbs rustdesk-hbbr

# 创建备份
tar -czvf rustdesk-backup-$(date +%Y%m%d).tar.gz ~/rustdesk-server/data

# 启动容器
podman start rustdesk-hbbs rustdesk-hbbr
```

### 9.2 恢复数据

```bash
# 停止容器
podman stop rustdesk-hbbs rustdesk-hbbr

# 删除旧数据
rm -rf ~/rustdesk-server/data/*

# 恢复备份
tar -xzvf rustdesk-backup-YYYYMMDD.tar.gz -C ~/rustdesk-server/

# 启动容器
podman start rustdesk-hbbs rustdesk-hbbr
```

---

## 10. 故障排除

### 10.1 常见问题

| 问题 | 原因 | 解决方案 |
|------|------|----------|
| 容器无法启动 | 用户命名空间未配置 | 配置 user.max_user_namespaces |
| 端口无法访问 | rootless 端口限制 | 使用 sudo 或配置端口转发 |
| SELinux 权限问题 | 卷挂载权限 | 使用 :Z 或 :z 标签 |
| 网络不通 | slirp4netns 未安装 | 安装 slirp4netns |

### 10.2 查看日志

```bash
# 实时查看日志
podman logs -f rustdesk-hbbs

# 查看最近 100 行日志
podman logs --tail 100 rustdesk-hbbs
```

### 10.3 调试模式

```bash
# 进入容器
podman exec -it rustdesk-hbbs bash

# 检查容器内网络
podman exec rustdesk-hbbs ping rustdesk-hbbr
```

---

## 11. 版本兼容性

| RustDesk Server 版本 | Podman 版本要求 | 状态 |
|----------------------|-----------------|------|
| 1.2.0+ | 3.0+ | 支持 |
| 1.1.0+ | 2.0+ | 支持 |

---

**文档版本**: v1.0  
**适用产品**: RustDesk Server Community & Pro  
**最后更新**: 2026-06-12
