# RustDesk Server - Red Hat 系列操作系统配置指南

## 概述

本文档提供 RustDesk Server 生产环境的操作系统配置指南，适用于 Red Hat Enterprise Linux、CentOS Stream、Rocky Linux 和 AlmaLinux。

---

## 1. 系统基础配置

### 1.1 更新系统

```bash
# 更新系统包
sudo dnf update -y

# 安装基础工具
sudo dnf install -y wget curl vim net-tools bind-utils chrony

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

# 添加到 wheel 组（如需 sudo 权限）
sudo usermod -aG wheel rustdesk

# 禁止 root 远程登录
sudo sed -i 's/^PermitRootLogin yes/PermitRootLogin no/' /etc/ssh/sshd_config

# 限制 SSH 登录用户
echo "AllowUsers rustdesk" | sudo tee -a /etc/ssh/sshd_config

# 重启 SSH 服务
sudo systemctl restart sshd
```

### 2.2 防火墙配置

```bash
# 启用防火墙
sudo systemctl enable --now firewalld

# 允许必要端口
sudo firewall-cmd --add-port=22/tcp --permanent    # SSH
sudo firewall-cmd --add-port=80/tcp --permanent    # HTTP
sudo firewall-cmd --add-port=443/tcp --permanent   # HTTPS
sudo firewall-cmd --add-port=21114-21119/tcp --permanent  # RustDesk 端口
sudo firewall-cmd --add-port=21114-21119/udp --permanent  # RustDesk 端口

# 重新加载防火墙配置
sudo firewall-cmd --reload

# 查看当前规则
sudo firewall-cmd --list-all
```

### 2.3 SELinux 配置

```bash
# 查看 SELinux 状态
sestatus

# 设置为 enforcing 模式（推荐）
sudo setenforce 1
sudo sed -i 's/^SELINUX=.*/SELINUX=enforcing/' /etc/selinux/config

# 如果使用 Docker，可能需要设置容器标签
sudo setsebool -P container_use_cephfs on
sudo setsebool -P container_manage_cgroup on
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

# 启用 CPU 性能模式
echo 'performance' | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
```

### 3.2 内存配置

```bash
# 查看内存使用
free -h

# 配置透明大页（THP）
echo 'never' | sudo tee /sys/kernel/mm/transparent_hugepage/enabled
echo 'never' | sudo tee /sys/kernel/mm/transparent_hugepage/defrag

# 永久生效
echo 'echo never > /sys/kernel/mm/transparent_hugepage/enabled' | sudo tee /etc/rc.d/rc.local
echo 'echo never > /sys/kernel/mm/transparent_hugepage/defrag' | sudo tee -a /etc/rc.d/rc.local
sudo chmod +x /etc/rc.d/rc.local
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
sudo dnf install -y audit

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
# 启用自动安全更新
sudo dnf install -y dnf-automatic

# 配置自动更新
sudo sed -i 's/^apply_updates = no/apply_updates = yes/' /etc/dnf/automatic.conf
sudo sed -i 's/^emit_via = stdio/emit_via = email/' /etc/dnf/automatic.conf
sudo sed -i 's/^email_to = root/email_to = admin@example.com/' /etc/dnf/automatic.conf

# 启用自动更新服务
sudo systemctl enable --now dnf-automatic.timer
```

---

## 5. 验证步骤

```bash
# 验证系统更新
sudo dnf check-update

# 验证时间同步
timedatectl status

# 验证防火墙规则
sudo firewall-cmd --list-all

# 验证 SELinux 状态
sestatus

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
| Red Hat Enterprise Linux | 8.0+ | 支持 |
| CentOS Stream | 8+ | 支持 |
| Rocky Linux | 8.0+ | 支持 |
| AlmaLinux | 8.0+ | 支持 |

---

**文档版本**: v1.0  
**适用产品**: RustDesk Server Community & Pro  
**最后更新**: 2026-06-12
