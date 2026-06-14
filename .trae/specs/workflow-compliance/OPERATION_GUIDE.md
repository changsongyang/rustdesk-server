# RustDesk Pro Server 商业版构建部署文档

## 目录
1. [概述](#概述)
2. [工作流架构](#工作流架构)
3. [触发机制规范](#触发机制规范)
4. [版本管理规范](#版本管理规范)
5. [Docker镜像管理规范](#docker镜像管理规范)
6. [安全扫描要求](#安全扫描要求)
7. [操作审计要求](#操作审计要求)
8. [故障排除指南](#故障排除指南)

---

## 概述

### 1.1 目的

本文档旨在为RustDesk Pro Server商业版提供完整的构建部署指南，确保所有构建、打包和部署流程符合公司安全合规要求。

### 1.2 适用范围

- RustDesk Pro Server 商业版

### 1.3 文档版本

| 版本 | 日期 | 修改说明 |
|------|------|---------|
| 1.0 | 2026-06-13 | 初始版本 |
| 2.0 | 2026-06-14 | 更新触发机制、任务结构、注释规范 |

---

## 工作流架构

### 2.1 工作流文件结构

```
.github/workflows/
├── commercial-ci.yml       # 商业版CI工作流（代码检查、单元测试）
└── commercial-build-deploy.yml  # 商业版CD工作流（构建、打包、部署）
```

### 2.2 任务执行顺序

```
┌─────────────────────────────────────────────────────────────────┐
│                        CI Pipeline (commercial-ci.yml)          │
├─────────────────────────────────────────────────────────────────┤
│  P1: Rustfmt → P2: Clippy → P3: Test → P4: Build → P5: Summary │
└─────────────────────────────────────────────────────────────────┘
                              ↓ (独立触发)
┌─────────────────────────────────────────────────────────────────┐
│                   CD Pipeline (commercial-build-deploy.yml)     │
├─────────────────────────────────────────────────────────────────┤
│  P1: Pre-Build → P2: Build (Linux/Windows并行) → P3: Summary   │
│  → P4: Package → P5: Release → P6: Docker → P7: Manifest       │
│  → P8: Deploy Summary                                           │
└─────────────────────────────────────────────────────────────────┘
```

**注意**: CI和CD工作流独立触发，不再通过workflow_run自动触发。

### 2.3 任务优先级

| 优先级 | 任务 | 说明 | 并行 |
|-------|------|------|------|
| P1 | pre-build | 构建准备、版本管理、部署条件判断 | - |
| P2 | build-linux | Linux多架构并行构建 | 3架构并行 |
| P2 | build-windows | Windows构建 | 与Linux并行 |
| P3 | build-summary | 构建结果验证 | - |
| P4 | deb-package | Debian包构建（多架构） | 3架构并行 |
| P5 | github-release | GitHub Release发布 | - |
| P6 | docker-build | Docker镜像构建（多架构） | 3架构并行 |
| P7 | docker-manifest | Docker Manifest创建 | - |
| P8 | deploy-summary | 部署结果汇总、审计记录 | - |

### 2.4 并行执行策略

| 阶段 | 并行任务 | 最大并行数 | 预计时间 |
|------|---------|-----------|---------|
| P2 - Build | build-linux(3), build-windows(1) | 5 | 30分钟 |
| P4 - Package | deb-package(3) | 5 | 5分钟 |
| P6 - Deploy | docker-build(3) | 5 | 10分钟 |

---

## 触发机制规范

### 3.1 自动触发

#### 3.1.1 push触发

**触发条件**：
- 推送到`main`分支（仅执行构建，不部署）
- 推送符合`pro-v*`或`pro-*`模式的标签（完整流程）

**配置位置**：[commercial-build-deploy.yml 第43-49行](file:///c:/Users/ycsit/Downloads/rustdesk/rustdesk-server/.github/workflows/commercial-build-deploy.yml#L43-L49)

```yaml
on:
  push:
    branches: [main]
    tags:
      - 'pro-v*'
      - 'pro-*'
```

#### 3.1.2 触发行为矩阵

| 触发方式 | pre-build | build-linux | build-windows | deb-package | docker-build | deploy-summary |
|---------|------------|-------------|---------------|-------------|--------------|----------------|
| **push main** | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| **push pro-v\*** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| **手动触发(无version)** | ✅ | ✅ | ✅ | ❌ | ❌ | ❌ |
| **手动触发(pro-v1.0.0)** | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |

**说明**：
- `should_deploy=false` 时，仅执行构建阶段（P1-P3）
- `should_deploy=true` 时，执行完整流程（P1-P8）

### 3.2 手动触发

#### 3.2.1 workflow_dispatch参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| version | string | 否 | 版本号（如pro-v1.0.0），填写后触发完整部署流程 |

**配置位置**：[commercial-build-deploy.yml 第51-57行](file:///c:/Users/ycsit/Downloads/rustdesk/rustdesk-server/.github/workflows/commercial-build-deploy.yml#L51-L57)

```yaml
workflow_dispatch:
  inputs:
    version:
      description: '版本号 (e.g., pro-v1.0.0)'
      required: false
      type: string
```

#### 3.2.2 手动触发示例

```bash
# 仅执行构建（不填写version）
gh workflow run commercial-build-deploy.yml --repo changsongyang/rustdesk-server

# 执行完整部署流程（填写version）
gh workflow run commercial-build-deploy.yml \
  --repo changsongyang/rustdesk-server \
  -f version="pro-v1.0.0"
```

### 3.3 部署条件判断逻辑

**位置**：[commercial-build-deploy.yml 第119-130行](file:///c:/Users/ycsit/Downloads/rustdesk/rustdesk-server/.github/workflows/commercial-build-deploy.yml#L119-L130)

```bash
# 判断是否需要部署
# 条件: push tag (pro-v*/pro-*) 或 手动输入version
if [[ "${{ github.ref }}" == refs/tags/pro-* ]] || [[ "${{ github.event.inputs.version }}" != "" ]]; then
  echo "should_deploy=true" >> $GITHUB_OUTPUT
else
  echo "should_deploy=false" >> $GITHUB_OUTPUT
fi
```

---

## 版本管理规范

### 4.1 版本号来源

版本号统一从`pre-build` job输出，其他job不得重复计算。

**来源优先级**：
1. 手动输入 (`github.event.inputs.version`)
2. Git Tag (`github.ref`)
3. 开发版本 (`dev-${GITHUB_SHA::8}`)

### 4.2 版本号格式

| 类型 | 格式示例 | 使用场景 |
|------|---------|---------|
| 正式版本 | `pro-v1.0.0` | 正式发布 |
| 预发布版本 | `pro-1.0.0-beta` | 测试发布 |
| 开发版本 | `dev-a1b2c3d4` | 开发构建 |

### 4.3 版本号处理流程

```
输入版本: pro-v1.0.0
    ↓
去除前缀: pro-v, pro-, v
    ↓
提取版本: 1.0.0
    ↓
应用场景:
  - Docker标签: pro-v1.0.0-amd64
  - Debian包: 1.0.0
  - GitHub Release: pro-v1.0.0
```

### 4.4 pre-build输出参数

| 输出参数 | 说明 | 示例 |
|---------|------|------|
| build_id | 构建ID（时间戳+短SHA） | 20260614-143022-a1b2c3d4 |
| git_tag | Git标签 | pro-v1.0.0 |
| deb_version | Debian版本号 | 1.0.0 |
| should_deploy | 是否需要部署 | true/false |

---

## Docker镜像管理规范

### 5.1 镜像标签命名规则

**格式**：`{registry}/{image}:{version}-{arch}`

**示例**：
```
ycstech/rustdesk-pro-server:pro-v1.0.0-amd64
ycstech/rustdesk-pro-server:pro-v1.0.0-arm64v8
ycstech/rustdesk-pro-server:pro-v1.0.0-armv7
ycstech/rustdesk-pro-server:latest-amd64
```

### 5.2 支持的架构

| 架构 | Docker平台标识 | Rust目标 | 说明 |
|------|--------------|---------|------|
| amd64 | linux/amd64 | x86_64-unknown-linux-musl | x86_64 |
| arm64v8 | linux/arm64 | aarch64-unknown-linux-musl | ARM 64-bit |
| armv7 | linux/arm/v7 | armv7-unknown-linux-musleabihf | ARM 32-bit |

### 5.3 镜像仓库配置

| 仓库 | 镜像名称 | 说明 |
|------|---------|------|
| Docker Hub | `ycstech/rustdesk-pro-server` | 主镜像仓库 |
| GHCR | `ghcr.io/changsongyang/rustdesk-pro-server` | GitHub容器仓库 |

### 5.4 镜像构建流程

```
docker-build (P6)
    ↓
构建单架构镜像
    ↓
推送到 Docker Hub 和 GHCR
    ↓
docker-manifest (P7)
    ↓
创建多架构 Manifest
    ↓
推送 Manifest 到两个仓库
```

---

## 安全扫描要求

### 6.1 安全扫描工具

| 工具 | 用途 | 说明 |
|------|------|------|
| Trivy | 漏洞扫描 | 扫描CRITICAL和HIGH级别漏洞 |
| SBOM | 软件物料清单 | SPDX格式 |
| Cosign | 镜像签名 | 可选配置 |

### 6.2 漏洞扫描配置

**配置**：
- 扫描级别: CRITICAL, HIGH
- 输出格式: SARIF
- exit-code: 0（仅报告，不阻止构建）

### 6.3 安全扫描阈值

| 级别 | 当前配置 | 建议配置 |
|------|---------|---------|
| CRITICAL | 报告 | 阻止 |
| HIGH | 报告 | 阻止 |
| MEDIUM | 不扫描 | 报告 |
| LOW | 不扫描 | 忽略 |

---

## 操作审计要求

### 7.1 审计信息记录

所有工作流执行必须记录以下审计信息：

| 字段 | 来源 | 说明 |
|------|------|------|
| triggered_by | github.actor | 触发人 |
| event_type | github.event_name | 事件类型 |
| trigger_time | date -u | 触发时间 |
| git_ref | github.ref | Git引用 |
| git_sha | github.sha | Git提交SHA |
| run_id | github.run_id | 运行ID |

### 7.2 审计日志示例

```
==============================================
           DEPLOYMENT SUMMARY
==============================================
Build ID:      20260614-143022-a1b2c3d4
Version:       pro-v1.0.0
Environment:   Production
----------------------------------------------
AUDIT INFORMATION:
   Triggered by:   changsongyang
   Event Type:     push
   Trigger Time:   2026-06-14T14:30:22Z
   Git Ref:        refs/tags/pro-v1.0.0
   Git SHA:        a1b2c3d4e5f6g7h8i9j0
----------------------------------------------
GitHub Release: Completed
Docker Hub:     Completed
GHCR:           Completed
Multi-arch:     Completed
==============================================
```

---

## 故障排除指南

### 8.1 常见错误及解决方案

#### 8.1.1 YAML语法错误

**错误信息**：
```
Invalid workflow file: .github/workflows/commercial-build-deploy.yml#L1
(Line: 626, Col: 5): Required property is missing: runs-on
```

**解决方案**：
- 检查job是否缺少`runs-on`属性
- 确保所有job定义完整

#### 8.1.2 条件表达式错误

**错误信息**：
```
Unrecognized named-value: 'secrets'. Located at position 1
```

**解决方案**：
- `secrets`不能在步骤级别的`if`条件中使用
- 使用`||`运算符实现优雅失败

#### 8.1.3 镜像推送失败

**错误信息**：
```
HTTP 403: denied
```

**解决方案**：
- 检查`DOCKER_HUB_USERNAME`和`DOCKER_HUB_PASSWORD` secrets
- 确认Docker Hub权限

### 8.2 工作流执行状态检查

```bash
# 查看最近的工作流运行
gh run list --repo changsongyang/rustdesk-server --limit 10

# 查看特定工作流运行详情
gh run view <run-id> --repo changsongyang/rustdesk-server

# 查看工作流日志
gh run view <run-id> --log --repo changsongyang/rustdesk-server
```

### 8.3 调试技巧

1. **启用调试日志**
   - 在工作流中添加`echo "DEBUG: $VAR"`输出

2. **检查artifacts**
   - 访问GitHub Actions artifacts页面下载构建产物

3. **检查部署条件**
   - 查看`should_deploy`输出值是否正确

---

## 附录

### A. 相关文件路径

| 文件 | 路径 |
|------|------|
| 商业版CI工作流 | `.github/workflows/commercial-ci.yml` |
| 商业版CD工作流 | `.github/workflows/commercial-build-deploy.yml` |
| Cargo配置 | `commercial/Cargo.toml` |
| Docker配置 | `commercial/docker/` |

### B. GitHub Secrets配置

| Secret | 用途 | 必需 |
|--------|------|------|
| DOCKER_HUB_USERNAME | Docker Hub用户名 | 是 |
| DOCKER_HUB_PASSWORD | Docker Hub密码 | 是 |
| COSIGN_PRIVATE_KEY | Cosign签名密钥 | 否 |
| COSIGN_PASSWORD | Cosign密钥密码 | 否 |

### C. 联系支持

如遇到问题，请联系：
- 项目维护者: RustDesk Team <info@rustdesk.com>