# RustDesk Workflow 合规化规范操作指南

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

本文档旨在为RustDesk项目的CI/CD工作流提供一套标准化的合规管理方案，确保所有构建、打包和部署流程符合公司安全合规要求。

### 1.2 适用范围

- RustDesk Server 社区版
- RustDesk Pro Server 商业版

### 1.3 文档版本

| 版本 | 日期 | 修改说明 |
|------|------|---------|
| 1.0 | 2026-06-13 | 初始版本 |

---

## 工作流架构

### 2.1 工作流文件结构

```
.github/workflows/
├── build.yaml              # 社区版构建工作流
├── commercial-ci.yml       # 商业版CI工作流
└── commercial-build-deploy.yml  # 商业版CD工作流
```

### 2.2 任务执行顺序

```
┌─────────────────────────────────────────────────────────────────┐
│                        CI Pipeline (commercial-ci.yml)          │
├─────────────────────────────────────────────────────────────────┤
│  P1: Rustfmt → P2: Clippy → P3: Test → P4: Build → P5: Summary │
└─────────────────────────────────────────────────────────────────┘
                              ↓ (workflow_run)
┌─────────────────────────────────────────────────────────────────┐
│                   CD Pipeline (commercial-build-deploy.yml)     │
├─────────────────────────────────────────────────────────────────┤
│  P0: CI Check → P1: Pre-Build → P2: Build → P3: Summary        │
│  → P4: Package → P5: Release → P6: Docker → P7: Manifest       │
└─────────────────────────────────────────────────────────────────┘
```

### 2.3 任务优先级

| 优先级 | 任务 | 说明 |
|-------|------|------|
| P1 | pre-build | 构建准备和版本信息 |
| P2 | build-linux/build-windows | 并行构建（最多5个并行） |
| P3 | build-summary | 构建结果验证 |
| P4 | deb-package | Debian包构建 |
| P5 | github-release | GitHub Release发布 |
| P6 | docker-build | Docker镜像构建（最多5个并行） |
| P7 | docker-manifest | Docker Manifest创建 |

---

## 触发机制规范

### 3.1 自动触发

#### 3.1.1 push触发

**触发条件**：
- 推送到`main`分支
- 推送符合`pro-v*`或`pro-*`模式的标签

**配置位置**：[commercial-build-deploy.yml 第48-52行](file:///c:/Users/ycsit/Downloads/rustdesk/rustdesk-server/.github/workflows/commercial-build-deploy.yml#L48-L52)

```yaml
push:
  branches: [main]
  tags:
    - 'pro-v*'
    - 'pro-*'
```

#### 3.1.2 workflow_run触发

**触发条件**：
- `CI`工作流成功完成
- 分支为`main`

**配置位置**：[commercial-build-deploy.yml 第40-45行](file:///c:/Users/ycsit/Downloads/rustdesk/rustdesk-server/.github/workflows/commercial-build-deploy.yml#L40-L45)

```yaml
workflow_run:
  workflows: ["CI"]
  types:
    - completed
  branches: [main]
```

### 3.2 手动触发

#### 3.2.1 workflow_dispatch参数

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| version | string | 否 | 版本号（如pro-v1.0.0） |
| skip-ci-check | boolean | 否 | 跳过CI状态检查（仅用于紧急发布） |
| skip-deploy | boolean | 否 | 跳过部署阶段 |

**配置位置**：[commercial-build-deploy.yml 第54-68行](file:///c:/Users/ycsit/Downloads/rustdesk/rustdesk-server/.github/workflows/commercial-build-deploy.yml#L54-L68)

### 3.3 触发条件矩阵

| 触发方式 | 代码检查 | 单元测试 | Docker构建 | 说明 |
|---------|---------|---------|-----------|------|
| push tag (pro-v*) | ✅ | ✅ | ✅ | 完整流程 |
| push main | ✅ | ✅ | ❌ | 仅CI |
| workflow_run | ✅ | ✅ | ✅ | 依赖CI结果 |
| 手动触发(无version) | ✅ | ✅ | ❌ | 跳过Docker |
| 手动触发(pro-v1.0.0) | ✅ | ✅ | ✅ | 完整流程 |

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

### 4.4 统一版本号配置

所有Docker相关job必须使用`pre-build`输出的版本号：

```yaml
# ✅ 正确做法
- name: Set version variables from pre-build
  run: |
    echo "GIT_TAG=${{ needs.pre-build.outputs.git_tag }}" >> $GITHUB_ENV

# ❌ 错误做法
- name: Set version variables
  run: |
    if [ "${{ github.event.inputs.version }}" != "" ]; then
      T=${{ github.event.inputs.version }}
    elif [ "${{ github.ref_type }}" = "tag" ]; then
      T=${GITHUB_REF#refs/tags/}
    else
      T="dev-${GITHUB_SHA::8}"
    fi
```

---

## Docker镜像管理规范

### 5.1 镜像标签命名规则

**格式**：`{registry}/{image}:{version}-{arch}`

**示例**：
```
ycstech/rustdesk-pro-server:pro-v1.0.0-amd64
ycstech/rustdesk-pro-server:pro-v1.0.0-arm64v8
ycstech/rustdesk-pro-server:latest-amd64
```

### 5.2 支持的架构

| 架构 | Docker平台标识 | 说明 |
|------|--------------|------|
| amd64 | linux/amd64 | x86_64 |
| arm64v8 | linux/arm64 | ARM 64-bit |
| armv7 | linux/arm/v7 | ARM 32-bit |

### 5.3 镜像仓库配置

| 仓库 | 镜像名称 | 说明 |
|------|---------|------|
| Docker Hub | `ycstech/rustdesk-pro-server` | 主镜像仓库 |
| GHCR | `ghcr.io/changsongyang/rustdesk-pro-server` | GitHub容器仓库 |

### 5.4 镜像构建配置

**配置位置**：[commercial-build-deploy.yml 第656-672行](file:///c:/Users/ycsit/Downloads/rustdesk/rustdesk-server/.github/workflows/commercial-build-deploy.yml#L656-L672)

```yaml
- name: 🐳 Build and push Docker image
  uses: docker/build-push-action@v5
  with:
    context: .
    file: ./commercial/docker/Dockerfile.gha
    platforms: ${{ matrix.job.docker_platform }}
    push: true
    provenance: false
    cache-from: type=gha
    cache-to: type=gha,mode=max
    tags: |
      ${{ env.DOCKERHUB_IMAGE }}:${{ env.LATEST_TAG }}-${{ matrix.job.name }}
      ${{ env.DOCKERHUB_IMAGE }}:${{ env.GIT_TAG }}-${{ matrix.job.name }}
      ${{ env.GHCR_IMAGE }}:${{ env.LATEST_TAG }}-${{ matrix.job.name }}
      ${{ env.GHCR_IMAGE }}:${{ env.GIT_TAG }}-${{ matrix.job.name }}
```

---

## 安全扫描要求

### 6.1 安全扫描工具

| 工具 | 用途 | 配置位置 |
|------|------|---------|
| Trivy | 漏洞扫描 | [commercial-build-deploy.yml 第681-688行](file:///c:/Users/ycsit/Downloads/rustdesk/rustdesk-server/.github/workflows/commercial-build-deploy.yml#L681-L688) |
| SBOM | 软件物料清单 | [commercial-build-deploy.yml 第697-702行](file:///c:/Users/ycsit/Downloads/rustdesk/rustdesk-server/.github/workflows/commercial-build-deploy.yml#L697-L702) |
| Cosign | 镜像签名 | [commercial-build-deploy.yml 第711-717行](file:///c:/Users/ycsit/Downloads/rustdesk/rustdesk-server/.github/workflows/commercial-build-deploy.yml#L711-L717) |

### 6.2 漏洞扫描配置

```yaml
- name: 🔍 Scan Docker image for vulnerabilities
  uses: aquasecurity/trivy-action@master
  with:
    image-ref: '${{ env.DOCKERHUB_IMAGE }}:${{ env.GIT_TAG }}-${{ matrix.job.name }}'
    format: 'sarif'
    output: 'trivy-results.sarif'
    severity: 'CRITICAL,HIGH'
    exit-code: '0'  # 仅报告，不阻止构建
```

### 6.3 SBOM生成配置

```yaml
- name: 📝 Generate SBOM for Docker image
  uses: anchore/sbom-action@v0
  with:
    image: '${{ env.DOCKERHUB_IMAGE }}:${{ env.GIT_TAG }}-${{ matrix.job.name }}'
    format: spdx-json
    output-file: 'sbom-${{ matrix.job.name }}.spdx.json'
```

### 6.4 镜像签名配置

```yaml
- name: ✍️ Sign Docker image
  uses: sigstore/cosign-installer@v3

- name: Sign and push Docker image
  run: |
    cosign sign --yes ${{ env.DOCKERHUB_IMAGE }}:${{ env.GIT_TAG }}-${{ matrix.job.name }}
    cosign sign --yes ${{ env.GHCR_IMAGE }}:${{ env.GIT_TAG }}-${{ matrix.job.name }}
```

### 6.5 安全扫描阈值

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
| trigger_time | github.event.created_at | 触发时间 |
| git_ref | github.ref | Git引用 |
| git_sha | github.sha | Git提交SHA |

### 7.2 审计日志示例

```
==============================================
           DEPLOYMENT SUMMARY
==============================================
Build ID:      20260613-143022-a1b2c3d4
Version:       pro-v1.0.0
Environment:   Production
----------------------------------------------
📝 AUDIT INFORMATION:
   Triggered by:   changsongyang
   Event Type:     push
   Trigger Time:   2026-06-13T14:30:22Z
   Git Ref:        refs/tags/pro-v1.0.0
   Git SHA:        a1b2c3d4e5f6g7h8i9j0
----------------------------------------------
✅ GitHub Release: Completed
✅ Docker Hub:     Completed
✅ GHCR:           Completed
✅ Multi-arch:     Completed
==============================================
```

### 7.3 审计日志保留

| 类型 | 保留期限 | 说明 |
|------|---------|------|
| 工作流日志 | 90天 | GitHub Actions默认 |
| Trivy扫描结果 | 30天 | 可配置 |
| SBOM | 90天 | 可配置 |

---

## 故障排除指南

### 8.1 常见错误及解决方案

#### 8.1.1 YAML语法错误

**错误信息**：
```
Invalid workflow file: .github/workflows/commercial-build-deploy.yml#L1
(Line: 254, Col: 14): The expression is not closed.
```

**解决方案**：
- 检查`${{ }}`表达式是否正确闭合
- 确保条件表达式使用`${{ }}`包裹

#### 8.1.2 secrets上下文错误

**错误信息**：
```
Unrecognized named-value: 'secrets'. Located at position 1 within expression
```

**解决方案**：
- `secrets`不能在步骤级别的`if`条件中使用
- 使用`||`运算符实现优雅失败

#### 8.1.3 镜像推送失败

**错误信息**：
```
HTTP 403: Could not delete the workflow run
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

2. **跳过特定步骤**
   - 手动触发时使用`skip-deploy`参数

3. **检查artifacts**
   - 访问GitHub Actions artifacts页面下载构建产物

---

## 附录

### A. 相关文件路径

| 文件 | 路径 |
|------|------|
| 商业版CI工作流 | `.github/workflows/commercial-ci.yml` |
| 商业版CD工作流 | `.github/workflows/commercial-build-deploy.yml` |
| 社区版工作流 | `.github/workflows/build.yaml` |
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
