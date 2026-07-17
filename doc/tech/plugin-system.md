# 插件系统逆向分析

> 基于二进制字符串分析还原的插件系统架构。

## PluginAPI 核心类型

从字符串中提取的插件 API 接口签名：

```typescript
interface PluginAPI {
  on(event: 'session.start' | 'agent.start' | 'agent.end' | 'tool.call' | 'tool.result', handler: EventHandler): void;
  registerTool(def: ToolDefinition): void;
  registerCommand(name: string, opts: CommandOptions, handler: CommandHandler): CommandSubscription;
  ai: { ask(prompt: string): Promise<boolean> };
  logger: { log(message: string): void };
  $: (strings: TemplateStringsArray, ...values: any[]) => Promise<ShellResult>;
  experimental: {
    createAgent(config: AgentConfig): AgentHandle;
    registerAgentMode(config: AgentModeConfig): void;
  };
  getBuiltinAgent(mode: 'low' | 'medium' | 'high' | 'ultra'): AgentHandle;
}
```

## 事件生命周期

```
session.start  ─── 线程启动/切换
     │
agent.start    ─── 用户提交提示词
     │
  ┌── tool.call ─── 工具执行前（可拦截/修改）
  │      │
  │  tool.result ── 工具执行后（可修改结果）
  │      │
  └── (循环至完成)
     │
agent.end      ─── Agent 完成一轮
```

## 命令注册

注册的命令通过命令面板（`Ctrl+O`）触发：

```typescript
interface CommandOptions {
  title: string;         // 显示名称
  category: string;      // 分类
  description: string;   // 描述
}
```

可用性控制：

```typescript
subscription.setAvailability({
  type: 'enabled' | 'disabled' | 'hidden',
  reason?: string        // disabled 时的理由
});
```

## 工具注册

```typescript
interface ToolDefinition {
  name: string;
  description: string;
  inputSchema: JSONSchema;
  execute(input: any): Promise<any>;
}
```

## 自定义 Agent

通过 experimental API 创建自定义 Agent：

```typescript
// 需要 // @amp-agent-mode 元数据注释
// @amp-agent-mode {"key":"architect","label":"architect"}

const agent = amp.experimental.createAgent({
  name: string;
  model: string;               // 如 'openai/gpt-5.5'
  instructions: string;        // 系统提示词
  tools: 'all' | string[];    // 可用工具
  reasoningEffort: string;     // 推理强度
  parentThreadID?: string;     // 关联到父线程
});

// 使用
agent.run(message, options?);
agent.createThread(options?);
```

## 内置 Agent

```typescript
// 获取内置 Agent
const agent = amp.getBuiltinAgent('medium');
await agent.run('检查代码质量');
```

## UI API

```typescript
interface UIContext {
  notify(message: string): void;
  confirm(message: string): Promise<boolean>;
  input(prompt: string): Promise<string>;
  select(options: SelectOption[]): Promise<string>;
  system: {
    open(url: string): Promise<void>;
  };
}
```

## Shell 执行

```typescript
// amp.$ 是一个模板字符串函数
const result = await amp.$`git status --short`;
// { stdout: string, stderr: string, exitCode: number }
```

## 安全（Plugin 示例）

内置的权限插件（从字符串还原）展示了如何使用 `amp.ai.ask()` 做 AI 分类的安全控制：

```typescript
amp.on('tool.call', async (ctx) => {
  // 识别 git 命令
  // 安全命令：status, log, diff, show, branch -a/v, stash list,
  //          remote -v, fetch, pull, add, commit, push (无 force)
  // 危险命令：其他 → 用 amp.ai.ask() 判断风险
  // 根据风险等级选择：允许 / AI 判断 / 阻止
});
```
