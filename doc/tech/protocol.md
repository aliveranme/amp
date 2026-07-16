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

## 5. 环境变量清单（完整）

从二进制 JS bundle 中提取的全部环境变量：

### amp 核心变量

| 变量 | 用途 |
|------|------|
| `AMP_API_KEY` | API 密钥（BYOK 主变量，非交互模式使用） |
| `AMP_URL` | 自定义后端地址 |
| `AMP_HOME` | 安装目录（默认 `~/.amp`） |
| `AMP_SKIP_UPDATE_CHECK` | 跳过更新检查 |
| `AMP_FORCE_BEL` | 强制终端 bell（用于 SSH） |
| `AMP_LOG_FILE` | 日志文件路径 |
| `AMP_LOG_LEVEL` | 日志级别 |
| `AMP_MAX_LOG_FILE_SIZE` | 最大日志文件大小 |
| `AMP_SETTINGS_FILE` | 自定义设置文件路径 |
| `AMP_PWD` | 工作目录 |

### Agent / 执行

| 变量 | 用途 |
|------|------|
| `AMP_INITIAL_AGENT_MODE_KEY` | 初始 Agent 模式 |
| `AMP_THREAD_ID` | 线程 ID（用于 CI 模式） |
| `AMP_EXECUTOR` | 执行器类型 |
| `AMP_DIRECT_TERMINAL` | 直接终端模式 |
| `AMP_BUILD_TRANSCRIPT_CACHE` | 构建日志缓存 |
| `AMP_BUILD_TRANSCRIPT_PERF_LOG` | 构建性能日志 |

### 调试 / 开发

| 变量 | 用途 |
|------|------|
| `AMP_DEBUG` | 调试模式 |
| `AMP_DISABLE_PLUGINS` | 禁用插件 |
| `AMP_DISABLE_AMP_COAUTHOR_TRAILER` | 禁用 Co-author 尾注 |
| `AMP_DISABLE_SECRET_REDACTION` | 禁用密钥脱敏 |
| `AMP_ENABLE_TRACING` | 启用链路追踪 |
| `AMP_HEADLESS_OAUTH` | 无头 OAuth 模式 |
| `AMP_HEADLESS_VERBOSE` | 无头模式详细输出 |
| `AMP_PLUGIN_RUNTIME_LOG_FILE` | 插件运行时日志 |
| `AMP_PLUGIN_SOURCE_BASE64` | Base64 编码的插件源码 |
| `AMP_PLUGIN_URI` | 插件 URI |
| `AMP_SDK_VERSION` | SDK 版本约束 |

### Portal / Orb

| 变量 | 用途 |
|------|------|
| `AMP_PORTAL_DOMAIN` | Portal 域名 |
| `AMP_PORTAL_PROXY_PORT` | Portal 代理端口 |
| `AMP_PORTAL_PROXY_TARGET_ORIGIN` | Portal 代理目标 |
| `AMP_PORTAL_HAIRPIN_CERT` | Portal Hairpin 证书 |
| `AMP_PORTAL_HAIRPIN_KEY` | Portal Hairpin 密钥 |
| `AMP_PORTAL_HAIRPIN_DOMAIN` | Portal Hairpin 域名 |

### UI / 终端

| 变量 | 用途 |
|------|------|
| `AMP_EDITOR` | 编辑器选择 |
| `AMP_USER_EMAIL` | 用户邮箱 |
| `AMP_USE_NATIVE_WEBSOCKET` | 使用原生 WebSocket |
| `NO_ANIMATION` / `NO_ANIMATIONS` | 禁用动画 |
| `NO_COLOR` | 禁用颜色 |

### 性能 / 实验

| 变量 | 用途 |
|------|------|
| `AMP_EXPERIMENT_PERFSORT` | 性能排序实验 |
| `AMP_FUZZY_INDEX_MAX_FILES` | 模糊搜索最大文件数 |
| `AMP_RELAUNCH_MINIMUM_OPEN_DURATION_MS` | 最小重开时长 |
| `AMP_RIPGREP_PATH` | rg 二进制路径 |

### OpenTelemetry

| 变量 | 用途 |
|------|------|
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OTLP 导出端点 |
| `OTEL_EXPORTER_OTLP_HEADERS` | OTLP 导出头部 |
| `OTEL_EXPORTER_OTLP_INSECURE` | OTLP 非安全模式 |
| `OTEL_EXPORTER_PROMETHEUS_HOST` | Prometheus 主机 |
| `OTEL_EXPORTER_PROMETHEUS_PORT` | Prometheus 端口 |

### gRPC

| 变量 | 用途 |
|------|------|
| `GRPC_DEFAULT_SSL_ROOTS_FILE_PATH` | gRPC SSL 根证书 |
| `GRPC_EXPERIMENTAL_ENABLE_OUTLIER_DETECTION` | 异常检测 |
| `GRPC_NODE_TRACE` | gRPC 跟踪 |
| `GRPC_NODE_VERBOSITY` | gRPC 详细度 |
| `GRPC_NODE_USE_ALTERNATIVE_RESOLVER` | 替代解析器 |
| `GRPC_SSL_CIPHER_SUITES` | SSL 加密套件 |
| `GRPC_TRACE` | gRPC 跟踪 |
| `GRPC_VERBOSITY` | gRPC 详细度 |
| `grpc_proxy` / `no_grpc_proxy` | gRPC 代理 |

### 网络代理

| 变量 | 用途 |
|------|------|
| `HTTP_PROXY` / `http_proxy` | HTTP 代理 |
| `HTTPS_PROXY` / `https_proxy` | HTTPS 代理 |
| `NO_PROXY` / `no_proxy` | 无代理白名单 |
| `SSL_CERT_DIR` | SSL 证书目录 |
| `NODE_EXTRA_CA_CERTS` | 自定义 CA 证书 |

### 环境检测

| 变量 | 用途 |
|------|------|
| `SHELL` | Shell 路径 |
| `TERM` | 终端类型 |
| `TERM_PROGRAM` | 终端程序 |
| `TERM_PROGRAM_VERSION` | 终端程序版本 |
| `TMUX` / `TMUX_PANE` | tmux 会话 |
| `SSH_CLIENT` / `SSH_CONNECTION` / `SSH_TTY` | SSH 会话 |
| `WSL_DISTRO_NAME` | WSL 发行版 |
| `DISPLAY` / `WAYLAND_DISPLAY` | 显示服务器 |
| `ZED_CHANNEL` / `ZED_TERM` | Zed 编辑器 |
| `NVIM` | Neovim |
| `INSIDE_EMACS` | Emacs |
| `CI` | CI 环境 |
| `VITEST` / `JEST_WORKER_ID` / `NODE_TEST_CONTEXT` | 测试环境 |
| `__CFBundleIdentifier` | macOS 应用标识 |

### Rivet 集成

| 变量 | 用途 |
|------|------|
| `RIVET_POOL` | Rivet 资源池 |
| `RIVET_PUBLIC_ENDPOINT` | Rivet 公共端点 |

### 构建 / 包管理

| 变量 | 用途 |
|------|------|
| `BUN_INSTALL` | Bun 安装路径 |
| `BUN_BE_BUN` | Bun 内部 |
| `BUN_TEST` | Bun 测试 |
| `PNPM_HOME` | pnpm 主目录 |
| `npm_config_registry` | npm 注册表 |
| `npm_config_user_agent` | npm 用户代理 |
| `YARN_WRAP_OUTPUT` | Yarn 输出包装 |
| `npm_config_arch` | npm 架构 |
| `TURBOPACK` | Turbopack 标志 |
| `PREBUILDS_ONLY` | 仅预构建 |

### 跨平台路径

| 变量 | 用途 |
|------|------|
| `HOME` | 用户主目录 (Unix) |
| `USERPROFILE` | 用户主目录 (Windows) |
| `APPDATA` | 应用数据 (Windows) |
| `LOCALAPPDATA` | 本地应用数据 (Windows) |
| `ProgramData` | 程序数据 (Windows) |
| `PATHEXT` | 可执行扩展名 (Windows) |
| `comspec` | CMD 路径 (Windows) |
| `XDG_CONFIG_HOME` | XDG 配置目录 |
| `XDG_CACHE_HOME` | XDG 缓存目录 |
| `XDG_DATA_HOME` | XDG 数据目录 |
| `XDG_DOWNLOAD_DIR` | XDG 下载目录 |
