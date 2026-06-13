# Workflow 快速参考卡

## 触发方式速查

| 场景 | 触发方式 | 命令/操作 |
|------|---------|----------|
| 完整发布 | push tag | `git tag pro-v1.0.0 && git push origin pro-v1.0.0` |
| 手动发布 | workflow_dispatch | GitHub Actions → workflow_dispatch |
| 跳过Docker | 手动无version | workflow_dispatch 不填version |
| 紧急发布 | 手动skip-ci | workflow_dispatch 勾选skip-ci-check |

## 版本号速查

| 输入 | 输出格式 | 示例 |
|------|---------|------|
| pro-v1.0.0 | pro-v1.0.0 | Docker: pro-v1.0.0-amd64 |
| pro-1.0.0 | 1.0.0 | Debian: 1.0.0 |
| v1.0.0 | 1.0.0 | Debian: 1.0.0 |
| (空) | dev-{sha8} | Docker: dev-a1b2c3d4-amd64 |

## 工作流状态速查

```bash
# 查看运行状态
gh run list --repo changsongyang/rustdesk-server --created 2026-06-13 --limit 5

# 取消运行
gh run cancel <run-id> --repo changsongyang/rustdesk-server

# 删除运行
gh run delete <run-id> --repo changsongyang/rustdesk-server

# 查看日志
gh run view <run-id> --log --repo changsongyang/rustdesk-server
```

## Docker镜像速查

| 镜像 | 标签示例 |
|------|---------|
| Docker Hub | `ycstech/rustdesk-pro-server:pro-v1.0.0-amd64` |
| GHCR | `ghcr.io/changsongyang/rustdesk-pro-server:pro-v1.0.0-amd64` |
| Latest | `ycstech/rustdesk-pro-server:latest-amd64` |

## 触发条件检查表

- [ ] push tag到main分支？
- [ ] tag格式符合pro-v*或pro-*？
- [ ] CI工作流已通过？
- [ ] Secrets已配置（DOCKER_HUB_*）？
- [ ] 手动触发时是否需要skip-ci？

## 安全扫描检查表

- [ ] Trivy扫描结果已上传（artifact）？
- [ ] SBOM已生成（SPDX格式）？
- [ ] 镜像签名已完成（Cosign）？
- [ ] 无CRITICAL漏洞？

## 审计记录检查表

- [ ] 触发人已记录？
- [ ] 触发时间已记录？
- [ ] Git引用已记录？
- [ ] 执行结果已记录？
