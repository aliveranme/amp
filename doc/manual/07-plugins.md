# 插件系统

> 来源：[https://ampcode.com/manual](https://ampcode.com/manual)

插件是 TypeScript 文件，通过导出接收 `PluginAPI` 的默认函数来扩展 amp。

## 基础结构

```typescript
import type { PluginAPI } from '@ampcode/plugin'

export default function (amp: PluginAPI) {
    amp.logger.log('Plugin initialized')
}
```

## 插件位置

| 类型 | 路径 | 范围 |
|------|------|------|
| 项目 | `.amp/plugins/*.ts` | 仅在该项目运行时生效 |
| 系统 | `~/.config/amp/plugins/*.ts` | 跨项目个人使用 |
| 全局 | 在工作区设置中配置 | 工作区所有成员（有限公开实验） |

## 何时选什么

| 方式 | 适用场景 |
|------|---------|
| AGENTS.md | 持久性指令 |
| Skills | 任务特定指导 |
| 插件 | 自定义工具或事件驱动行为 |
| MCP | 集成 MCP 服务器 |

## 能力

### 事件生命周期

```
session.start → agent.start → [tool.call → tool.result (per tool)] → agent.end
```

| 事件 | 时机 | 用途 |
|------|------|------|
| `session.start` | 线程会话启动 | 初始化 |
| `tool.call` | 工具运行前 | 返回 `allow`、`reject-and-continue`、`modify`（修改输入）或 `synthesize`（提供结果不运行） |
| `tool.result` | 工具完成后 | 返回原始结果或替代状态/输出 |
| `agent.start` | 用户提交提示词时 | 预处理 |
| `agent.end` | agent 完成一轮时 | 返回 `continue` + 用户消息开启下一轮（**必须包含标记/守卫防止无限循环**） |

### 注册命令

```typescript
amp.registerCommand(
    'open-plugin-docs',
    {
        title: 'Open plugin docs',
        category: 'docs',
        description: 'Open the Amp Plugin API manual page.',
    },
    async (ctx) => {
        await ctx.system.open('https://ampcode.com/manual/plugin-api')
    },
)
```

命令可用性控制：`registerCommand` 返回的 subscription 有 `setAvailability(...)`：
- `{ type: 'enabled' }` — 显示且可选（默认）
- `{ type: 'disabled', reason: '...' }` — 显示但不可选
- `{ type: 'hidden' }` — 隐藏

### 注册工具

```typescript
amp.registerTool({
    name: 'project_status',
    description: 'Show the current git status for this repository.',
    inputSchema: {
        type: 'object',
        properties: {},
        required: [],
    },
    async execute() {
        const result = await amp.$`git status --short`
        return result.stdout || 'No changes.'
    },
})
```

### 自定义 Agent 模式

通过 `amp.experimental.createAgent(...)` 和 `amp.experimental.registerAgentMode(...)`。

需要 `// @amp-agent-mode` 元数据注释：

```typescript
// @amp-agent-mode {"key":"architect","label":"architect"}
```

限制：模式键和标签 ≤16 字符、非空、唯一、不冲突内置模式。

### 自定义子 Agent

```typescript
const reviewer = amp.experimental.createAgent({
    name: 'focused-reviewer',
    model: 'openai/gpt-5.5',
    instructions: '...',
    tools: 'all',
    reasoningEffort: 'medium',
})
```

`parentThreadID` 选项使子 agent 关联到调用线程。

### 内置 Agent 句柄

`amp.getBuiltinAgent(mode)` 适用于 'low'、'medium'、'high'、'ultra'。已弃用的 'smart'、'deep'、'rush' 仍可接受（映射到替换模式）。

支持的 `run(message, options?)` 和 `createThread(options?)` 方法。 `executor: 'orb'` 用于 orb 支持的线程，或 `executor: { type: 'runner', id }` 用于实况 runner。

### UI 功能

- `ctx.ui.notify()` — 通知
- `ctx.ui.confirm()` — 确认对话框
- `ctx.ui.input()` — 输入框
- `ctx.ui.select()` — 选择框

### AI 分类

`amp.ai.ask(...)` 用于线程作用域的 yes/no 决策。

## 实用插件示例

手册中包含一个完整的权限插件示例——使用 `amp.ai.ask(...)` 分类 git 命令的风险等级，在破坏性操作前提示用户，允许安全模式（status、log、diff、show 等）无需确认。

## 插件生命周期管理

- 修改后：`Ctrl+O` → `plugins: reload`
- 查看已加载插件：`plugins: list`
- 插件界面在 TUI 和 Web 上同步显示

## 参考

完整 Plugin API 参考：<https://ampcode.com/manual/plugin-api>
