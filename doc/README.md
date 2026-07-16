# doc — amp 技术文档

> 本目录存放 amp 官方文档和逆向工程整理的完整技术文档。

## 目录结构

```
doc/
├── README.md                    # 目录索引
├── OVERVIEW.md                  # amp 概述与核心特性
│
├── manual/                      # 官方文档（来自 ampcode.com/manual）
│   ├── 01-getting-started.md    # 安装与 IDE 集成
│   ├── 02-usage.md              # 使用指南与提示词技巧
│   ├── 03-agents-md.md          # AGENTS.md 配置
│   ├── 04-projects.md           # 项目管理
│   ├── 05-orbs-runners.md       # Orbs 与远程执行
│   ├── 06-tools.md              # Agent 工具集
│   ├── 07-plugins.md            # 插件系统
│   ├── 08-mcp.md                # MCP 集成
│   ├── 09-configuration.md      # 配置参考
│   ├── 10-pricing.md            # 定价与商业版
│   ├── 11-cli.md                # CLI 参考
│   ├── 12-code-review.md        # 代码审查
│   └── 13-skills.md             # Agent Skills
│
└── tech/                        # 逆向工程技术文档
    ├── architecture.md           # 系统架构分析
    ├── plugin-system.md          # 插件系统还原
    └── protocol.md               # 通信协议分析
```

## 来源

- 官方文档: <https://ampcode.com/manual>
- 官网: <https://ampcode.com/>
- 逆向分析: 基于 amp CLI v0.0.1784219219-g3e9560 二进制分析
