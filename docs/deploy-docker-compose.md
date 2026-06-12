# RustDesk Server - Docker Compose 部署指南

## 概述

本文档提供 RustDesk Server 的 Docker Compose 编排部署指南，适用于社区版和商业版，包含完整的服务定义、健康检查和网络配置。

---

## 1. 环境准备

### 1.1 安装 Docker Compose

**Debian/Ubuntu:**
```bash
# 安装 Docker Compose
sudo apt install -y docker-compose-plugin

# 验证安装
docker compose version
```

**Red Hat/CentOS:**
```bash
# 安装 Docker Compose
sudo dnf install -y docker-compose-plugin

# 验证安装
docker compose version
```

**手动安装（适用于所有系统）:**
```bash
# 下载最新版本
DOCKER_COMPOSE_VERSION=$(curl -s https://api.github.com/repos/docker/compose/releases/latest | grep -oP '"tag_name": "\K(.*)(?=")')
sudo curl -L "https://github.com/docker/compose/releases/download/${DOCKER_COMPOSE_VERSION}/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose

# 添加执行权限
sudo chmod +x /usr/local/bin/docker-compose

# 创建软链接
sudo ln -s /usr/local/bin/docker-compose /usr/bin/docker-compose

# 验证安装
docker-compose version
```

---

## 2. 社区版部署

### 2.1 创建 docker-compose.yml

```bash
mkdir -p ~/rustdesk-server && cd ~/rustdesk-server
cat > docker-compose.yml << 'EOF'
version: '3.8'

services:
  hbbs:
    image: rustdesk/rustdesk-server:latest
    container_name: rustdesk-hbbs
    command: hbbs -r your-domain.com:21116
    volumes:
      - ./data:/data
    ports:
      - "21114:21114"
      - "21115:21115"
      - "21116:21116"
      - "21116:21116/udp"
      - "21118:21118"
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:21115/status"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    networks:
      - rustdesk-net

  hbbr:
    image: rustdesk/rustdesk-server:latest
    container_name: rustdesk-hbbr
    command: hbbr
    volumes:
      - ./data:/data
    ports:
      - "21117:21117"
      - "21119:21119"
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:21117/status"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    networks:
      - rustdesk-net

networks:
  rustdesk-net:
    driver: bridge
    name: rustdesk-net

volumes:
  rustdesk-data:
    driver: local
EOF
```

### 2.2 启动服务

```bash
# 启动服务（后台模式）
docker-compose up -d

# 查看服务状态
docker-compose ps

# 查看日志
docker-compose logs -f
```

### 2.3 验证部署

```bash
# 检查健康状态
docker-compose ps

# 获取公钥
docker-compose exec hbbs cat /data/id_ed25519.pub
```

---

## 3. 商业版部署

### 3.1 创建 docker-compose.yml

```bash
mkdir -p ~/rustdesk-pro && cd ~/rustdesk-pro
cat > docker-compose.yml << 'EOF'
version: '3.8'

services:
  rustdesk-pro:
    image: rustdesk/rustdesk-pro-server:latest
    container_name: rustdesk-pro
    volumes:
      - ./data:/data
    ports:
      - "21114:21114"
      - "21115:21115"
      - "21116:21116"
      - "21116:21116/udp"
      - "21117:21117"
      - "21118:21118"
      - "21119:21119"
      - "8000:8000"
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:21115/status"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 60s
    networks:
      - rustdesk-pro-net
    environment:
      - RUST_LOG=info

networks:
  rustdesk-pro-net:
    driver: bridge
    name: rustdesk-pro-net

volumes:
  rustdesk-pro-data:
    driver: local
EOF
```

### 3.2 启动商业版服务

```bash
# 启动服务
docker-compose up -d

# 查看服务状态
docker-compose ps

# 查看日志
docker-compose logs -f
```

---

## 4. 高级配置

### 4.1 使用环境变量

```yaml
services:
  hbbs:
    image: rustdesk/rustdesk-server:latest
    environment:
      - RUST_LOG=info
      - HBBS_RELAY_SERVER=your-relay-server.com:21117
      - HBBS_DB_PATH=/data/db
    # ... 其他配置
```

### 4.2 自定义网络配置

```yaml
networks:
  rustdesk-net:
    driver: bridge
    ipam:
      config:
        - subnet: 172.20.0.0/16
          gateway: 172.20.0.1
    name: rustdesk-net
```

### 4.3 添加反向代理

```yaml
services:
  nginx:
    image: nginx:alpine
    container_name: rustdesk-nginx
    volumes:
      - ./nginx/conf.d:/etc/nginx/conf.d
      - ./nginx/certs:/etc/nginx/certs
    ports:
      - "80:80"
      - "443:443"
    depends_on:
      - hbbs
      - hbbr
    restart: unless-stopped
    networks:
      - rustdesk-net
```

**Nginx 配置示例 (`nginx/conf.d/rustdesk.conf`):**
```nginx
server {
    listen 80;
    server_name your-domain.com;
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl;
    server_name your-domain.com;

    ssl_certificate /etc/nginx/certs/fullchain.pem;
    ssl_certificate_key /etc/nginx/certs/privkey.pem;

    location / {
        proxy_pass http://hbbs:21115;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
```

---

## 5. 服务管理

### 5.1 基本操作

```bash
# 启动服务
docker-compose up -d

# 停止服务
docker-compose down

# 重启服务
docker-compose restart

# 查看服务状态
docker-compose ps

# 查看日志
docker-compose logs -f
docker-compose logs -f hbbs

# 查看服务详细信息
docker-compose inspect hbbs
```

### 5.2 更新服务

```bash
# 停止服务
docker-compose down

# 拉取最新镜像
docker-compose pull

# 重新启动服务
docker-compose up -d
```

### 5.3 备份数据

```bash
# 创建备份目录
mkdir -p ~/backups

# 停止服务
docker-compose down

# 备份数据
tar -czvf ~/backups/rustdesk-backup-$(date +%Y%m%d).tar.gz ~/rustdesk-server/data

# 启动服务
docker-compose up -d
```

### 5.4 扩展服务

```bash
# 查看服务运行状态
docker-compose top

# 扩展服务（如需多实例）
docker-compose up -d --scale hbbs=2
```

---

## 6. 健康检查配置

### 6.1 自定义健康检查

```yaml
services:
  hbbs:
    healthcheck:
      test: ["CMD", "/usr/bin/healthcheck.sh"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
      disable: false
```

### 6.2 健康检查脚本示例

创建 `healthcheck.sh`:
```bash
#!/bin/bash
# 健康检查脚本

# 检查 hbbs 服务
if curl -f http://localhost:21115/status > /dev/null 2>&1; then
    exit 0
else
    exit 1
fi
```

---

## 7. 验证部署

### 7.1 检查服务状态

```bash
# 检查所有服务是否健康
docker-compose ps

# 检查特定服务
docker-compose ps hbbs

# 查看健康检查日志
docker inspect rustdesk-hbbs | grep -A 20 "Health"
```

### 7.2 验证端口监听

```bash
# 检查端口监听
netstat -tlnp | grep 2111

# 使用 ss 命令
ss -tlnp | grep 2111
```

### 7.3 测试 API

```bash
# 测试 hbbs API
curl http://localhost:21115/status

# 测试 hbbr API
curl http://localhost:21117/status
```

---

## 8. 故障排除

### 8.1 常见问题

| 问题 | 原因 | 解决方案 |
|------|------|----------|
| 服务无法启动 | 端口被占用 | 修改端口映射或释放端口 |
| 健康检查失败 | 服务未就绪 | 增加 `start_period` 时间 |
| 数据目录权限问题 | 用户权限不足 | 设置正确的目录权限 |
| 网络不通 | 网络配置问题 | 检查 Docker 网络配置 |

### 8.2 日志分析

```bash
# 查看所有日志
docker-compose logs -f

# 查看特定服务日志
docker-compose logs -f hbbs

# 查看最近 100 行日志
docker-compose logs --tail 100 hbbs

# 过滤错误日志
docker-compose logs hbbs | grep -i error
```

### 8.3 调试模式

```bash
# 启动服务并查看实时日志
docker-compose up

# 进入容器调试
docker-compose exec hbbs bash

# 检查容器内网络
docker-compose exec hbbs ping hbbr
```

---

## 9. 版本兼容性

| RustDesk Server 版本 | Docker Compose 版本要求 | 状态 |
|----------------------|------------------------|------|
| 1.2.0+ | 3.0+ | 支持 |
| 1.1.0+ | 2.0+ | 支持 |

---

**文档版本**: v1.0  
**适用产品**: RustDesk Server Community & Pro  
**最后更新**: 2026-06-12
