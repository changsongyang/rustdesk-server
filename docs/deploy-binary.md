# RustDesk Server - 二进制部署指南

## 概述

本文档提供 RustDesk Server 的二进制部署指南，包括从官方下载预编译二进制文件或从源码编译，以及系统服务配置。

---

## 1. 下载预编译二进制文件

### 1.1 从 GitHub Releases 下载

```bash
# 创建安装目录
mkdir -p /opt/rustdesk-server/bin
mkdir -p /var/lib/rustdesk-server

# 下载最新版本
cd /opt/rustdesk-server/bin
curl -LO https://github.com/rustdesk/rustdesk-server/releases/latest/download/rustdesk-server-linux-amd64.tar.gz

# 解压文件
tar -xzvf rustdesk-server-linux-amd64.tar.gz

# 查看文件
ls -la
```

### 1.2 下载特定版本

```bash
# 下载特定版本（示例：1.2.3）
VERSION="1.2.3"
curl -LO "https://github.com/rustdesk/rustdesk-server/releases/download/v${VERSION}/rustdesk-server-linux-amd64.tar.gz"
```

### 1.3 验证文件完整性

```bash
# 下载校验文件
curl -LO https://github.com/rustdesk/rustdesk-server/releases/latest/download/rustdesk-server-linux-amd64.tar.gz.sha256

# 验证 SHA256
sha256sum -c rustdesk-server-linux-amd64.tar.gz.sha256
```

---

## 2. 从源码编译

### 2.1 安装依赖

**Debian/Ubuntu:**
```bash
# 更新系统
sudo apt update && sudo apt upgrade -y

# 安装依赖
sudo apt install -y build-essential git curl

# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# 加载 Rust 环境
source "$HOME/.cargo/env"

# 验证安装
rustc --version
cargo --version
```

**Red Hat/CentOS:**
```bash
# 安装依赖
sudo dnf install -y gcc gcc-c++ git curl

# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# 加载 Rust 环境
source "$HOME/.cargo/env"

# 验证安装
rustc --version
cargo --version
```

### 2.2 克隆仓库

```bash
# 克隆仓库
git clone https://github.com/rustdesk/rustdesk-server.git
cd rustdesk-server

# 查看分支
git branch -a

# 切换到稳定分支（可选）
git checkout stable
```

### 2.3 编译项目

```bash
# 编译所有二进制文件
cargo build --release --all-features

# 编译特定组件
cargo build --release --bin hbbs
cargo build --release --bin hbbr
cargo build --release --bin rustdesk-utils

# 查看编译产物
ls -la target/release/
```

### 2.4 交叉编译（可选）

```bash
# 安装交叉编译目标（以 arm64 为例）
rustup target add aarch64-unknown-linux-musl

# 安装交叉编译工具链
# Debian/Ubuntu
sudo apt install -y gcc-aarch64-linux-gnu

# Red Hat/CentOS
sudo dnf install -y gcc-aarch64-linux-gnu

# 交叉编译
cargo build --release --target=aarch64-unknown-linux-musl --all-features

# 查看编译产物
ls -la target/aarch64-unknown-linux-musl/release/
```

---

## 3. 安装二进制文件

### 3.1 复制文件

```bash
# 创建目录结构
sudo mkdir -p /opt/rustdesk-server/bin
sudo mkdir -p /var/lib/rustdesk-server
sudo mkdir -p /etc/rustdesk-server

# 复制二进制文件（从下载的压缩包）
sudo cp rustdesk-server-linux-amd64/hbbs /opt/rustdesk-server/bin/
sudo cp rustdesk-server-linux-amd64/hbbr /opt/rustdesk-server/bin/
sudo cp rustdesk-server-linux-amd64/rustdesk-utils /opt/rustdesk-server/bin/

# 或者从编译目录复制
# sudo cp target/release/hbbs /opt/rustdesk-server/bin/
# sudo cp target/release/hbbr /opt/rustdesk-server/bin/
# sudo cp target/release/rustdesk-utils /opt/rustdesk-server/bin/

# 设置执行权限
sudo chmod +x /opt/rustdesk-server/bin/*

# 创建软链接
sudo ln -s /opt/rustdesk-server/bin/hbbs /usr/local/bin/hbbs
sudo ln -s /opt/rustdesk-server/bin/hbbr /usr/local/bin/hbbr
sudo ln -s /opt/rustdesk-server/bin/rustdesk-utils /usr/local/bin/rustdesk-utils
```

### 3.2 创建配置文件

```bash
# 创建配置文件
sudo cat > /etc/rustdesk-server/config.toml << 'EOF'
# RustDesk Server 配置文件

[hbbs]
# 监听地址
listen_addr = "0.0.0.0:21115"
# 心跳端口
heartbeat_port = 21114
# 端口范围
port_range_start = 21116
port_range_end = 21118
# 中继服务器地址
relay_server = "your-domain.com:21117"
# 数据目录
data_dir = "/var/lib/rustdesk-server"

[hbbr]
# 监听地址
listen_addr = "0.0.0.0:21117"
# 数据目录
data_dir = "/var/lib/rustdesk-server"

[logging]
# 日志级别: trace, debug, info, warn, error
level = "info"
# 日志输出: console, file, both
output = "both"
# 日志文件目录
log_dir = "/var/log/rustdesk-server"
EOF

# 设置权限
sudo chown -R rustdesk:rustdesk /etc/rustdesk-server
```

### 3.3 创建专用用户

```bash
# 创建专用用户
sudo useradd -r -s /usr/sbin/nologin -d /var/lib/rustdesk-server rustdesk

# 设置目录权限
sudo chown -R rustdesk:rustdesk /var/lib/rustdesk-server
sudo chown -R rustdesk:rustdesk /opt/rustdesk-server
```

---

## 4. 配置 systemd 服务

### 4.1 创建 hbbs 服务文件

```bash
sudo cat > /etc/systemd/system/rustdesk-hbbs.service << 'EOF'
[Unit]
Description=RustDesk Rendezvous Server (hbbs)
After=network.target
Wants=network.target

[Service]
Type=simple
User=rustdesk
Group=rustdesk
WorkingDirectory=/var/lib/rustdesk-server
ExecStart=/opt/rustdesk-server/bin/hbbs -r your-domain.com:21116
Restart=always
RestartSec=5
LimitNOFILE=65535
LimitNPROC=65535

[Install]
WantedBy=multi-user.target
EOF
```

### 4.2 创建 hbbr 服务文件

```bash
sudo cat > /etc/systemd/system/rustdesk-hbbr.service << 'EOF'
[Unit]
Description=RustDesk Relay Server (hbbr)
After=network.target
Wants=network.target

[Service]
Type=simple
User=rustdesk
Group=rustdesk
WorkingDirectory=/var/lib/rustdesk-server
ExecStart=/opt/rustdesk-server/bin/hbbr
Restart=always
RestartSec=5
LimitNOFILE=65535
LimitNPROC=65535

[Install]
WantedBy=multi-user.target
EOF
```

### 4.3 启用并启动服务

```bash
# 重新加载 systemd
sudo systemctl daemon-reload

# 启用服务（开机自启）
sudo systemctl enable rustdesk-hbbs
sudo systemctl enable rustdesk-hbbr

# 启动服务
sudo systemctl start rustdesk-hbbs
sudo systemctl start rustdesk-hbbr

# 查看服务状态
sudo systemctl status rustdesk-hbbs
sudo systemctl status rustdesk-hbbr
```

---

## 5. 防火墙配置

### 5.1 Debian/Ubuntu (ufw)

```bash
# 允许必要端口
sudo ufw allow 21114/tcp                    # 心跳服务
sudo ufw allow 21115/tcp                    # API 服务
sudo ufw allow 21116/tcp                    # Rendezvous TCP
sudo ufw allow 21116/udp                    # Rendezvous UDP
sudo ufw allow 21117/tcp                    # Relay 服务
sudo ufw allow 21118/tcp                    # 备用端口
sudo ufw allow 21119/tcp                    # 备用端口

# 重新加载防火墙
sudo ufw reload

# 查看状态
sudo ufw status verbose
```

### 5.2 Red Hat/CentOS (firewalld)

```bash
# 允许必要端口
sudo firewall-cmd --add-port=21114/tcp --permanent
sudo firewall-cmd --add-port=21115/tcp --permanent
sudo firewall-cmd --add-port=21116/tcp --permanent
sudo firewall-cmd --add-port=21116/udp --permanent
sudo firewall-cmd --add-port=21117/tcp --permanent
sudo firewall-cmd --add-port=21118/tcp --permanent
sudo firewall-cmd --add-port=21119/tcp --permanent

# 重新加载防火墙
sudo firewall-cmd --reload

# 查看状态
sudo firewall-cmd --list-all
```

---

## 6. 验证部署

### 6.1 检查服务状态

```bash
# 查看服务状态
sudo systemctl status rustdesk-hbbs
sudo systemctl status rustdesk-hbbr

# 查看所有服务
systemctl list-units --type=service --state=running | grep rustdesk
```

### 6.2 检查端口监听

```bash
# 检查端口监听
ss -tlnp | grep 2111

# 使用 netstat
netstat -tlnp | grep 2111
```

### 6.3 测试服务

```bash
# 测试 hbbs
curl http://localhost:21115/status

# 测试 hbbr
curl http://localhost:21117/status
```

### 6.4 获取公钥

```bash
# 获取公钥（客户端需要此信息）
cat /var/lib/rustdesk-server/id_ed25519.pub
```

---

## 7. 日志管理

### 7.1 查看日志

```bash
# 查看 hbbs 日志
journalctl -u rustdesk-hbbs -f

# 查看 hbbr 日志
journalctl -u rustdesk-hbbr -f

# 查看最近日志
journalctl -u rustdesk-hbbs --since "1 hour ago"

# 查看错误日志
journalctl -u rustdesk-hbbs -p err
```

### 7.2 配置日志轮转

```bash
sudo cat > /etc/logrotate.d/rustdesk-server << 'EOF'
/var/log/rustdesk-server/*.log {
    daily
    rotate 7
    compress
    delaycompress
    missingok
    notifempty
    create 640 rustdesk rustdesk
    postrotate
        systemctl reload rustdesk-hbbs > /dev/null 2>&1 || true
        systemctl reload rustdesk-hbbr > /dev/null 2>&1 || true
    endscript
}
EOF
```

---

## 8. 备份和恢复

### 8.1 备份数据

```bash
# 创建备份目录
mkdir -p /backup/rustdesk-server

# 停止服务
sudo systemctl stop rustdesk-hbbs
sudo systemctl stop rustdesk-hbbr

# 创建备份
tar -czvf /backup/rustdesk-server/backup-$(date +%Y%m%d).tar.gz /var/lib/rustdesk-server

# 启动服务
sudo systemctl start rustdesk-hbbs
sudo systemctl start rustdesk-hbbr
```

### 8.2 恢复数据

```bash
# 停止服务
sudo systemctl stop rustdesk-hbbs
sudo systemctl stop rustdesk-hbbr

# 备份当前数据
mv /var/lib/rustdesk-server /var/lib/rustdesk-server.bak

# 恢复备份
tar -xzvf /backup/rustdesk-server/backup-YYYYMMDD.tar.gz -C /

# 设置权限
sudo chown -R rustdesk:rustdesk /var/lib/rustdesk-server

# 启动服务
sudo systemctl start rustdesk-hbbs
sudo systemctl start rustdesk-hbbr
```

---

## 9. 更新版本

### 9.1 下载新版本

```bash
# 停止服务
sudo systemctl stop rustdesk-hbbs
sudo systemctl stop rustdesk-hbbr

# 下载新版本
cd /opt/rustdesk-server/bin
curl -LO https://github.com/rustdesk/rustdesk-server/releases/latest/download/rustdesk-server-linux-amd64.tar.gz

# 解压
tar -xzvf rustdesk-server-linux-amd64.tar.gz

# 复制文件
cp rustdesk-server-linux-amd64/hbbs /opt/rustdesk-server/bin/
cp rustdesk-server-linux-amd64/hbbr /opt/rustdesk-server/bin/
cp rustdesk-server-linux-amd64/rustdesk-utils /opt/rustdesk-server/bin/

# 设置权限
chmod +x /opt/rustdesk-server/bin/*

# 启动服务
sudo systemctl start rustdesk-hbbs
sudo systemctl start rustdesk-hbbr

# 验证更新
hbbs --version
```

---

## 10. 故障排除

### 10.1 常见问题

| 问题 | 原因 | 解决方案 |
|------|------|----------|
| 服务无法启动 | 权限问题 | 检查目录权限 |
| 端口被占用 | 端口冲突 | 修改端口或释放端口 |
| 日志显示权限错误 | 用户权限不足 | 检查文件权限 |
| 客户端无法连接 | 防火墙阻止 | 配置防火墙规则 |

### 10.2 调试模式

```bash
# 停止服务
sudo systemctl stop rustdesk-hbbs

# 手动启动（调试模式）
sudo -u rustdesk /opt/rustdesk-server/bin/hbbs -r your-domain.com:21116 --log-level debug
```

---

## 11. 版本兼容性

| RustDesk Server 版本 | 最低 Rust 版本 | 状态 |
|----------------------|----------------|------|
| 1.2.0+ | 1.70+ | 支持 |
| 1.1.0+ | 1.65+ | 支持 |

---

**文档版本**: v1.0  
**适用产品**: RustDesk Server Community & Pro  
**最后更新**: 2026-06-12
