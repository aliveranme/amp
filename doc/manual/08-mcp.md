# MCP 集成

> 来源：[https://ampcode.com/manual](https://ampcode.com/manual)

通过本地或远程 MCP 服务器向 amp 添加工具。

**最佳实践**：将 MCP 打包在 Skills 的 `mcp.json` 中，而非添加到全局配置——工具在 skill 加载前保持隐藏。

## CLI 命令

```bash
# 添加本地 MCP 服务器
amp mcp add context7 -- npx -y @upstash/context7-mcp

# 添加远程 MCP 服务器
amp mcp add linear https://mcp.linear.app/sse
```

## 配置格式

```json
"amp.mcpServers": {
    "playwright": {
        "command": "npx",
        "args": ["-y", "@playwright/mcp@latest", "--headless"]
    },
    "linear": {
        "url": "https://mcp.linear.app/sse"
    },
    "sourcegraph": {
        "url": "${SRC_ENDPOINT}/.api/mcp/v1",
        "headers": { "Authorization": "token ${SRC_ACCESS_TOKEN}" }
    }
}
```

使用 `${VAR_NAME}` 引用环境变量。

## 加载顺序（优先级从高到低）

1. CLI 标志（`--mcp-config`）
2. 用户/工作区设置（`amp.mcpServers`）
3. Skills（如未在上层配置则加载）

## 工作区 MCP 信任

- `.amp/settings.json` 中的 MCP 服务器需明确批准才能运行
- 检查状态：`amp mcp doctor`
- 批准：`amp mcp approve my-server`
- 全局设置或 `--mcp-config` 中的服务器不需要审批

## Skills 中的 MCP 配置

```json
{
  "chrome-devtools": {
    "command": "npx",
    "args": ["-y", "chrome-devtools-mcp@latest"],
    "includeTools": ["navigate_*", "take_screenshot", "click", "fill*"]
  }
}
```

远程服务器格式：

```json
{
  "linear": {
    "url": "https://mcp.linear.app/sse",
    "includeTools": ["list_issues", "create_issue", "update_issue"]
  }
}
```

### 字段说明

- **本地**：`command`（字符串，必填）、`args`（字符串数组，可选）、`env`（对象，可选）
- **远程**：`url`（字符串，必填）、`headers`（对象，可选）
- **通用**：`includeTools`（字符串数组——工具名或 glob 模式；推荐使用）

## 远程 MCP OAuth

部分服务器（如 Linear）支持自动 OAuth——启动时自动触发浏览器流程。

手动注册：
```bash
amp mcp oauth login my-server \
  --server-url https://... \
  --client-id ... \
  --client-secret ... \
  --scopes "openid,profile,email,user:all"
```

- 回调 URI：`http://localhost:8976/oauth/callback`
- Token 存储在 `~/.amp/oauth/`，自动刷新
- 清除凭据：`amp mcp oauth logout my-server`
