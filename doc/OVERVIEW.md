# amp 概述

> 来源：[ampcode.com](https://ampcode.com/)，整理于 2026-07-17

## 什么是 amp？

**amp** 是一款前沿 AI 编程助手（coding agent），可在 Web、终端（Mac/Linux/WSL）和手机端使用。它运行 AI 驱动的编程 agent，可以在本地或远程机器（称为 **Orb**）上工作，用户可以从任何设备发起线程并在另一设备上继续。

核心定位："Frontier coding agent built for leading models, and what comes next."

## 核心特性

| 特性 | 说明 |
|------|------|
| **Orbs** | 远程机器上持续运行的 agent，关闭本地电脑后仍在工作 |
| **跨平台** | Web、终端、手机三端可用 |
| **Dial Modes** | 四种 agent 模式（low / medium / high / ultra），控制 agent 工作强度 |
| **可扩展** | 插件系统：拦截事件、添加工具、标准化策略 |
| **大线程** | 可读写并回答任意大小线程的问题 |
| **自定义 Agent** | 插件可创建自定义 agent 并维持连续对话 |
| **按量付费** | 个人用户无加价 |

## 设计理念

1. **多模型** — 使用 GPT-5.6、Claude Fable 5、快速模型等，各取所长
2. **有主见** — "如果团队自己不用且不喜欢某个特性，就删掉它"
3. **前沿先行** — "没有向后兼容包袱，没有遗留特性"
4. **线程机制** — 可保存、分享交互过程，类似代码的版本控制

## 架构原则

- **不限制 token 用量**
- **始终使用最佳模型**
- **提供原始模型能力**
- **随新模型持续进化**

## 相关工作流

amp 包含几大组件协同工作：

```
用户 → CLI / Web / Mobile
        ↓
    amp Agent (低/中/高/超 模式)
        ↓
   ┌───┼───┬──────┬──────┬──────┐
   │   │   │      │      │      │
 工具 子Agent Oracle Librarian Painter
        ↓
   MCP 服务器 / 插件 / Skills
        ↓
   本地代码 / 远程 Orbs
```

## 参考链接

- 官网：<https://ampcode.com/>
- 手册：<https://ampcode.com/manual>
- SDK：<https://ampcode.com/manual/sdk>
- 模型说明：<https://ampcode.com/models>
- 安全：<https://ampcode.com/security>
