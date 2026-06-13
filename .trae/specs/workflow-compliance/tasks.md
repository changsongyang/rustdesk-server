# Workflow合规化规范管理方案 - 实施计划

## [x] Task 1: 构建打包流程标准化
- **Priority**: P0
- **Depends On**: None
- **Description**: 
  - 定义标准化的CI流程：代码格式检查(rustfmt) → 静态分析(clippy) → 单元测试 → 集成测试 → 版本号生成 → 打包输出
  - 确保所有检查项通过后才能进入CD流程
- **Acceptance Criteria Addressed**: AC-1
- **Test Requirements**:
  - `programmatic` TR-1.1: CI工作流必须包含rustfmt检查步骤 ✅
  - `programmatic` TR-1.2: CI工作流必须包含clippy静态分析步骤 ✅
  - `programmatic` TR-1.3: CI工作流必须包含单元测试步骤 ✅
  - `human-judgement` TR-1.4: 检查CI流程是否按顺序执行 ✅

## [x] Task 2: 版本号生成规则标准化
- **Priority**: P0
- **Depends On**: Task 1
- **Description**: 
  - 统一版本号来源：从Git tag或手动输入获取
  - 定义版本号格式规范：`pro-vX.Y.Z` 或 `vX.Y.Z`
  - 确保所有平台使用统一的版本号
- **Acceptance Criteria Addressed**: AC-1
- **Test Requirements**:
  - `programmatic` TR-2.1: 版本号必须从pre-build job输出 ✅
  - `programmatic` TR-2.2: Docker镜像标签必须使用统一版本号 ✅
  - `programmatic` TR-2.3: Debian包版本号必须与镜像版本一致 ✅

## [x] Task 3: Docker镜像标签命名规范
- **Priority**: P0
- **Depends On**: Task 2
- **Description**: 
  - 定义镜像标签格式：`{image_name}:{version}-{arch}`
  - 添加latest标签作为稳定版本标识
  - 支持多架构镜像构建
- **Acceptance Criteria Addressed**: AC-2
- **Test Requirements**:
  - `programmatic` TR-3.1: Docker镜像必须包含版本标签 ✅
  - `programmatic` TR-3.2: Docker镜像必须包含latest标签 ✅
  - `programmatic` TR-3.3: 多架构镜像标签格式必须统一 ✅

## [x] Task 4: Docker镜像安全扫描集成
- **Priority**: P0
- **Depends On**: Task 3
- **Description**: 
  - 集成Trivy安全扫描工具
  - 配置CRITICAL和HIGH级别漏洞检查
  - 生成安全扫描报告并保存为artifact
- **Acceptance Criteria Addressed**: AC-3
- **Test Requirements**:
  - `programmatic` TR-4.1: Docker构建后必须执行Trivy扫描 ✅
  - `programmatic` TR-4.2: 扫描结果必须保存为artifact ✅
  - `human-judgement` TR-4.3: 检查扫描配置是否正确 ✅

## [x] Task 5: SBOM生成规范
- **Priority**: P1
- **Depends On**: Task 4
- **Description**: 
  - 集成SBOM（软件物料清单）生成工具
  - 使用SPDX格式输出SBOM
  - 保存SBOM作为构建产物
- **Acceptance Criteria Addressed**: AC-3
- **Test Requirements**:
  - `programmatic` TR-5.1: 每个Docker镜像必须生成SBOM ✅
  - `programmatic` TR-5.2: SBOM必须为SPDX格式 ✅
  - `human-judgement` TR-5.3: 检查SBOM内容完整性 ✅

## [x] Task 6: Docker镜像签名规范
- **Priority**: P1
- **Depends On**: Task 5
- **Description**: 
  - 集成Cosign镜像签名工具
  - 配置签名密钥管理
  - 确保镜像完整性和真实性
- **Acceptance Criteria Addressed**: AC-3
- **Test Requirements**:
  - `programmatic` TR-6.1: Docker镜像必须经过签名 ✅
  - `human-judgement` TR-6.2: 检查签名配置是否正确 ✅

## [x] Task 7: 自动触发条件规范
- **Priority**: P0
- **Depends On**: Task 1
- **Description**: 
  - 定义触发分支策略：main分支和特定tag模式
  - 配置workflow_run触发CI完成后的CD流程
  - 实现环境隔离：开发/测试/生产环境分离
- **Acceptance Criteria Addressed**: AC-4
- **Test Requirements**:
  - `programmatic` TR-7.1: 只有pro-v*和pro-*标签才能触发完整CD ✅
  - `programmatic` TR-7.2: workflow_run触发必须检查CI状态 ✅
  - `human-judgement` TR-7.3: 检查触发条件配置是否符合规范 ✅

## [x] Task 8: 手动触发权限控制
- **Priority**: P0
- **Depends On**: Task 7
- **Description**: 
  - 添加skip-ci-check参数用于紧急发布
  - 添加skip-deploy参数用于测试构建
  - 实现触发前检查项：CI状态、分支保护、审核状态
- **Acceptance Criteria Addressed**: AC-5
- **Test Requirements**:
  - `programmatic` TR-8.1: 手动触发必须有version参数选项 ✅
  - `programmatic` TR-8.2: skip-ci-check参数必须为布尔类型 ✅
  - `human-judgement` TR-8.3: 检查参数配置是否合理 ✅

## [x] Task 9: 操作审计记录机制
- **Priority**: P1
- **Depends On**: Task 8
- **Description**: 
  - 记录触发人信息（github.actor）
  - 记录触发时间和事件类型
  - 记录执行结果和关键参数
- **Acceptance Criteria Addressed**: AC-6
- **Test Requirements**:
  - `programmatic` TR-9.1: 工作流必须输出触发人信息 ✅
  - `programmatic` TR-9.2: 工作流必须输出执行时间戳 ✅
  - `human-judgement` TR-9.3: 检查审计信息完整性 ✅

## [x] Task 10: 社区版与商业版隔离机制
- **Priority**: P0
- **Depends On**: Task 1
- **Description**: 
  - 分离社区版和商业版的工作流文件
  - 定义不同的输出产物路径
  - 确保商业版特有功能不泄露到社区版
- **Acceptance Criteria Addressed**: AC-7
- **Test Requirements**:
  - `programmatic` TR-10.1: 社区版和商业版工作流必须独立 ✅
  - `programmatic` TR-10.2: 输出产物路径必须隔离 ✅
  - `human-judgement` TR-10.3: 检查隔离机制有效性 ✅

## [x] Task 11: 合规文档和操作指南
- **Priority**: P2
- **Depends On**: 所有任务
- **Description**: 
  - 编写合规规范文档
  - 编写操作指南
  - 编写故障排除指南
- **Acceptance Criteria Addressed**: 所有AC
- **Test Requirements**:
  - `human-judgement` TR-11.1: 文档必须完整覆盖所有流程 ✅
  - `human-judgement` TR-11.2: 操作指南必须清晰易懂 ✅
  - `human-judgement` TR-11.3: 故障排除指南必须实用 ✅

---

## 实施总结

| 任务 | 状态 | 完成时间 |
|------|------|---------|
| Task 1: 构建打包流程标准化 | ✅ 完成 | 2026-06-13 |
| Task 2: 版本号生成规则标准化 | ✅ 完成 | 2026-06-13 |
| Task 3: Docker镜像标签命名规范 | ✅ 完成 | 2026-06-13 |
| Task 4: Docker镜像安全扫描集成 | ✅ 完成 | 2026-06-13 |
| Task 5: SBOM生成规范 | ✅ 完成 | 2026-06-13 |
| Task 6: Docker镜像签名规范 | ✅ 完成 | 2026-06-13 |
| Task 7: 自动触发条件规范 | ✅ 完成 | 2026-06-13 |
| Task 8: 手动触发权限控制 | ✅ 完成 | 2026-06-13 |
| Task 9: 操作审计记录机制 | ✅ 完成 | 2026-06-13 |
| Task 10: 社区版与商业版隔离机制 | ✅ 完成 | 2026-06-13 |
| Task 11: 合规文档和操作指南 | ✅ 完成 | 2026-06-13 |
