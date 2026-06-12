# RustDesk Server - Debian 系列操作系统配置指南

## 概述

本文档提供 RustDesk Server 生产环境的操作系统配置指南，适用于 Debian GNU/Linux 和 Ubuntu Server。

---

## 1. 系统基础配置

### 1.1 更新系统

```bash
# 更新系统包
sudo apt update && sudo apt upgrade -y

# 安装基础工具
sudo apt install -y wget curl vim net-tools dnsutils chrony

# 启用 chrony 时间同步
sudo systemctl enable --now chronyd
sudo timedatectl set-ntp true
```

### 1.2 设置主机名

```bash
# 设置主机名
sudo hostnamectl set-hostname rustdesk-server.example.com

# 更新 /etc/hosts
echo "$(hostname -I | awk '{print $1}') $(hostname)" | sudo tee -a /etc/hosts
```

### 1.3 配置时区

```bash
# 设置时区（根据实际情况调整）
sudo timedatectl set-timezone Asia/Shanghai

# 验证时区设置
timedatectl status
```

---

## 2. 安全加固

### 2.1 用户管理

```bash
# 创建专用用户
sudo useradd -m -s /bin/bash rustdesk
sudo passwd rustdesk

# 添加到 sudo 组
sudo usermod -aG sudo rustdesk

# 禁止 root 远程登录
sudo sed -i 's/^PermitRootLogin yes/PermitRootLogin no/' /etc/ssh/sshd_config

# 限制 SSH 登录用户
echo "AllowUsers rustdesk" | sudo tee -a /etc/ssh/sshd_config

# 重启 SSH 服务
sudo systemctl restart sshd
```

### 2.2 防火墙配置

```bash
# 安装 ufw
sudo apt install -y ufw

# 启用防火墙
sudo ufw enable

# 允许必要端口
sudo ufw allow 22/tcp                    # SSH
sudo ufw allow 80/tcp                    # HTTP
sudo ufw allow 443/tcp                   # HTTPS
sudo ufw allow 21114:21119/tcp           # RustDesk TCP 端口
sudo ufw allow 21114:21119/udp           # RustDesk UDP 端口

# 查看当前规则
sudo ufw status verbose
```

### 2.3 AppArmor 配置

```bash
# 查看 AppArmor 状态
sudo aa-status

# 确保 AppArmor 已启用
sudo systemctl enable --now apparmor

# 如需自定义配置，创建配置文件
# sudo nano /etc/apparmor.d/usr.bin.rustdesk
```

### 2.4 禁用不必要服务

```bash
# 禁用不需要的服务
sudo systemctl disable --now avahi-daemon
sudo systemctl disable --now bluetooth
sudo systemctl disable --now cups
sudo systemctl disable --now rpcbind
```

---

## 3. 性能调优

### 3.1 CPU 配置

```bash
# 查看 CPU 信息
lscpu

# 安装 cpufrequtils（Debian）
sudo apt install -y cpufrequtils

# 启用 CPU 性能模式
echo 'GOVERNOR="performance"' | sudo tee /etc/default/cpufrequtils
sudo systemctl restart cpufrequtils
```

### 3.2 内存配置

```bash
# 查看内存使用
free -h

# 配置透明大页（THP）
echo 'never' | sudo tee /sys/kernel/mm/transparent_hugepage/enabled
echo 'never' | sudo tee /sys/kernel/mm/transparent_hugepage/defrag

# 永久生效
echo 'echo never > /sys/kernel/mm/transparent_hugepage/enabled' | sudo tee /etc/rc.local
echo 'echo never > /sys/kernel/mm/transparent_hugepage/defrag' | sudo tee -a /etc/rc.local
sudo chmod +x /etc/rc.local
```

### 3.3 文件系统优化

```bash
# 查看磁盘信息
lsblk

# 挂载选项优化（编辑 /etc/fstab）
# 添加 noatime, nodiratime 到挂载选项
sudo sed -i 's/\(.*ext4.*defaults\)/\1,noatime,nodiratime/' /etc/fstab

# 重新挂载
sudo mount -o remount /
```

### 3.4 网络调优

```bash
# 创建网络调优配置
sudo cat > /etc/sysctl.d/99-rustdesk.conf << 'EOF'
# 网络性能调优
net.core.somaxconn = 65535
net.core.netdev_max_backlog = 65535
net.ipv4.tcp_max_syn_backlog = 65535
net.ipv4.tcp_syncookies = 1
net.ipv4.tcp_tw_reuse = 1
net.ipv4.tcp_tw_recycle = 0
net.ipv4.tcp_fin_timeout = 30
net.ipv4.tcp_keepalive_time = 1200
net.ipv4.ip_local_port_range = 1024 65535
net.ipv4.tcp_max_tw_buckets = 5000
net.ipv4.tcp_sack = 1
net.ipv4.tcp_window_scaling = 1
net.ipv4.tcp_rmem = 4096 87380 6291456
net.ipv4.tcp_wmem = 4096 65536 6291456
net.core.rmem_max = 6291456
net.core.wmem_max = 6291456

# 增加文件描述符限制
fs.file-max = 1048576
EOF

# 应用配置
sudo sysctl --system
```

### 3.5 用户进程限制

```bash
# 编辑 limits.conf
sudo cat >> /etc/security/limits.conf << 'EOF'
# RustDesk 进程限制
rustdesk soft nofile 65535
rustdesk hard nofile 65535
rustdesk soft nproc 65535
rustdesk hard nproc 65535
EOF

# 编辑 /etc/pam.d/common-session
echo 'session required pam_limits.so' | sudo tee -a /etc/pam.d/common-session
```

---

## 4. 合规建议

### 4.1 日志审计

```bash
# 安装 auditd
sudo apt install -y auditd audispd-plugins

# 启用 auditd
sudo systemctl enable --now auditd

# 配置审计规则
sudo cat > /etc/audit/rules.d/rustdesk-audit.rules << 'EOF'
# 审计 RustDesk 相关文件
-w /usr/local/bin/hbbs -p rwxa
-w /usr/local/bin/hbbr -p rwxa
-w /var/lib/rustdesk/ -p rwxa
EOF

# 重启 auditd
sudo systemctl restart auditd
```

### 4.2 定期安全更新

```bash
# 安装 unattended-upgrades
sudo apt install -y unattended-upgrades apt-listchanges

# 配置自动更新
sudo dpkg-reconfigure -plow unattended-upgrades

# 或者手动配置
sudo cat > /etc/apt/apt.conf.d/50unattended-upgrades << 'EOF'
Unattended-Upgrade::Allowed-Origins {
        "${distro_id}:${distro_codename}-security";
};
Unattended-Upgrade::AutoFixInterruptedDpkg "true";
Unattended-Upgrade::MinimalSteps "true";
Unattended-Upgrade::InstallOnShutdown "false";
Unattended-Upgrade::Mail "admin@example.com";
Unattended-Upgrade::MailReport "on-change";
EOF
```

---

## 5. 验证步骤

```bash
# 验证系统更新
sudo apt update

# 验证时间同步
timedatectl status

# 验证防火墙规则
sudo ufw status verbose

# 验证 AppArmor 状态
sudo aa-status

# 验证网络配置
sysctl net.core.somaxconn

# 验证用户限制
su - rustdesk -c 'ulimit -n'

# 验证服务状态
systemctl list-units --type=service --state=running
```

---

## 6. 版本兼容性

| 操作系统 | 版本要求 | 状态 |
|----------|----------|------|
| Debian GNU/Linux | 11+ (Bullseye) | 支持 |
| Ubuntu Server | 20.04+ (Focal) | 支持 |
| Ubuntu Server | 22.04+ (Jammy) | 推荐 |

---

**文档版本**: v1.0  
**适用产品**: RustDesk Server Community & Pro  
**最后更新**: 2026-06-12
