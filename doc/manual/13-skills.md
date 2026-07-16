# Agent Skills

> 来源：[https://ampcode.com/manual](https://ampcode.com/manual)

Skills 是指令和资源的打包集合，教会 agent 特定任务。

## Skill 位置（优先级从高到低）

1. `~/.config/agents/skills/`
2. `~/.agents/skills/`
3. `~/.config/amp/skills/`
4. `.agents/skills/`
5. `.claude/skills/`
6. `~/.claude/skills/`
7. 插件、遗留工具箱目录、内置 skills

## 创建 Skill

- 使用内置的 `building-skills` skill 可创建定制 skill
- "Create a skill for deploying to staging"
- "Project-specific skill" → 保存到 `.agents/skills/`（提交到 git 供团队共享）
- "User-wide skill" → 个人所有，跨项目

查看 skills：命令面板 → `skill: list`

## Skill 格式

目录中包含 `SKILL.md`，使用 YAML 前置元数据：

```yaml
---
name: my-skill
description: A description of what this skill does
---
```

- `name` 和 `description` 总是对模型可见；内容在调用时按需加载
- 名称必须唯一（项目 > 用户级 > 内置）
- 可捆绑资源（脚本、模板）到同一目录

## Skills 中的 MCP 服务器

在 skill 目录中包含 `mcp.json`：

```json
{
  "chrome-devtools": {
    "command": "npx",
    "args": ["-y", "chrome-devtools-mcp@latest"],
    "includeTools": ["navigate_*", "take_screenshot", "click", "fill*"]
  }
}
```

- 服务器在启动时启动，工具在 skill 加载前保持隐藏
- 比添加到全局配置更优，保持工具列表整洁
