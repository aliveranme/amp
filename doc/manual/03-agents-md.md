# AGENTS.md

> 来源：[https://ampcode.com/manual](https://ampcode.com/manual)

AGENTS.md 用于指导 amp 理解代码库结构、构建/测试命令和约定规范。

## 文件位置与优先级

| 位置 | 作用域 |
|------|--------|
| 当前目录、父目录、子树的 `AGENTS.md` | 架构、命令、API 概览、审查步骤 |
| `$HOME/.config/amp/AGENTS.md` / `$HOME/.config/AGENTS.md` | 个人偏好、本地测试配置 |
| `/etc/ampcode/AGENTS.md` / macOS `Library` 路径 / Windows `ProgramData` 路径 | 系统级/组织级管理 |

- 当前目录和父目录（直到 `$HOME`）中的 AGENTS.md 始终包含
- 子树中的 AGENTS.md 在 agent 读取该目录下文件时包含
- 若 AGENTS.md 不存在，回退到 `AGENT.md`（无 S）或 `CLAUDE.md`

## 查看生效文件

命令面板 → `agents-md list`

## 编写 AGENTS.md

- amp 会自动生成（如果没有）
- 使用 `@` 引用文件：`See @doc/style.md and @specs/**/*.md.`
- 相对路径相对于 agent 文件所在目录；支持绝对路径和 `@~/`
- 代码块中的 `@` 引用会被忽略
- 支持 glob 模式（`@doc/*.md`, `@.agent/**/*.md`）

### YAML 前置元数据

```yaml
---
globs: ["src/components/**", "**/*.tsx"]
---
```

YAML 中的 `globs` 可限定指导范围仅适用于特定文件类型的操作。

- globs 默认带 `**/` 前缀，除非以 `../` 或 `./` 开头
- 没有 globs 的文件在被 `@` 引用时始终包含

### 从其他工具迁移

| 来源 | 迁移命令 |
|------|---------|
| **Claude Code** | `mv CLAUDE.md AGENTS.md && ln -s AGENTS.md CLAUDE.md` |
| **Cursor** | `mv .cursorrules AGENTS.md && ln -s AGENTS.md .cursorrules` 并添加 `@.cursor/rules/*.mdc` |
| **AGENT.md** | `mv AGENT.md AGENTS.md`（两个名称都可继续使用） |
