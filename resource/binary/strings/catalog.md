# 字符串分析

从二进制中提取了约 131,867 个字符串，按类别整理如下。

## API 端点

### ampcode.com 端点

| 端点 | 用途 |
|------|------|
| `https://ampcode.com` | 主站 |
| `https://ampcode.com/install` | 安装引导 |
| `https://ampcode.com/manual` | 用户手册 |
| `https://ampcode.com/manual#ide` | IDE 集成文档 |
| `https://ampcode.com/manual/plugin-api` | 插件 API 参考 |
| `https://ampcode.com/manual/orbs` | Orbs 文档 |
| `https://ampcode.com/manual/appendix` | 附录 |
| `https://ampcode.com/chronicle` | 更新日志 |
| `https://ampcode.com/settings/security` | 安全设置 |
| `https://ampcode.com/threads/${t.id}` | 线程 URL 模式 |
| `https://ampcode.com/news/neo` | 产品公告 |
| `https://ampcode.com/news/stick-a-fork-in-it` | 产品公告 |

### 第三方端点

| 端点 | 用途 |
|------|------|
| `https://auth.openai.com/oauth/authorize` | OpenAI OAuth 授权 |
| `https://auth.openai.com/oauth/token` | OpenAI OAuth Token |
| `https://api.github.com` | GitHub API |

## 插件系统接口

```
@ampcode/plugin          — 插件类型定义
amp.on('session.start')  — 会话事件
amp.on('agent.start')    — Agent 事件
amp.on('agent.end')
amp.on('tool.call')      — 工具调用事件
amp.on('tool.result')
registerTool(name, def)  — 注册工具
registerCommand(...)     — 注册命令
```

## 配置项

二进制中发现的完整配置键列表：

| 键 | 类型 | 默认值 |
|----|------|--------|
| `amp.showCosts` | boolean | true |
| `amp.git.commit.ampThread.enabled` | boolean | true |
| `amp.git.commit.coauthor.enabled` | boolean | true |
| `amp.keymap` | object | {} |
| `amp.mcpServers` | object | — |
| `amp.defaultVisibility` | object | — |
| `amp.notifications.enabled` | boolean | true |
| `amp.remoteThreadCreation.enabled` | boolean | false |
| `amp.skills.disableClaudeCodeSkills` | boolean | false |
| `amp.skills.path` | string | — |
| `amp.terminal.copyOnSelect` | boolean | true |
| `amp.terminal.detailsExpandedByDefault` | boolean | false |
| `amp.thread.autoArchiveOnQuit` | boolean | false |
| `amp.tools.disable` | array | [] |
| `amp.mcpPermissions` | array | [] |
| `amp.updates.mode` | string | "auto" |
| `amp.fuzzy.alwaysIncludePaths` | array | [] |
| `amp.permissions` | boolean | — (legacy) |

## MCP 相关字符串

```
amp mcp add          — 添加 MCP 服务器
amp mcp doctor       — 检查 MCP 状态
amp mcp approve      — 批准 MCP 服务器
amp mcp oauth login  — 远程 MCP OAuth 登录
amp mcp oauth logout — 退出 OAuth
mcp registry         — MCP 注册表
```

## 嵌入文件引用 (Bun-VFS)

23 个 Node.js polyfill 模块嵌入在 bun-vfs 中：

```
assert, buffer, console, constants, crypto,
domain, events, http, https, net, os, path,
process, punycode, querystring, stream,
string_decoder, sys, timers, tty, url, util, zlib
```

## 模型名称引用

```
"gpt-5" / "gpt-5.6"           — OpenAI 模型
"claude" / "fable" / "sonnet" / "opus" / "haiku"  — Anthropic 模型
"gemini"                      — Google 模型
```

## 代理/环境变量

```
HTTP_PROXY
HTTPS_PROXY
NODE_EXTRA_CA_CERTS
AMP_API_KEY
AMP_SKIP_UPDATE_CHECK
AMP_HOME
AMP_FORCE_BEL
```

## 构建引用

```
"version": "0.0.1"            — 内部版本标识
bake://server-runtime.js       — Bun Bake 服务端运行时
packages/bun-usockets/src/crypto/root_certs.cpp  — Bun 源码路径
```
