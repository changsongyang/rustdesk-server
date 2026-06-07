# RustDesk Pro Server 部署文档

## 目录

1. [环境要求](#1-环境要求)
2. [源码部署](#2-源码部署)
3. [二进制部署](#3-二进制部署)
4. [Docker 部署](#4-docker-部署)
5. [Podman 部署](#5-podman-部署)
6. [Kubernetes 部署](#6-kubernetes-部署)
7. [监控部署](#7-监控部署)
8. [Nginx 反向代理](#8-nginx-反向代理)
9. [升级指南](#9-升级指南)
10. [故障排查](#10-故障排查)
11. [备份与恢复](#11-备份与恢复)
12. [安全建议](#12-安全建议)

---

## 1. 环境要求

### 1.1 系统要求

| 资源 | 最小值 | 建议值 |
|------|--------|--------|
| CPU | 1 核 | 2 核 |
| 内存 | 512 MB | 2 GB |
| 存储 | 1 GB | 10 GB |

### 1.2 软件依赖

| 依赖 | 版本 | 说明 |
|------|------|------|
| Rust | 1.75+ | 源码编译 |
| Cargo | 1.75+ | Rust 包管理 |
| Docker | 20.10+ | Docker 部署 |
| Docker Compose | 1.29+ | 容器编排 |
| kubectl | 1.24+ | K8s 部署 |
| SQLite | 3.x | 数据库 |

---

## 2. 源码部署

### 2.1 克隆项目

```bash
git clone https://github.com/rustdesk/rustdesk-server.git
cd rustdesk-server/commercial
```

### 2.2 安装 Rust

```bash
# Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Windows (PowerShell)
Invoke-WebRequest https://win.rustup.rs/x86_64 -OutFile rustup-init.exe
.\rustup-init.exe -y
$env:Path += ";$HOME\.cargo\bin"
```

### 2.3 编译项目

```bash
# 编译 Release 版本
cargo build --release

# 编译时间较长，请耐心等待...
```

### 2.4 运行服务

```bash
# 创建必要目录
mkdir -p data logs keys

# 运行服务
./target/release/rustdesk-pro serve

# 指定参数运行
./target/release/rustdesk-pro serve --host 0.0.0.0 --port 8080 --db ./data/rustdesk_pro.db
```

### 2.5 验证部署

```bash
# 检查服务状态
curl http://localhost:8080/health

# 预期输出
{"status":"ok","version":"1.0.0","timestamp":"2024-01-01T00:00:00Z"}
```

---

## 3. 二进制部署

### 3.1 下载二进制包

```bash
# Linux (x86_64)
wget https://github.com/rustdesk/rustdesk-server/releases/download/v1.0.0/rustdesk-pro-server-v1.0.0-x86_64-linux.tar.gz
tar -xzf rustdesk-pro-server-v1.0.0-x86_64-linux.tar.gz
cd rustdesk-pro-server-v1.0.0

# Linux (ARM64)
wget https://github.com/rustdesk/rustdesk-server/releases/download/v1.0.0/rustdesk-pro-server-v1.0.0-aarch64-linux.tar.gz
tar -xzf rustdesk-pro-server-v1.0.0-aarch64-linux.tar.gz

# Windows
Invoke-WebRequest -Uri "https://github.com/rustdesk/rustdesk-server/releases/download/v1.0.0/rustdesk-pro-server-v1.0.0-windows-x86_64.zip" -OutFile rustdesk-pro-server.zip
Expand-Archive rustdesk-pro-server.zip -DestinationPath .
```

### 3.2 运行服务

```bash
# 创建目录
mkdir -p data logs keys

# 运行
./rustdesk-pro serve
```

### 3.3 配置为系统服务

#### 3.3.1 Systemd (Linux)

创建 `/etc/systemd/system/rustdesk-pro.service`：

```ini
[Unit]
Description=RustDesk Pro Server
After=network.target

[Service]
Type=simple
User=rustdesk
Group=rustdesk
WorkingDirectory=/opt/rustdesk-pro
ExecStart=/opt/rustdesk-pro/rustdesk-pro serve
Restart=always
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

启用服务：

```bash
# 创建用户
useradd -m -s /bin/bash rustdesk

# 设置权限
chown -R rustdesk:rustdesk /opt/rustdesk-pro

# 启用并启动服务
systemctl daemon-reload
systemctl enable rustdesk-pro
systemctl start rustdesk-pro

# 检查状态
systemctl status rustdesk-pro
```

#### 3.3.2 Windows 服务

```powershell
# 使用 NSSM 安装为 Windows 服务
nssm install "RustDesk Pro Server" "C:\rustdesk-pro\rustdesk-pro.exe" "serve"

# 启动服务
net start "RustDesk Pro Server"
```

---

## 4. Docker 部署

### 4.1 Docker Compose 部署

```bash
# 进入项目目录
cd rustdesk-server/commercial

# 使用预构建镜像
docker-compose -f docker/docker-compose.yml up -d

# 或构建本地镜像
docker-compose -f docker/docker-compose.yml build
docker-compose -f docker/docker-compose.yml up -d
```

### 4.2 Docker Compose 配置

```yaml
version: '3.8'

services:
  rustdesk-pro:
    image: rustdesk/rustdesk-pro-server:latest
    container_name: rustdesk-pro-server
    restart: unless-stopped
    ports:
      - "8080:8080"
    volumes:
      - rustdesk-data:/app/data
      - rustdesk-logs:/app/logs
      - rustdesk-keys:/app/keys
    environment:
      - SERVER_PORT=8080
      - DATABASE_URL=sqlite:./data/rustdesk_pro.db
      - JWT_SECRET=${JWT_SECRET:-your-secret-key}
      - LOG_LEVEL=info
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 3s
      retries: 3

volumes:
  rustdesk-data:
    driver: local
  rustdesk-logs:
    driver: local
  rustdesk-keys:
    driver: local
```

### 4.3 环境变量说明

| 变量 | 默认值 | 说明 |
|------|--------|------|
| SERVER_PORT | 8080 | 服务端口 |
| SERVER_HOST | 0.0.0.0 | 绑定地址 |
| DATABASE_URL | sqlite:./data/rustdesk_pro.db | 数据库连接 |
| JWT_SECRET | rustdesk-pro-jwt-secret | JWT 密钥 |
| JWT_EXPIRATION_HOURS | 24 | 令牌有效期 |
| LOG_LEVEL | info | 日志级别 |
| ADMIN_USERNAME | admin | 默认管理员 |
| ADMIN_PASSWORD | admin123 | 默认密码 |
| ADMIN_EMAIL | admin@rustdesk.local | 默认邮箱 |

### 4.4 验证 Docker 部署

```bash
# 查看容器状态
docker-compose -f docker/docker-compose.yml ps

# 查看日志
docker-compose -f docker/docker-compose.yml logs -f

# 健康检查
curl http://localhost:8080/health
```

---

## 5. Podman 部署

Podman 是 Docker 的无守护进程替代方案，提供更好的安全性和 rootless 模式支持。

### 5.1 安装 Podman

```bash
# RHEL/CentOS/Fedora
sudo dnf install podman podman-compose

# Ubuntu/Debian
sudo apt-get update
sudo apt-get install podman podman-compose

# Arch Linux
sudo pacman -S podman podman-compose

# macOS
brew install podman podman-compose
podman machine init
podman machine start
```

### 5.2 Rootless 模式配置（推荐）

Podman 支持无 root 权限运行容器，提供更高的安全性：

```bash
# 配置 subuid 和 subgid（如未配置）
echo "$USER:100000:65536" | sudo tee -a /etc/subuid
echo "$USER:100000:65536" | sudo tee -a /etc/subgid

# 启用 lingering（允许用户注销后容器继续运行）
sudo loginctl enable-linger $USER
```

### 5.3 使用 Podman Compose 部署

#### 5.3.1 创建 Podman Compose 配置

创建 `docker/podman-compose.yml`：

```yaml
version: '3.8'

services:
  rustdesk-pro:
    image: docker.io/rustdesk/rustdesk-pro-server:latest
    container_name: rustdesk-pro-server
    restart: unless-stopped
    ports:
      - "8080:8080"
    volumes:
      - rustdesk-data:/app/data
      - rustdesk-logs:/app/logs
      - rustdesk-keys:/app/keys
    environment:
      - SERVER_PORT=8080
      - DATABASE_URL=sqlite:./data/rustdesk_pro.db
      - JWT_SECRET=${JWT_SECRET:-your-secret-key}
      - LOG_LEVEL=info
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 3s
      retries: 3

volumes:
  rustdesk-data:
  rustdesk-logs:
  rustdesk-keys:
```

#### 5.3.2 启动服务

```bash
# 进入项目目录
cd rustdesk-server/commercial

# 使用 Podman Compose 启动
podman-compose -f docker/podman-compose.yml up -d

# 查看容器状态
podman-compose -f docker/podman-compose.yml ps

# 查看日志
podman-compose -f docker/podman-compose.yml logs -f
```

### 5.4 使用 Podman Pod 部署

Podman Pod 类似 Kubernetes Pod，可以将多个容器组合在一起：

```bash
# 创建 Pod
podman pod create --name rustdesk-pod -p 8080:8080

# 运行主服务容器
podman run -d \
  --pod rustdesk-pod \
  --name rustdesk-pro \
  -v rustdesk-data:/app/data \
  -v rustdesk-logs:/app/logs \
  -v rustdesk-keys:/app/keys \
  -e SERVER_PORT=8080 \
  -e DATABASE_URL=sqlite:./data/rustdesk_pro.db \
  -e JWT_SECRET=${JWT_SECRET} \
  docker.io/rustdesk/rustdesk-pro-server:latest

# 查看 Pod 状态
podman pod ps
podman pod inspect rustdesk-pod
```

### 5.5 生成 Systemd 服务

Podman 可以生成 Systemd 单元文件，实现开机自启：

```bash
# 生成 systemd 服务文件
podman generate systemd --name rustdesk-pro --files --new

# 移动到 systemd 目录
mkdir -p ~/.config/systemd/user/
mv container-rustdesk-pro.service ~/.config/systemd/user/

# 启用并启动服务
systemctl --user enable --now container-rustdesk-pro

# 查看服务状态
systemctl --user status container-rustdesk-pro
```

### 5.6 Podman vs Docker 对比

| 特性 | Docker | Podman |
|------|--------|--------|
| 守护进程 | 需要 dockerd | 无守护进程 |
| Root 权限 | 默认需要 | 支持 rootless |
| 安全性 | 一般 | 更高 |
| Pod 支持 | 无 | 原生支持 |
| Systemd 集成 | 手动配置 | 自动生成 |
| 命令兼容 | - | 完全兼容 |

### 5.7 常用 Podman 命令

```bash
# 列出容器
podman ps -a

# 查看容器日志
podman logs rustdesk-pro

# 进入容器
podman exec -it rustdesk-pro /bin/bash

# 停止容器
podman stop rustdesk-pro

# 删除容器
podman rm rustdesk-pro

# 查看镜像
podman images

# 拉取镜像
podman pull docker.io/rustdesk/rustdesk-pro-server:latest

# 构建镜像
podman build -t rustdesk-pro-server:latest -f docker/Dockerfile .

# 推送镜像
podman push docker.io/your-registry/rustdesk-pro-server:latest
```

### 5.8 验证 Podman 部署

```bash
# 检查容器状态
podman ps

# 健康检查
curl http://localhost:8080/health

# 查看容器详情
podman inspect rustdesk-pro
```

---

## 6. Kubernetes 部署

### 6.1 创建命名空间

```bash
kubectl create namespace rustdesk-pro
```

### 6.2 创建配置文件

#### 6.2.1 Secret 配置

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: rustdesk-pro-secrets
  namespace: rustdesk-pro
type: Opaque
data:
  jwt-secret: eW91ci1zZWNyZXQta2V5        # base64 encoded
```

#### 6.2.2 Deployment 配置

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rustdesk-pro
  namespace: rustdesk-pro
  labels:
    app: rustdesk-pro
spec:
  replicas: 2
  selector:
    matchLabels:
      app: rustdesk-pro
  template:
    metadata:
      labels:
        app: rustdesk-pro
    spec:
      containers:
        - name: rustdesk-pro
          image: rustdesk/rustdesk-pro-server:latest
          ports:
            - containerPort: 8080
          env:
            - name: SERVER_PORT
              value: "8080"
            - name: DATABASE_URL
              value: "sqlite:./data/rustdesk_pro.db"
            - name: JWT_SECRET
              valueFrom:
                secretKeyRef:
                  name: rustdesk-pro-secrets
                  key: jwt-secret
            - name: LOG_LEVEL
              value: "info"
          volumeMounts:
            - name: data
              mountPath: /app/data
            - name: logs
              mountPath: /app/logs
            - name: keys
              mountPath: /app/keys
          livenessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 30
            periodSeconds: 10
          readinessProbe:
            httpGet:
              path: /health
              port: 8080
            initialDelaySeconds: 10
            periodSeconds: 5
          resources:
            requests:
              cpu: "100m"
              memory: "256Mi"
            limits:
              cpu: "500m"
              memory: "512Mi"
      volumes:
        - name: data
          persistentVolumeClaim:
            claimName: rustdesk-data
        - name: logs
          persistentVolumeClaim:
            claimName: rustdesk-logs
        - name: keys
          persistentVolumeClaim:
            claimName: rustdesk-keys
```

#### 6.2.3 Service 配置

```yaml
apiVersion: v1
kind: Service
metadata:
  name: rustdesk-pro-service
  namespace: rustdesk-pro
spec:
  type: ClusterIP
  selector:
    app: rustdesk-pro
  ports:
    - protocol: TCP
      port: 8080
      targetPort: 8080
```

#### 6.2.4 Ingress 配置

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: rustdesk-pro-ingress
  namespace: rustdesk-pro
  annotations:
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/force-ssl-redirect: "true"
spec:
  tls:
    - hosts:
        - rustdesk.example.com
      secretName: rustdesk-tls
  rules:
    - host: rustdesk.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: rustdesk-pro-service
                port:
                  number: 8080
```

#### 6.2.5 PersistentVolumeClaim 配置

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: rustdesk-data
  namespace: rustdesk-pro
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi
```

### 6.3 部署到 Kubernetes

```bash
# 应用配置
kubectl apply -f k8s/

# 检查部署状态
kubectl get pods -n rustdesk-pro
kubectl get svc -n rustdesk-pro
kubectl get ingress -n rustdesk-pro

# 查看日志
kubectl logs -n rustdesk-pro -l app=rustdesk-pro -f
```

### 6.4 水平扩展

```bash
# 扩缩容
kubectl scale deployment rustdesk-pro --replicas=3 -n rustdesk-pro

# 自动扩缩容 (HPA)
kubectl autoscale deployment rustdesk-pro --min=2 --max=5 --cpu-percent=70 -n rustdesk-pro
```

---

## 7. 监控部署

### 7.1 启动监控栈

```bash
docker-compose -f monitoring/docker-compose.monitoring.yml up -d
```

### 7.2 Prometheus 配置

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'rustdesk-pro-server'
    static_configs:
      - targets: ['rustdesk-pro:8080']
    metrics_path: '/metrics'
    scrape_interval: 10s
```

### 6.3 访问监控服务

| 服务 | 地址 | 默认账号 |
|------|------|----------|
| Prometheus | http://localhost:9090 | - |
| Grafana | http://localhost:3000 | admin/admin |

---

## 8. Nginx 反向代理

### 8.1 HTTP 配置

```nginx
http {
    server {
        listen 80;
        server_name your-domain.com;

        location / {
            proxy_pass http://localhost:8080;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
        }
    }
}
```

### 8.2 HTTPS 配置

```nginx
http {
    server {
        listen 80;
        server_name your-domain.com;
        return 301 https://$host$request_uri;
    }

    server {
        listen 443 ssl;
        server_name your-domain.com;

        ssl_certificate /etc/nginx/ssl/fullchain.pem;
        ssl_certificate_key /etc/nginx/ssl/privkey.pem;
        ssl_protocols TLSv1.2 TLSv1.3;
        ssl_ciphers HIGH:!aNULL:!MD5;

        location / {
            proxy_pass http://localhost:8080;
            proxy_set_header Host $host;
            proxy_set_header X-Real-IP $remote_addr;
            proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
            proxy_set_header X-Forwarded-Proto $scheme;
            
            # WebSocket 支持
            proxy_http_version 1.1;
            proxy_set_header Upgrade $http_upgrade;
            proxy_set_header Connection "upgrade";
        }
    }
}
```

---

## 9. 升级指南

### 9.1 源码部署升级

```bash
# 拉取最新代码
git pull origin main

# 重新编译
cargo build --release

# 重启服务
systemctl restart rustdesk-pro
```

### 9.2 Docker 部署升级

```bash
# 拉取最新镜像
docker-compose -f docker/docker-compose.yml pull

# 重启服务
docker-compose -f docker/docker-compose.yml up -d
```

### 9.3 K8s 部署升级

```bash
# 更新镜像版本
kubectl set image deployment/rustdesk-pro rustdesk-pro=rustdesk/rustdesk-pro-server:v1.1.0 -n rustdesk-pro

# 或应用新配置
kubectl apply -f k8s/deployment.yaml
```

### 9.4 数据库迁移

```bash
# 如果有数据库迁移
docker-compose -f docker/docker-compose.yml exec rustdesk-pro migrate
```

---

## 10. 故障排查

### 9.1 常见问题

| 问题 | 原因 | 解决方案 |
|------|------|----------|
| 端口占用 | 8080 端口已被占用 | 修改 SERVER_PORT 环境变量 |
| 数据库连接失败 | 数据目录权限问题 | 检查目录权限 `chown -R rustdesk:rustdesk /app/data` |
| JWT 验证失败 | 密钥不一致 | 检查 JWT_SECRET 配置 |
| 服务启动失败 | 依赖缺失 | 安装依赖 `apt-get install libssl3 sqlite3` |
| Docker 网络问题 | 网络模式冲突 | 检查 Docker 网络配置 |
| K8s 服务不可达 | Service 配置错误 | 检查 Service 和 Endpoint |

### 10.2 日志查看

```bash
# 源码部署
tail -f logs/rustdesk-pro.log

# Docker 部署
docker-compose -f docker/docker-compose.yml logs -f

# K8s 部署
kubectl logs -n rustdesk-pro -l app=rustdesk-pro -f
```

### 10.3 调试模式

```bash
# 设置调试日志
export RUST_LOG=debug
./target/release/rustdesk-pro serve

# Docker 调试
docker-compose -f docker/docker-compose.yml up  # 非后台模式
```

### 9.4 数据库问题

```bash
# 检查数据库文件权限
ls -la data/

# 修复权限
chown -R rustdesk:rustdesk data/

# 验证数据库连接
sqlite3 data/rustdesk_pro.db "SELECT COUNT(*) FROM users;"
```

### 10.5 网络问题

```bash
# 检查端口监听
netstat -tlnp | grep 8080

# 测试网络连通性
curl -v http://localhost:8080/health

# 防火墙检查
ufw status
iptables -L
```

---

## 11. 备份与恢复

### 11.1 备份数据库

```bash
# 源码/二进制部署
cp data/rustdesk_pro.db data/rustdesk_pro.db.backup

# Docker 部署
docker cp rustdesk-pro-server:/app/data/rustdesk_pro.db ./backup/

# K8s 部署
kubectl cp rustdesk-pro-xxxx:/app/data/rustdesk_pro.db ./backup/
```

### 10.2 恢复数据库

```bash
# 停止服务
systemctl stop rustdesk-pro

# 恢复备份
cp backup/rustdesk_pro.db data/rustdesk_pro.db

# 修复权限
chown rustdesk:rustdesk data/rustdesk_pro.db

# 启动服务
systemctl start rustdesk-pro
```

### 11.3 自动化备份脚本

```bash
#!/bin/bash
BACKUP_DIR="/backup/rustdesk"
DATE=$(date +%Y%m%d_%H%M%S)

mkdir -p $BACKUP_DIR

# 备份数据库
cp /opt/rustdesk-pro/data/rustdesk_pro.db "$BACKUP_DIR/rustdesk_pro_$DATE.db"

# 保留最近7天备份
find $BACKUP_DIR -name "rustdesk_pro_*.db" -mtime +7 -delete

echo "Backup completed: $BACKUP_DIR/rustdesk_pro_$DATE.db"
```

---

## 12. 安全建议

### 11.1 生产环境配置

- 修改默认管理员密码
- 使用强 JWT_SECRET（至少32位）
- 启用 HTTPS
- 限制数据库文件权限（600）
- 使用专用数据库用户

### 11.2 防火墙配置

```bash
# Linux
ufw allow 443/tcp
ufw allow from 192.168.1.0/24 to any port 8080
ufw enable

# K8s
使用 NetworkPolicy 限制 Pod 访问
```

### 12.3 安全审计

- 定期检查日志
- 监控异常访问模式
- 定期更新依赖
- 启用审计日志功能

---

## 附录：部署检查清单

- [ ] 环境依赖已安装
- [ ] 数据库目录已创建并配置权限
- [ ] JWT_SECRET 已配置（生产环境）
- [ ] 防火墙规则已配置
- [ ] TLS 证书已配置（生产环境）
- [ ] 监控已启用
- [ ] 备份策略已配置
- [ ] 健康检查已配置
- [ ] 日志记录已配置