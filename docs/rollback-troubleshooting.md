# RustDesk Server - 回滚与故障排除指南

## 概述

本文档提供 RustDesk Server 各种部署方式的回滚程序和故障排除指南，帮助管理员在遇到问题时快速恢复服务。

---

## 1. Docker 部署回滚

### 1.1 回滚到之前的镜像版本

```bash
# 查看当前运行的容器
docker ps

# 停止当前容器
docker stop rustdesk-hbbs
docker stop rustdesk-hbbr

# 删除当前容器
docker rm rustdesk-hbbs
docker rm rustdesk-hbbr

# 重新启动使用旧版本镜像
docker run -d --name rustdesk-hbbs \
  -p 21114:21114 \
  -p 21115:21115 \
  -p 21116:21116 \
  -p 21116:21116/udp \
  -v /var/lib/rustdesk-server:/data \
  rustdesk/rustdesk-server:v1.1.9 \
  hbbs -r your-domain.com:21116

docker run -d --name rustdesk-hbbr \
  -p 21117:21117 \
  -v /var/lib/rustdesk-server:/data \
  rustdesk/rustdesk-server:v1.1.9 \
  hbbr

# 验证回滚
docker logs rustdesk-hbbs
docker logs rustdesk-hbbr
```

### 1.2 恢复数据目录

```bash
# 如果数据目录被损坏，从备份恢复
docker stop rustdesk-hbbs rustdesk-hbbr
docker rm rustdesk-hbbs rustdesk-hbbr

# 备份当前数据
mv /var/lib/rustdesk-server /var/lib/rustdesk-server.bak

# 从备份恢复
tar -xzvf /backup/rustdesk-backup-20240101.tar.gz -C /

# 重新启动容器
docker run -d --name rustdesk-hbbs ...
docker run -d --name rustdesk-hbbr ...
```

---

## 2. Docker Compose 部署回滚

### 2.1 回滚到之前的版本

```bash
# 停止当前服务
cd /opt/rustdesk-server
docker-compose down

# 修改 docker-compose.yml 使用旧版本
sed -i 's/rustdesk\/rustdesk-server:latest/rustdesk\/rustdesk-server:v1.1.9/g' docker-compose.yml

# 重新启动服务
docker-compose up -d

# 验证回滚
docker-compose logs hbbs
docker-compose logs hbbr
```

### 2.2 使用版本控制回滚

```bash
# 使用 git 回滚配置文件
cd /opt/rustdesk-server
git log --oneline docker-compose.yml
git checkout <commit-hash> docker-compose.yml

# 重新启动服务
docker-compose down
docker-compose up -d
```

---

## 3. Podman 部署回滚

### 3.1 回滚容器版本

```bash
# 停止并删除当前容器
podman stop rustdesk-hbbs rustdesk-hbbr
podman rm rustdesk-hbbs rustdesk-hbbr

# 使用旧版本重新创建容器
podman run -d --name rustdesk-hbbs \
  -p 21114:21114 \
  -p 21115:21115 \
  -p 21116:21116 \
  -p 21116:21116/udp \
  -v rustdesk-data:/data:Z \
  rustdesk/rustdesk-server:v1.1.9 \
  hbbs -r your-domain.com:21116

podman run -d --name rustdesk-hbbr \
  -p 21117:21117 \
  -v rustdesk-data:/data:Z \
  rustdesk/rustdesk-server:v1.1.9 \
  hbbr

# 验证
podman logs rustdesk-hbbs
```

---

## 4. Kubernetes 部署回滚

### 4.1 使用 kubectl 回滚

```bash
# 查看部署历史
kubectl rollout history deployment/rustdesk-hbbs -n rustdesk
kubectl rollout history deployment/rustdesk-hbbr -n rustdesk

# 回滚到上一个版本
kubectl rollout undo deployment/rustdesk-hbbs -n rustdesk
kubectl rollout undo deployment/rustdesk-hbbr -n rustdesk

# 回滚到特定版本
kubectl rollout undo deployment/rustdesk-hbbs -n rustdesk --to-revision=2

# 验证回滚状态
kubectl rollout status deployment/rustdesk-hbbs -n rustdesk
kubectl get pods -n rustdesk
```

### 4.2 手动回滚到指定镜像

```bash
# 修改 Deployment 使用旧版本
kubectl set image deployment/rustdesk-hbbs \
  hbbs=rustdesk/rustdesk-server:v1.1.9 -n rustdesk

kubectl set image deployment/rustdesk-hbbr \
  hbbr=rustdesk/rustdesk-server:v1.1.9 -n rustdesk

# 验证
kubectl get pods -n rustdesk
```

---

## 5. 二进制部署回滚

### 5.1 回滚二进制文件

```bash
# 停止服务
sudo systemctl stop rustdesk-hbbs
sudo systemctl stop rustdesk-hbbr

# 备份当前二进制文件
mv /opt/rustdesk-server/bin/hbbs /opt/rustdesk-server/bin/hbbs.bak
mv /opt/rustdesk-server/bin/hbbr /opt/rustdesk-server/bin/hbbr.bak

# 恢复旧版本
cp /opt/rustdesk-server/bin/hbbs.old /opt/rustdesk-server/bin/hbbs
cp /opt/rustdesk-server/bin/hbbr.old /opt/rustdesk-server/bin/hbbr

# 设置权限
chmod +x /opt/rustdesk-server/bin/hbbs
chmod +x /opt/rustdesk-server/bin/hbbr

# 启动服务
sudo systemctl start rustdesk-hbbs
sudo systemctl start rustdesk-hbbr

# 验证
sudo systemctl status rustdesk-hbbs
```

### 5.2 恢复数据目录

```bash
# 停止服务
sudo systemctl stop rustdesk-hbbs
sudo systemctl stop rustdesk-hbbr

# 备份当前数据
mv /var/lib/rustdesk-server /var/lib/rustdesk-server.bak

# 从备份恢复
tar -xzvf /backup/rustdesk-backup-20240101.tar.gz -C /

# 设置权限
sudo chown -R rustdesk:rustdesk /var/lib/rustdesk-server

# 启动服务
sudo systemctl start rustdesk-hbbs
sudo systemctl start rustdesk-hbbr
```

---

## 6. 常见故障排除

### 6.1 服务无法启动

**问题**: 服务启动失败或立即退出

**排查步骤**:

```bash
# Docker 部署
docker logs rustdesk-hbbs

# Docker Compose 部署
docker-compose logs hbbs

# Kubernetes 部署
kubectl logs -n rustdesk <pod-name>

# 二进制部署
journalctl -u rustdesk-hbbs -f
```

**常见原因**:

| 错误信息 | 原因 | 解决方案 |
|----------|------|----------|
| `permission denied` | 权限不足 | 检查数据目录权限 |
| `address already in use` | 端口被占用 | 检查端口占用，修改端口或停止占用进程 |
| `cannot access data directory` | 数据目录不存在 | 创建数据目录 |
| `invalid configuration` | 配置文件错误 | 检查配置文件格式 |

### 6.2 端口占用问题

```bash
# 检查端口占用
ss -tlnp | grep 2111

# 查找占用进程
lsof -i :21115

# 杀死占用进程
kill -9 <pid>
```

### 6.3 客户端无法连接

**排查步骤**:

```bash
# 检查防火墙规则
ufw status                  # Debian/Ubuntu
firewall-cmd --list-all     # Red Hat/CentOS

# 检查网络连接
curl http://localhost:21115/status

# 检查外部访问
curl http://your-server-ip:21115/status

# 检查 DNS 解析
nslookup your-domain.com
```

### 6.4 日志显示错误

**常见日志错误及解决方案**:

| 错误信息 | 解决方案 |
|----------|----------|
| `Failed to bind to address` | 端口被占用或权限不足 |
| `Connection refused` | 服务未启动或网络不通 |
| `SSL error` | 证书配置错误 |
| `Database connection failed` | 数据库配置错误 |

### 6.5 Kubernetes Pod 状态异常

```bash
# 查看 Pod 状态
kubectl get pods -n rustdesk

# 查看 Pod 详细信息
kubectl describe pod <pod-name> -n rustdesk

# 查看事件
kubectl get events -n rustdesk

# 检查资源限制
kubectl describe nodes
```

**常见 Pod 状态**:

| 状态 | 原因 | 解决方案 |
|------|------|----------|
| `CrashLoopBackOff` | 容器启动失败 | 查看日志排查 |
| `ImagePullBackOff` | 镜像拉取失败 | 检查镜像名称和仓库 |
| `Pending` | 资源不足或调度失败 | 检查节点资源 |
| `Running` | 运行中 | 正常状态 |

### 6.6 数据目录问题

```bash
# 检查数据目录权限
ls -la /var/lib/rustdesk-server

# 检查磁盘空间
df -h

# 检查目录大小
du -sh /var/lib/rustdesk-server
```

### 6.7 性能问题排查

```bash
# 检查 CPU 使用
top
htop

# 检查内存使用
free -h

# 检查网络连接
netstat -an | grep ESTABLISHED | wc -l

# 检查磁盘 I/O
iostat -xz 1

# 检查进程状态
ps aux | grep rustdesk
```

---

## 7. 紧急恢复流程

### 7.1 服务完全不可用

```bash
# 1. 停止所有服务
# Docker
docker stop $(docker ps -q)

# Kubernetes
kubectl scale deployment rustdesk-hbbs --replicas=0 -n rustdesk
kubectl scale deployment rustdesk-hbbr --replicas=0 -n rustdesk

# 二进制
sudo systemctl stop rustdesk-hbbs
sudo systemctl stop rustdesk-hbbr

# 2. 从最近备份恢复数据
tar -xzvf /backup/rustdesk-backup-latest.tar.gz -C /

# 3. 启动服务
# Docker
docker-compose up -d

# Kubernetes
kubectl scale deployment rustdesk-hbbs --replicas=1 -n rustdesk
kubectl scale deployment rustdesk-hbbr --replicas=1 -n rustdesk

# 二进制
sudo systemctl start rustdesk-hbbs
sudo systemctl start rustdesk-hbbr

# 4. 验证服务
curl http://localhost:21115/status
```

### 7.2 数据损坏恢复

```bash
# 停止服务
sudo systemctl stop rustdesk-hbbs
sudo systemctl stop rustdesk-hbbr

# 备份损坏的数据
mv /var/lib/rustdesk-server /var/lib/rustdesk-server.damaged

# 从备份恢复
tar -xzvf /backup/rustdesk-backup-20240101.tar.gz -C /

# 设置权限
sudo chown -R rustdesk:rustdesk /var/lib/rustdesk-server

# 启动服务
sudo systemctl start rustdesk-hbbs
sudo systemctl start rustdesk-hbbr

# 验证
cat /var/lib/rustdesk-server/id_ed25519.pub
```

---

## 8. 日志分析

### 8.1 Docker 日志

```bash
# 查看实时日志
docker logs -f rustdesk-hbbs

# 查看最近 100 行
docker logs --tail 100 rustdesk-hbbs

# 过滤错误日志
docker logs rustdesk-hbbs 2>&1 | grep -i error

# 保存日志到文件
docker logs rustdesk-hbbs > /var/log/rustdesk-hbbs.log
```

### 8.2 Kubernetes 日志

```bash
# 查看 Pod 日志
kubectl logs -f <pod-name> -n rustdesk

# 查看上一个容器的日志（如果容器崩溃）
kubectl logs -p <pod-name> -n rustdesk

# 查看所有 Pod 日志
kubectl logs -l app=rustdesk-hbbs -n rustdesk
```

### 8.3 系统日志

```bash
# 查看系统日志
journalctl -u rustdesk-hbbs

# 查看实时日志
journalctl -u rustdesk-hbbs -f

# 过滤错误
journalctl -u rustdesk-hbbs -p err

# 查看最近 1 小时的日志
journalctl -u rustdesk-hbbs --since "1 hour ago"
```

---

## 9. 监控和预警

### 9.1 设置健康检查

```bash
# 创建健康检查脚本
cat > /usr/local/bin/check_rustdesk.sh << 'EOF'
#!/bin/bash

HBBS_STATUS=$(curl -s http://localhost:21115/status | jq -r '.status')
HBBR_STATUS=$(curl -s http://localhost:21117/status | jq -r '.status')

if [ "$HBBS_STATUS" != "ok" ]; then
    echo "HBBS is not healthy"
    exit 1
fi

if [ "$HBBR_STATUS" != "ok" ]; then
    echo "HBBR is not healthy"
    exit 1
fi

echo "All services are healthy"
exit 0
EOF

chmod +x /usr/local/bin/check_rustdesk.sh

# 添加到 cron
echo "*/5 * * * * /usr/local/bin/check_rustdesk.sh >> /var/log/rustdesk-health.log" | crontab -
```

### 9.2 监控指标

| 指标 | 监控方法 | 阈值建议 |
|------|----------|----------|
| CPU 使用率 | `top`, `htop` | < 80% |
| 内存使用率 | `free -h` | < 85% |
| 磁盘空间 | `df -h` | < 90% |
| 服务响应时间 | `curl -w "%{time_total}"` | < 1s |
| 端口监听 | `ss -tlnp` | 所有端口正常监听 |

---

## 10. 联系支持

如果您遇到无法解决的问题，请准备以下信息联系支持：

1. **版本信息**:
   ```bash
   hbbs --version
   hbbr --version
   ```

2. **日志文件**:
   - Docker: `docker logs rustdesk-hbbs`
   - Kubernetes: `kubectl logs <pod-name> -n rustdesk`
   - 二进制: `journalctl -u rustdesk-hbbs`

3. **系统信息**:
   ```bash
   cat /etc/os-release
   uname -a
   free -h
   df -h
   ```

4. **网络信息**:
   ```bash
   ss -tlnp | grep 2111
   curl http://localhost:21115/status
   ```

---

**文档版本**: v1.0  
**适用产品**: RustDesk Server Community & Pro  
**最后更新**: 2026-06-12
