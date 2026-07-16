# amp 源码模块映射

> 基于 `resource/binary/extracted/amp.js` (7MB/6123 行) 逆向分析
> 构建系统：Bun Bake (JSC bytecode 编译 + minify)
> 分析日期：2026-07-17

## 整体架构层次

```
┌──────────────────────────────────────────────────────────────┐
│                       CLI / TUI 层                           │
│  (命令注册、终端界面、键位绑定、交互循环)                       │
├──────────────────────────────────────────────────────────────┤
│                       Agent 引擎层                           │
│  (线程管理、Session 生命周期、Actor 状态机、                  │
│   子 Agent 编排、Oracle/Librarian/Painter)                   │
├──────────────────────────────────────────────────────────────┤
│                      插件 / 扩展层                            │
│  (Plugin API、Tool 注册、Command 注册、                       │
│   MCP 框架、Agent Skills、事件系统)                          │
├──────────────────────────────────────────────────────────────┤
│                      API 客户端层                             │
│  (gRPC 服务调用、REST API、GraphQL、                         │
│   WebSocket、SSE 流式代理)                                   │
├──────────────────────────────────────────────────────────────┤
│                      认证 / 凭据层                            │
│  (OAuth 2.0 PKCE、41+ Provider 凭据类型、                    │
│   系统密钥链集成、Token 管理)                                │
├──────────────────────────────────────────────────────────────┤
│                      基础设施层                               │
│  (AJV Schema 验证、Pino 日志、流式传输、                      │
│   进程管理、文件系统、错误处理)                               │
└──────────────────────────────────────────────────────────────┘
```

---

## 1. CLI / TUI 层

### 命令系统

| 命令 ID | 类型 | 用途 |
|---------|------|------|
| `amp.help` | CLI | 帮助信息 |
| `amp.quit` | CLI | 退出程序 |
| `amp.disconnect` | CLI | 断开连接 |
| `amp.reconnect` | CLI | 重新连接 |
| `amp.relaunch` | CLI | 重启 |
| `amp.cli` | CLI | CLI 子命令 |
| `amp.bat` | CLI | Windows 批处理 |
| `amp.exe` | CLI | Windows 可执行 |
| `amp.download.tmp` | CLI | 临时下载 |
| `amp.download.tmp.exe` | CLI | Windows 临时下载 |
| `amp.showVersion` | CLI | 版本显示 |
| `amp.showWelcome` | CLI | 欢迎页面 |
| `amp.showThreadVisibility` | CLI | 线程可见性 |
| `amp.enableRemoteThreadCreation` | CLI | 远程线程创建 |
| `amp.disableRemoteThreadCreation` | CLI | 禁用远程线程创建 |
| `amp.endCredits` | CLI | 结束贡献者名单 |
| `amp.mcpServers` | CLI | MCP 服务器管理 |
| `amp.network.timeout` | CLI | 网络超时设置 |

### TUI 组件

- **Command Palette** (`Ctrl+O`) — 全局命令入口
- **Thread Sidebar** (`Ctrl+\`) — 线程列表
- **Agent Mode Dial** (`Ctrl+S`) — 模式切换
- **Prompt Editor** (`Ctrl+G`) — 外部编辑器集成
- **Keybinding** 系统 — 自定义键位映射 (`amp.keymap`)

### 注册统计

| 类型 | 数量 |
|------|------|
| `registerTool()` | 25 |
| `registerCommand()` | 11 |

---

## 2. Agent 引擎层

### 会话 / 线程管理

使用 **Actor 模型** 管理并发会话：

```
Actor (线程执行单元)
  ├── 状态: ACTIVE / ABORTED / ACTOR_STOPPING / ACCESS_DENIED
  ├── 生命周期: create → run → pause → resume → archive
  └── 通信: gRPC stream (双向流式 RPC)
```

**gRPC 服务** (从字符串推断):

| 服务 | 方法 | 用途 |
|------|------|------|
| `threadservice` | CRUD | 线程创建/读取/更新/归档 |
| `threadsummary` | 聚合 | 线程摘要生成 |
| `threadsync` | 同步 | 线程状态同步 |
| `threadworker` | 执行 | 线程任务执行 |
| `threadhistory` | 历史 | 线程历史记录 |
| `actor` | 管理 | Actor 生命周期管理 |
| `agent` | 执行 | Agent 执行引擎 |
| `session` | 会话 | 会话管理 |

### 子 Agent 系统

- **子 Agent 隔离**：独立上下文窗口 + 独立工具权限
- **通信限制**：子 Agent 之间不能互相通信
- **结果聚合**：主 Agent 只获取摘要
- **并行执行**：子 Agent 相互独立，可并行

### 内置 Agent

| Agent | 模型 | 用途 |
|-------|------|------|
| Main (low/medium/high/ultra) | GPT-5.6 / Claude Fable 5 | 主执行引擎 |
| Oracle | GPT-5.6 Sol / Claude Fable 5 | 深度推理第二意见 |
| Librarian | GitHub API | 远程代码搜索 |
| Painter | GPT Image 2 | 图片生成/编辑 |

### 状态管理

从代码中提取的状态枚举：

| 状态 | 说明 |
|------|------|
| `ACTIVE` | 线程/Agent 活跃 |
| `ABORTED` | 已终止 |
| `ACTOR_STOPPING` | Actor 正在停止 |
| `ACCESS_DENIED` | 拒绝访问 |
| `ALREADY_EXISTS` | 资源已存在 |
| `AGGREGATION_TEMPORALITY_DELTA` | 遥测聚合模式 |

---

## 3. 插件 / 扩展层

### Plugin API 接口

```
PluginAPI
  ├── registerTool()        ← 25 次调用
  ├── registerCommand()     ← 11 次调用
  ├── .on()                 ← 事件监听
  │   ├── "tool.call"       — 工具调用拦截
  │   ├── "tool.result"     — 工具结果处理
  │   ├── "session.*"       — 会话事件
  │   └── "agent.*"         — Agent 事件
  ├── amp.$`cmd`            — Shell 执行
  ├── amp.ai.ask()          — AI 分类
  └── ctx.ui.*              — UI 交互
```

### MCP 框架

```
MCP Config
  ├── amp.mcpServers          — MCP 服务器定义
  ├── amp.mcpTrustedWorkspaces — 受信任工作区
  ├── amp.mcpPermissions      — 权限规则
  ├── amp.mcpConfig           — 高级配置
  ├── amp.mcpDoctor           — 诊断命令
  └── amp.mcpRegistry         — 远程注册表
```

### 事件生命周期

```
session.start → agent.start → [tool.call → tool.result]^N → agent.end
```

---

## 4. API 客户端层

### 通信协议

| 协议 | 用途 | 证据 |
|------|------|------|
| **gRPC** | 核心后端通信 | `grpc.` (97x), `loadPackageDefinition` (8x), `makeGenericClientConstructor` |
| **REST** | 文件/附件操作 | `GET`(51), `POST`(34), `PUT`(13), `DELETE`(6), `PATCH`(4) |
| **GraphQL** | 查询操作 | 二进制中提取的 GQL 操作字符串 |
| **WebSocket** | 实时通信 | `upgrade`, `open`, `message`, `close` 事件 |
| **SSE** | 流式响应 | `requestStream`, `responseStream`, `serverStream` |

### gRPC 流类型

| 流类型 | 说明 |
|--------|------|
| `clientStream` | 客户端流式请求 |
| `serverStream` | 服务端流式响应 (SSE) |
| `requestStream` | 双向请求流 |
| `responseStream` | 双向响应流 |

### REST API 端点

| 路径 | 方法 | 用途 |
|------|------|------|
| `/api/attachments` | 多种 | 附件管理 |
| `/api/telemetry` | POST | 遥测数据 |
| `/api/thread-actors` | 多种 | 线程 Actor 管理 |
| `/api/user-actor-credentials` | 多种 | 用户凭据管理 |

### 流式代理模型

```
客户端 HTTP/2 SSE ←→ amp CLI ←→ gRPC stream ←→ amp 后端
```

---

## 5. 认证 / 凭据层

### 支持的 Provider 凭据 (41+ 种)

**AI/LLM Providers:**
- `Anthropic API Key`
- `OpenAI API Key`
- `Google API Key`
- `Hugging Face Access Token`

**Cloud Providers:**
- `AWS Access Key ID` / `AWS Secret Access Key`
- `Azure` (multiple)
- `Cloudflare API Token`
- `Alibaba AccessKey ID` / `Alibaba Secret Key`
- `Heroku API Key`

**Code/Git Providers:**
- `GitHub Personal Access Token`
- `GitHub OAuth Access Token`
- `GitHub App Token`
- `GitHub Refresh Token`
- `GitLab Personal Access Token`
- `Bitbucket Personal Access Token`
- `Bitbucket Repository Access Token`
- `Sourcegraph Amp Access Token` (等多种 Sourcegraph token)

**Other:**
- `Stripe Secret Key`
- `Twilio API Key`
- `Asana Client ID/Secret`
- `Adobe Client Secret`
- `Canva Token`
- `Lob API Key`
- `New Relic user API Key`

### OAuth 2.0 流程

```
PKCE (Proof Key for Code Exchange)
  ├── codeVerifier     — 随机字符串（本地生成）
  ├── codeChallenge    — codeVerifier 哈希值
  ├── authorizationCode — 授权码（回调后获取）
  ├── refreshToken     — 刷新令牌
  └── openid           — OpenID Connect 支持
```

### 密钥存储

- **系统密钥链**: `@napi-rs/keyring@1.1.10` (macOS Keychain / Windows Credential Manager / Linux Secret Service)
- **环境变量**: `AMP_API_KEY` 等
- **配置文件**: `~/.config/amp/settings.json`

---

## 6. 基础设施层

### 日志系统

| 组件 | 用途 |
|------|------|
| `pino@9.14.0` | 结构化日志 |
| `levels` | 日志级别管理 |
| `stdSerializers` | 标准序列化 |
| `transport` | 日志传输 |
| `multistream` | 多目标输出 |
| `destination` | 日志目标 |

### 数据验证

- **AJV** (Another JSON Validator) — JSON Schema 验证
- `ajv-formats` — 格式验证扩展
- `fastUri` — 快速 URI 验证

### 进程管理

```
Bun.spawn     — 异步子进程创建 (2x)
Bun.spawnSync — 同步子进程创建 (2x)
Bun.stdin     — 标准输入
Bun.file      — 文件操作
Bun.plugin    — Bun 构建插件
Bun.Image     — 图片处理
```

### 错误处理

- 721 `async function` — 大量异步操作
- 203 `new Promise`, 186 `Promise.resolve` — 显式 Promise 模式
- 26 `Promise.reject` — 错误路径处理
- 运行时验证：`assert()`, `throw` 模式

---

## 7. 环境变量清单（完整）

从源码中提取的全部环境变量：

| 变量 | 用途 |
|------|------|
| `AMP_API_KEY` | API 密钥（BYOK 主变量） |
| `AMP_HOME` | 安装目录 |
| `AMP_INITIAL_AGENT_MODE_KEY` | 初始 Agent 模式 |
| `AMP_FUZZY_INDEX_MAX_FILES` | 模糊搜索最大文件数 |
| `AMP_EXPERIMENT_PERFSORT` | 性能排序实验 |
| `AMP_LOG_FILE` | 日志文件路径 |
| `AMP_LOG_LEVEL` | 日志级别 |
| `AMP_SKIP_UPDATE_CHECK` | 跳过更新检查 |
| `AMP_FORCE_BEL` | 强制终端 bell |
| `__MISE_*` | Mise 开发环境集成 |
| `HTTP_PROXY` / `HTTPS_PROXY` | 网络代理 |
| `NODE_EXTRA_CA_CERTS` | 自定义 CA 证书 |
