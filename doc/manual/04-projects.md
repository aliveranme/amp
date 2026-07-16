# 项目

> 来源：[https://ampcode.com/manual](https://ampcode.com/manual)

项目将仓库关联到 amp 的设置、密钥和线程。项目属于用户或工作区。

- Orb 线程可选（可使用 "No Project"）
- CLI 线程通过匹配 Git 远程 URL 自动关联到项目
- **仓库**：可以是已有仓库（GitHub 或任意 Git URL）或 amp 托管仓库
- 克隆 amp 托管仓库：`amp clone owner/project-name`

## 更改工作流（Changes Workflow）

| 模式 | 行为 |
|------|------|
| **Ship** | 直接提交并推送到 `origin/main` |
| **Push to Branch** | 推送到当前分支；GitHub 上返回 PR 链接 |

如需自动创建 PR，在 AGENTS.md 中添加："Create pull requests with `gh pr create`..."（orb 中已预装 `gh`）。
