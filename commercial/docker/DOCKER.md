# RustDesk Pro Server - Docker 部署指南

## 快速开始

### 1. 使用 Docker Compose 部署（推荐）

```bash
# 克隆仓库
cd rustdesk-server/commercial

# 创建环境变量文件
cp .env.example .env

# 编辑环境变量（可选）
# vim .env

# 启动服务
docker-compose up -d

# 查看日志
docker-compose logs -f

# 停止服务
docker-compose down
```

### 2. 使用 Docker 直接部署

```bash
# 构建镜像
docker build -t rustdesk-pro-server:latest .

# 运行容器
docker run -d \
  --name rustdesk-pro-server \
  -p 8080:8080 \
  -v rustdesk-data:/app/data \
  -v rustdesk-logs:/app/logs \
  -v rustdesk-keys:/app/keys \
  -e JWT_SECRET=your-secret-key \
  -e ADMIN_PASSWORD=your-admin-password \
  rustdesk-pro-server:latest

# 查看日志
docker logs -f rustdesk-pro-server

# 停止容器
docker stop rustdesk-pro-server
```

## 环境变量配置

| 变量名 | 默认值 | 说明 |
|--------|--------|------|
| SERVER_PORT | 8080 | 服务端口 |
| SERVER_HOST | 0.0.0.0 | 监听地址 |
| DATABASE_URL | sqlite:./data/rustdesk_pro.db | 数据库连接URL |
| PRO_DB_URL | ./data/rustdesk_pro.db | 数据库文件路径 |
| JWT_SECRET | rustdesk-pro-jwt-secret-key... | JWT密钥 |
| JWT_EXPIRATION_HOURS | 24 | JWT过期时间（小时） |
| LOG_LEVEL | info | 日志级别 |
| ADMIN_USERNAME | admin | 管理员用户名 |
| ADMIN_PASSWORD | admin123 | 管理员密码 |
| ADMIN_EMAIL | admin@rustdesk.local | 管理员邮箱 |

## 数据持久化

Docker Compose 会自动创建以下卷：

- `rustdesk-data`: 数据库文件
- `rustdesk-logs`: 日志文件
- `rustdesk-keys`: 许可证密钥

查看卷位置：
```bash
docker volume inspect rustdesk-data
```

备份数据：
```bash
# 备份数据库
docker cp rustdesk-pro-server:/app/data/rustdesk_pro.db ./backup/

# 备份整个数据目录
docker run --rm -v rustdesk-data:/data -v $(pwd):/backup alpine tar czf /backup/data-backup.tar.gz /data
```

恢复数据：
```bash
# 恢复数据库
docker cp ./backup/rustdesk_pro.db rustdesk-pro-server:/app/data/

# 恢复整个数据目录
docker run --rm -v rustdesk-data:/data -v $(pwd):/backup alpine tar xzf /backup/data-backup.tar.gz -C /
```

## 生产环境部署

### 1. 使用 HTTPS（推荐）

创建 SSL 证书：
```bash
mkdir -p ssl
# 将你的证书文件放入 ssl/ 目录
# ssl/cert.pem - SSL 证书
# ssl/key.pem - SSL 私钥
```

修改 `nginx.conf`，取消 HTTPS 部分的注释。

修改 `docker-compose.yml`，取消 nginx 服务的注释。

### 2. 性能优化

调整 Docker Compose 配置：
```yaml
services:
  rustdesk-pro:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 2G
        reservations:
          cpus: '1'
          memory: 1G
```

### 3. 日志管理

配置日志轮转：
```yaml
services:
  rustdesk-pro:
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

## 健康检查

Docker 容器包含健康检查，可通过以下命令查看：
```bash
docker inspect --format='{{.State.Health.Status}}' rustdesk-pro-server
```

## 故障排查

### 查看容器日志
```bash
docker logs -f rustdesk-pro-server
```

### 进入容器调试
```bash
docker exec -it rustdesk-pro-server /bin/bash
```

### 检查数据库
```bash
docker exec -it rustdesk-pro-server sqlite3 /app/data/rustdesk_pro.db
```

### 重启服务
```bash
docker-compose restart
```

### 完全重置
```bash
docker-compose down -v
docker-compose up -d
```

## API 测试

服务启动后，可以通过以下命令测试 API：

```bash
# 健康检查
curl http://localhost:8080/health

# 用户登录
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}'

# 创建用户（需要先登录获取 token）
curl -X POST http://localhost:8080/api/users \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"username":"user1","email":"user1@example.com","password":"password123","role":"viewer"}'

# 列出设备
curl -X GET http://localhost:8080/api/devices \
  -H "Authorization: Bearer <token>"
```

## 安全建议

1. **修改默认密码**：首次部署后立即修改管理员密码
2. **使用 HTTPS**：生产环境必须使用 HTTPS
3. **限制访问**：使用防火墙限制 API 访问
4. **定期备份**：定期备份数据库和配置
5. **更新密钥**：修改 JWT_SECRET 为强随机密钥
6. **监控日志**：定期检查日志文件

## 版本升级

```bash
# 拉取最新代码
git pull

# 重新构建镜像
docker-compose build

# 重启服务
docker-compose up -d
```

## 卸载

```bash
# 停止并删除容器
docker-compose down

# 删除数据卷（谨慎操作！）
docker-compose down -v

# 删除镜像
docker rmi rustdesk-pro-server:latest
```
