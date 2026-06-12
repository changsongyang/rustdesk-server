# RustDesk Server 部署文档

欢迎使用 RustDesk Server 部署文档！本文档提供了多种部署方法，帮助您在不同环境中快速部署 RustDesk Server。

---

## 快速导航

### 📋 操作系统配置指南
- [Red Hat 系列配置](os-config-redhat.md) - CentOS/Rocky Linux/Fedora
- [Debian 系列配置](os-config-debian.md) - Debian/Ubuntu

### 🐳 容器化部署
- [Docker 部署](deploy-docker.md) - 单容器部署方案
- [Docker Compose 部署](deploy-docker-compose.md) - 多容器编排
- [Podman 部署](deploy-podman.md) - rootless 容器部署
- [Podman Compose 部署](deploy-podman-compose.md) - Podman 编排

### ☸️ Kubernetes 部署
- [Kubernetes 部署](deploy-kubernetes.md) - 生产级 K8s 部署方案

### 📦 传统部署
- [二进制部署](deploy-binary.md) - 预编译二进制安装
- [源码部署](deploy-source.md) - 从源码编译部署

### 🔧 运维指南
- [回滚与故障排除](rollback-troubleshooting.md) - 问题排查和恢复

---

## 部署方式对比

| 部署方式 | 优点 | 缺点 | 适用场景 |
|----------|------|------|----------|
| **Docker** | 简单快捷，环境隔离 | 需要 Docker 环境 | 快速测试、开发环境 |
| **Docker Compose** | 多服务编排，配置集中 | 需要 Docker 环境 | 小型生产环境 |
| **Podman** | 无需 root，安全 | 生态不如 Docker 成熟 | 安全要求高的环境 |
| **Podman Compose** | 安全编排，无 root | 兼容性问题 | 安全要求高的编排场景 |
| **Kubernetes** | 高可用，自动扩缩容 | 复杂度高 | 大规模生产环境 |
| **二进制** | 轻量，无依赖 | 手动管理升级 | 资源受限环境 |
| **源码** | 完全可控，自定义编译 | 编译时间长 | 开发、定制化需求 |

---

## 版本兼容性矩阵

### 服务器版本兼容性

| RustDesk Server 版本 | 客户端最低版本 | Docker 版本要求 | Kubernetes 版本要求 |
|----------------------|----------------|-----------------|---------------------|
| 1.2.0+ | 1.2.0 | 20.10+ | 1.21+ |
| 1.1.0+ | 1.1.0 | 19.03+ | 1.19+ |
| 1.0.0+ | 1.0.0 | 18.09+ | 1.17+ |

### 操作系统支持

| 操作系统 | 版本 | 支持状态 |
|----------|------|----------|
| CentOS | 7/8 | ✅ 支持 |
| Rocky Linux | 8/9 | ✅ 支持 |
| Fedora | 38+ | ✅ 支持 |
| Debian | 10/11/12 | ✅ 支持 |
| Ubuntu | 20.04/22.04 | ✅ 支持 |
| Ubuntu | 18.04 | ⚠️ 有限支持 |

### 架构支持

| 架构 | Docker | 二进制 | 源码编译 |
|------|--------|--------|----------|
| x86_64 | ✅ | ✅ | ✅ |
| ARM64 | ✅ | ✅ | ✅ |
| ARMv7 | ✅ | ✅ | ✅ |

---

## 一键部署

使用我们的一键部署脚本快速开始：
- 支持 Docker、Docker Compose、Podman、Podman Compose、Kubernetes、二进制六种部署方式
- 交互式菜单选择
- 自动安装依赖
- 部署完成后自动验证


```bash
# 下载脚本
curl -LO https://raw.githubusercontent.com/changsongyang/rustdesk-pro-server/main/scripts/rustdesk-deploy.sh

# 赋予执行权限
chmod +x rustdesk-deploy.sh

# 运行脚本
sudo ./rustdesk-deploy.sh
```

---

## 端口说明

| 端口 | 服务 | 协议 | 用途 |
|------|------|------|------|
| 21114 | hbbs | TCP | 心跳服务 |
| 21115 | hbbs | TCP | API 服务 |
| 21116 | hbbs | TCP/UDP | Rendezvous |
| 21117 | hbbr | TCP | Relay 服务 |
| 21118 | hbbs | TCP | 备用端口 |
| 21119 | hbbs | TCP | 备用端口 |

---

## 快速验证清单

部署完成后，使用以下命令验证：

```bash
# 检查端口监听
ss -tlnp | grep 2111

# 测试服务状态
curl http://localhost:21115/status
curl http://localhost:21117/status

# 获取公钥
cat /var/lib/rustdesk-server/id_ed25519.pub
```

---

## ✨ 文档特点
1. 生产级配置 ：包含安全加固、性能调优、合规建议
2. 可复制执行 ：所有命令均可直接复制粘贴运行
3. 版本兼容 ：包含完整的版本兼容性矩阵
4. 验证步骤 ：每个部署方法都有详细的验证步骤
5. 回滚方案 ：提供完整的回滚程序和故障排除指南

## 支持与反馈

如果您在部署过程中遇到问题：

1. 查看 [故障排除指南](rollback-troubleshooting.md)
2. 检查项目 [GitHub Issues](https://github.com/rustdesk/rustdesk-server/issues)
3. 加入 [Discord 社区](https://discord.com/invite/nDceKgxnkV)

---

**文档版本**: v1.0  
**适用产品**: RustDesk Server Community & Pro  
**最后更新**: 2026-06-12
