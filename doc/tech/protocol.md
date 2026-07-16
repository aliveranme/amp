# 协议分析

> amp 与后端通信、OAuth 流程、MCP 集成的协议分析。

## 1. 核心 API 协议

### REST API

| 端点 | 方法 | 用途 |
|------|------|------|
| `ampcode.com/api/*` | 推测 RESTful | 核心业务 API |

### GraphQL

二进制中包含 GraphQL 操作字符串（见 `strings/graphql-operations.txt`），涉及：

- `Thread` — 线程 CRUD
- `User` — 用户管理
- `Auth` — 认证
- `Config` — 配置同步
- `Project` — 项目管理
- `Orb` — 远程执行

### WebSocket

- **HMR 连接**: 本地开发模式的 WebSocket 热替换
- **实时通信**: 远程线程 / Orbs 的可能通信通道

## 2. OAuth 2.0 / BYOK

### OpenAI OAuth 流程

```
用户请求使用自有 API Key
    │
    ▼
amp 打开浏览器到 auth.openai.com/oauth/authorize
    │  (带有回调 URL)
    │
    ▼
用户登录 OpenAI，授权
    │
    ▼
回调到 ampcode.com 或 localhost
    │
    ▼
amp 交换 authorization code 为 access token
    │  POST auth.openai.com/oauth/token
    │
    ▼
token 存储在本地 ~/.amp/ 或系统密钥链
```

### Token 存储

```env
AMP_API_KEY=sk-...  # 环境变量方式
```

### 安全设置

`https://ampcode.com/settings/security#access-token`

## 2b. BYOK 模型路由协议（amp code CLI 实现方案）

BYOK 的核心机制是 **API Key 透传 + 模型名到 Provider 的路由映射**。

### 环境变量接口

```env
# 必需
AMP_API_KEY=<openai-sk-or-other-provider-key>
AMP_URL=<https://your-amp-instance.com>

# 可选
AMP_MODEL_DEFAULT=gpt-4o              # 默认模型
AMP_MODEL_ROUTE=./route-config.toml    # 自定义路由配置文件
```

### 模型路由流程

```
客户端请求 (model=gpt-4o, prompt=...)
    │
    ▼
amp code CLI (路由层)
    │
    ├── 1. 解析请求中的 model 字段
    ├── 2. 匹配路由表（本地配置 / 内置映射）
    ├── 3. 注入 API Key（从 AMP_API_KEY 取出）
    │
    ▼
    ├──→ OpenAI:    POST api.openai.com/v1/chat/completions
    ├──→ Anthropic: POST api.anthropic.com/v1/messages
    ├──→ 自定义:    POST <AMP_URL>/v1/chat/completions
    │
    ▼
响应流 (SSE) 透传回客户端
```

### 路由表示例

```toml
# route-config.toml
[routes]
"gpt-4o" = { provider = "openai", endpoint = "https://api.openai.com/v1/chat/completions" }
"gpt-4o-mini" = { provider = "openai", endpoint = "https://api.openai.com/v1/chat/completions" }
"claude-sonnet-4" = { provider = "anthropic", endpoint = "https://api.anthropic.com/v1/messages" }
"claude-fable-5" = { provider = "anthropic", endpoint = "https://api.anthropic.com/v1/messages" }

# 通配兜底
"*" = { provider = "openai", endpoint = "https://api.openai.com/v1/chat/completions" }

[headers]
# 根据 provider 注入不同认证头
"openai" = { Authorization = "Bearer ${API_KEY}" }
"anthropic" = { x-api-key = "${API_KEY}", anthropic-version = "2023-06-01" }
```

### 流式代理设计

```
CLI 客户端                     amp code BYOK Proxy              LLM Provider
    │                                │                              │
    ├── POST /v1/chat/completions ──►│                              │
    │   (stream: true)               │                              │
    │                                ├── POST /v1/chat/completions ─►│
    │                                │   (Authorization: Bearer sk) │
    │                                │   ◄── SSE stream ────────────│
    │   ◄── SSE stream ─────────────│                              │
    │                                │                              │
```

### 多 API Key 管理

```env
# 单一全局 Key（简化模式）
AMP_API_KEY=sk-...

# 多 Provider Key（进阶模式，通过配置文件）
AMP_API_KEY_FILE=./api-keys.toml
```

```toml
# api-keys.toml
[keys]
openai = "sk-..."
anthropic = "sk-ant-..."

## 3. MCP 协议

### 本地 MCP

```
amp → 子进程执行 command + args
     → stdin/stdout JSON-RPC over stdio
```

### 远程 MCP (SSE)

```
amp → HTTP/SSE 连接至远程 URL
     → OAuth 2.0 授权（如适用）
     → JSON-RPC over SSE
```

### MCP 注册表

```
amp 查询远程 MCP 注册表
    → 缓存 5 分钟
    → 展示给用户选择
    → 用户批准后安装
```

### 权限模型

```json
{
  "amp.mcpPermissions": [
    { "matches": { "command": "npx", "args": "*" }, "action": "allow" },
    { "matches": { "url": "https://*" }, "action": "reject" }
  ]
}
```

## 4. 远程执行协议 (Orbs)

模式推测（基于功能描述而非二进制证据）：

```
CLI/Web 端
   │
   ▼ 
控制平面（ampcode.com）
   │
   ├──→ SSH / 自定义协议 Orbs 端
   │        └── 执行 Agent 任务
   │
   └──→ Runner（本地 --no-tui）
            └── 接受远程创建线程
```

## 5. 环境变量清单

| 变量 | 用途 |
|------|------|
| `AMP_API_KEY` | API 密钥（非交互模式） |
| `AMP_HOME` | 安装目录（默认 `~/.amp`） |
| `AMP_SKIP_UPDATE_CHECK` | 跳过更新检查 |
| `AMP_FORCE_BEL` | 强制终端 bell（用于 SSH） |
| `HTTP_PROXY` / `HTTPS_PROXY` | 网络代理 |
| `NODE_EXTRA_CA_CERTS` | 自定义 CA 证书 |
