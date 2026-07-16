# 代码审查

> 来源：[https://ampcode.com/manual](https://ampcode.com/manual)

审查代码中的 bug、安全、性能、风格违规。

## CLI 方式

```bash
amp review
```

或让主 agent 审查代码。

## 审查机制

amp 支持用户定义的审查标准，可按代码库区域划分范围。审查时，amp 为每个检查项生成一个子 agent。

### 检查项格式

检查项是 Markdown 文件，放置在 `.agents/checks/` 目录中：

```yaml
---
name: performance
description: Flags common performance anti-patterns
severity-default: medium
tools: [Grep, Read]
---
```

| 字段 | 必填 | 说明 |
|------|------|------|
| `name` | 是 | 检查项名称 |
| `description` | 否 | 描述 |
| `severity-default` | 否 | 默认严重级别：low / medium / high / critical |
| `tools` | 否 | 该检查项可用的工具列表 |

### 检查项位置

| 路径 | 作用域 |
|------|--------|
| `.agents/checks/` | 整个代码库 |
| `api/.agents/checks/` | 仅 `api/` 目录下的文件 |
| `$HOME/.config/amp/checks/` 或 `$HOME/.config/agents/checks/` | 全局 |

子项目中的同名检查项会覆盖父目录和全局中的检查项。
