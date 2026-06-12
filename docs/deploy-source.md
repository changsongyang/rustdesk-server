# RustDesk Server - 源码部署指南

## 概述

本文档提供从源码编译和部署 RustDesk Server 的完整指南，包括开发环境配置、依赖管理、编译过程和安装部署。

---

## 1. 环境准备

### 1.1 系统要求

| 组件 | 最低版本 | 推荐版本 |
|------|----------|----------|
| Rust | 1.70.0 | 1.75.0+ |
| Cargo | 1.70.0 | 1.75.0+ |
| Git | 2.30.0 | 2.40.0+ |
| GCC/G++ | 11.0 | 12.0+ |
| Node.js | 18.0 | 20.0+ |

### 1.2 安装基础依赖

**Debian/Ubuntu:**
```bash
# 更新系统
sudo apt update && sudo apt upgrade -y

# 安装基础工具
sudo apt install -y build-essential git curl wget tar unzip

# 安装编译依赖
sudo apt install -y libssl-dev pkg-config libclang-dev cmake
```

**Red Hat/CentOS:**
```bash
# 安装基础工具
sudo dnf install -y gcc gcc-c++ git curl wget tar unzip

# 安装编译依赖
sudo dnf install -y openssl-devel pkg-config clang-devel cmake
```

---

## 2. 安装 Rust 开发环境

### 2.1 使用 rustup 安装

```bash
# 安装 rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# 加载环境变量
source "$HOME/.cargo/env"

# 验证安装
rustc --version
cargo --version
```

### 2.2 配置 Rust 工具链

```bash
# 查看当前工具链
rustup show

# 安装稳定版工具链
rustup install stable
rustup default stable

# 安装 nightly 工具链（可选，用于开发）
rustup install nightly
rustup component add rustfmt
rustup component add clippy
```

---

## 3. 克隆源码仓库

### 3.1 克隆主仓库

```bash
# 创建工作目录
mkdir -p ~/projects/rustdesk
cd ~/projects/rustdesk

# 克隆仓库
git clone https://github.com/rustdesk/rustdesk-server.git
cd rustdesk-server

# 查看分支
git branch -a

# 切换到稳定分支
git checkout stable

# 查看提交信息
git log --oneline -5
```

### 3.2 克隆子模块

```bash
# 初始化子模块
git submodule init
git submodule update --recursive

# 或者克隆时包含子模块
git clone --recursive https://github.com/rustdesk/rustdesk-server.git
```

---

## 4. 编译项目

### 4.1 编译所有组件

```bash
# 进入项目目录
cd ~/projects/rustdesk/rustdesk-server

# 编译所有二进制文件（Release 模式）
cargo build --release --all-features

# 查看编译产物
ls -la target/release/
```

### 4.2 编译特定组件

```bash
# 只编译 hbbs（Rendezvous 服务器）
cargo build --release --bin hbbs

# 只编译 hbbr（Relay 服务器）
cargo build --release --bin hbbr

# 编译工具集
cargo build --release --bin rustdesk-utils
```

### 4.3 编译选项说明

```bash
# 使用特定功能编译
cargo build --release --features "web console"

# 禁用默认功能
cargo build --release --no-default-features --features "core"

# 启用调试信息（用于开发）
cargo build --release --debug

# 优化编译（较慢但生成更小更快的二进制）
cargo build --release --profile=release
```

### 4.4 交叉编译

#### 4.4.1 编译为 ARM64 架构

```bash
# 安装交叉编译目标
rustup target add aarch64-unknown-linux-musl

# 安装交叉编译工具链（Debian/Ubuntu）
sudo apt install -y gcc-aarch64-linux-gnu

# 安装交叉编译工具链（Red Hat/CentOS）
sudo dnf install -y gcc-aarch64-linux-gnu

# 交叉编译
cargo build --release --target=aarch64-unknown-linux-musl --all-features

# 查看产物
ls -la target/aarch64-unknown-linux-musl/release/
```

#### 4.4.2 编译为 ARMv7 架构

```bash
# 安装交叉编译目标
rustup target add armv7-unknown-linux-gnueabihf

# 安装交叉编译工具链
sudo apt install -y gcc-arm-linux-gnueabihf

# 交叉编译
cargo build --release --target=armv7-unknown-linux-gnueabihf --all-features
```

---

## 5. 验证编译结果

### 5.1 检查二进制文件

```bash
# 检查文件类型
file target/release/hbbs
file target/release/hbbr

# 检查文件大小
ls -lh target/release/hbbs target/release/hbbr

# 验证二进制可执行性
target/release/hbbs --help
target/release/hbbr --help
```

### 5.2 运行测试

```bash
# 运行所有测试
cargo test --release --all-features

# 运行特定测试
cargo test --release --bin hbbs --test integration

# 运行基准测试
cargo bench --release
```

---

## 6. 安装到系统

### 6.1 创建目录结构

```bash
# 创建系统目录
sudo mkdir -p /opt/rustdesk-server/bin
sudo mkdir -p /var/lib/rustdesk-server
sudo mkdir -p /etc/rustdesk-server
```

### 6.2 复制二进制文件

```bash
# 复制编译好的二进制文件
sudo cp target/release/hbbs /opt/rustdesk-server/bin/
sudo cp target/release/hbbr /opt/rustdesk-server/bin/
sudo cp target/release/rustdesk-utils /opt/rustdesk-server/bin/

# 设置执行权限
sudo chmod +x /opt/rustdesk-server/bin/*

# 创建软链接
sudo ln -s /opt/rustdesk-server/bin/hbbs /usr/local/bin/hbbs
sudo ln -s /opt/rustdesk-server/bin/hbbr /usr/local/bin/hbbr
sudo ln -s /opt/rustdesk-server/bin/rustdesk-utils /usr/local/bin/rustdesk-utils
```

### 6.3 创建专用用户

```bash
# 创建专用用户
sudo useradd -r -s /usr/sbin/nologin -d /var/lib/rustdesk-server rustdesk

# 设置目录权限
sudo chown -R rustdesk:rustdesk /var/lib/rustdesk-server
sudo chown -R rustdesk:rustdesk /opt/rustdesk-server
```

### 6.4 配置 systemd 服务

**创建 hbbs 服务文件:**
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

**创建 hbbr 服务文件:**
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

### 6.5 启用并启动服务

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

## 7. 配置防火墙

### 7.1 Debian/Ubuntu (ufw)

```bash
# 允许必要端口
sudo ufw allow 21114/tcp                    # 心跳服务
sudo ufw allow 21115/tcp                    # API 服务
sudo ufw allow 21116/tcp                    # Rendezvous TCP
sudo ufw allow 21116/udp                    # Rendezvous UDP
sudo ufw allow 21117/tcp                    # Relay 服务

# 重新加载防火墙
sudo ufw reload
```

### 7.2 Red Hat/CentOS (firewalld)

```bash
# 允许必要端口
sudo firewall-cmd --add-port=21114/tcp --permanent
sudo firewall-cmd --add-port=21115/tcp --permanent
sudo firewall-cmd --add-port=21116/tcp --permanent
sudo firewall-cmd --add-port=21116/udp --permanent
sudo firewall-cmd --add-port=21117/tcp --permanent

# 重新加载防火墙
sudo firewall-cmd --reload
```

---

## 8. 验证部署

### 8.1 检查服务状态

```bash
# 查看服务状态
sudo systemctl status rustdesk-hbbs
sudo systemctl status rustdesk-hbbr

# 检查端口监听
ss -tlnp | grep 2111
```

### 8.2 测试服务

```bash
# 测试 hbbs
curl http://localhost:21115/status

# 测试 hbbr
curl http://localhost:21117/status
```

### 8.3 获取公钥

```bash
# 获取公钥（客户端需要此信息）
cat /var/lib/rustdesk-server/id_ed25519.pub
```

---

## 9. 开发工作流

### 9.1 代码结构

```
rustdesk-server/
├── Cargo.toml              # 项目依赖配置
├── Cargo.lock              # 依赖锁定文件
├── src/
│   ├── main.rs             # 主入口
│   ├── hbbs/               # Rendezvous 服务器
│   ├── hbbr/               # Relay 服务器
│   └── common/             # 公共模块
├── tests/                  # 集成测试
└── docker/                 # Docker 相关文件
```

### 9.2 开发命令

```bash
# 构建（开发模式）
cargo build

# 运行（开发模式）
cargo run --bin hbbs -- -r localhost:21116

# 格式化代码
cargo fmt

# 静态分析
cargo clippy

# 生成文档
cargo doc --open
```

### 9.3 调试

```bash
# 使用 rust-gdb 调试
rust-gdb target/debug/hbbs

# 使用 LLDB 调试
rust-lldb target/debug/hbbs

# 设置断点调试
cargo run --bin hbbs -- -r localhost:21116 --log-level debug
```

---

## 10. 清理和维护

### 10.1 清理构建缓存

```bash
# 清理所有构建产物
cargo clean

# 清理特定目标
cargo clean --target=aarch64-unknown-linux-musl

# 清理旧的编译缓存
rm -rf ~/.cargo/registry/index
```

### 10.2 更新依赖

```bash
# 更新 Cargo.lock
cargo update

# 更新特定依赖
cargo update serde
```

### 10.3 查看依赖树

```bash
# 查看依赖树
cargo tree

# 查看特定依赖
cargo tree -p serde

# 查看可更新的依赖
cargo outdated
```

---

## 11. 常见问题

### 11.1 编译失败

**问题**: `error: linker command failed with exit code 1`

**解决方案**:
```bash
# 安装缺失的链接器
sudo apt install -y binutils
sudo apt install -y gcc-multilib
```

### 11.2 内存不足

**问题**: 编译时内存不足

**解决方案**:
```bash
# 限制并行编译数量
cargo build --release --jobs 2

# 或者增加交换空间
sudo fallocate -l 4G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

### 11.3 依赖冲突

**问题**: 依赖版本冲突

**解决方案**:
```bash
# 更新依赖
cargo update

# 查看冲突详情
cargo tree --invert problematic-crate
```

---

## 12. 版本兼容性

| RustDesk Server 版本 | 最低 Rust 版本 | 推荐 Rust 版本 |
|----------------------|----------------|----------------|
| 1.2.0+ | 1.70.0 | 1.75.0+ |
| 1.1.0+ | 1.65.0 | 1.70.0+ |
| 1.0.0+ | 1.60.0 | 1.65.0+ |

---

**文档版本**: v1.0  
**适用产品**: RustDesk Server Community & Pro  
**最后更新**: 2026-06-12
