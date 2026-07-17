# 工具集

> 来源：[https://ampcode.com/manual](https://ampcode.com/manual)

amp 可代表你运行工具和 shell 命令。默认无需审批即可执行。

内置工具列表：`amp tools list`

## 安全说明

不受信任的仓库、MCP 服务器和外部输入可能影响 amp 行为。建议使用自定义策略插件或隔离开发环境。

## Agent Skills

Skills 是指令和资源的打包集合，教 agent 特定任务。详见 [skills 章节](13-skills.md)。

## 子 Agent（Subagents）

amp 自动为受益于独立执行环境的复杂任务生成子 agent。

### 特性

- 每个子 agent 拥有独立的上下文窗口和工具权限
- 最适合：多步独立子任务、产生大量输出但后续不需要的操作、跨代码区域的并行工作、保持主线程上下文简洁
- **限制**：子 agent 之间不能通信、不能中途指导、从零开始无累积上下文、主 agent 只获取最终摘要

amp 主要在 `medium` 模式下使用子 agent。提示词中提及子 agent 或建议并行工作会触发此功能。

## Oracle

用于复杂推理/分析的"第二意见"模型——稍慢但更深入。

- 工具名：`oracle`
- 默认模型：GPT-5.6 Sol（reasoning level high）
- `high` 模式下（GPT-5.6 Sol 是主模型），oracle 切换到 Claude Fable 5
- 主 agent 自主决定是否调用；建议明确要求 amp 使用 oracle

### 示例

```text
"Use the oracle to review the last commit's changes..."
"Ask the oracle whether there isn't a better solution."
"Help me fix this bug. Use the oracle as much as possible, since it's smart."
"Analyze how the functions foobar and barfoo are used... work with the oracle to refactor duplication."
```

## Librarian

搜索远程代码库——所有公开的 GitHub 代码和私有仓库。

- 仅搜索默认分支
- 需在设置中配置 GitHub 连接
- 对于私有仓库，在 GitHub 连接设置时选择
- 返回更详细的长答案
- 可能需要明确提示 agent 使用

### 示例

```text
"Explain how new versions of our documentation are deployed... search our docs and infra repositories."
"Ask the Librarian to investigate why the error is happening..."
"Use the Librarian to investigate the foo service — were there any recent changes to the API endpoints?"
```

## Painter

通过 GPT Image 2 生成和编辑图片。

- 可创建 UI 线框图、应用图标、主图、编辑现有图片（如遮盖敏感信息）
- 通过 `@` 引用最多 3 张参考图片
- 可能需要明确提示 agent 使用

### 示例

```text
"Use the painter to create a UI mockup for my settings page."
"Generate an app icon... Dark background with a glowing terminal cursor in cyan."
"Use the painter to redact any visible API keys or passwords."
```
