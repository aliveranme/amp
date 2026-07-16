# BYOK 实现方案 — 功能与结构设计

> 基于 amp 源码逆向分析的设计映射
> 分析日期：2026-07-17

## 1. 核心架构

amp 的架构分为两层：**CLI 客户端** + **后端服务**。BYOK 实现需要完整复制这两层的核心能力。

```
┌──────────────────────────────────────────────────────────────┐
│                amp code CLI (BYOK Proxy)                      │
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────────┐               │
│  │ CLI 界面  │  │ Agent 引擎│  │ 插件 / MCP   │              │
│  │ (TUI/CMD) │  │ (路由)   │  │ 扩展系统     │              │
│  └────┬─────┘  └────┬─────┘  └──────┬───────┘               │
│       │             │               │                        │
│  ┌────┴─────────────┴───────────────┴───────┐               │
│  │          API 客户端 / 模型路由层          │               │
│  │  (gRPC ↔ REST 转换 · API Key 注入)       │               │
│  └────────────────┬─────────────────────────┘               │
│                   │                                          │
│                   ▼                                          │
│  ┌─────────────────────────────────────────┐               │
│  │          SSE 流式代理引擎                │               │
│  │  (tokio + reqwest · 零拷贝流式转发)      │               │
│  └────────────────┬─────────────────────────┘               │
└───────────────────┼──────────────────────────────────────────┘
                    │
                    ▼
┌──────────────────────────────────────────────────────────────┐
│                后端服务 (BYOK Server)                          │
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌─────────────┐ │
│  │ Thread   │  │ Session  │  │ Auth     │  │ Agent       │ │
│  │ Service  │  │ Service  │  │ Service  │  │ Engine      │ │
│  ├──────────┤  ├──────────┤  ├──────────┤  ├─────────────┤ │
│  │ Actor    │  │ Config   │  │ File     │  │ Telemetry   │ │
│  │ Service  │  │ Service  │  │ Service  │  │ Service     │ │
│  └──────────┘  └──────────┘  └──────────┘  └─────────────┘ │
│                                                              │
│  ┌─────────────────────────────────────────────────────┐    │
│  │             数据库 / 持久化层                         │    │
│  │  (线程存储 · 会话状态 · 配置 · 凭据 · 历史记录)        │    │
│  └─────────────────────────────────────────────────────┘    │
└──────────────────────────────────────────────────────────────┘
```

---

## 2. 功能映射：amp 功能 → BYOK 实现

### 2.1 必须实现的核心功能

| amp 功能 | BYOK 要求 | 优先级 | 说明 |
|----------|-----------|--------|------|
| Thread 管理 | **必需** | P0 | 线程 CRUD、归档、搜索 |
| Session 管理 | **必需** | P0 | 会话生命周期、状态持久化 |
| Agent 执行引擎 | **必需** | P0 | LLM 调用、Tool 执行、子 Agent |
| Actor 状态机 | **必需** | P0 | ACTIVE/ABORTED/STOPPING 状态流转 |
| SSE 流式响应 | **必需** | P0 | 模型输出的流式透传 |
| 模型路由 | **必需** | P0 | model → provider 映射 + API Key 注入 |
| CLI 命令系统 | **必需** | P0 | 基础 CLI 交互 |
| 配置管理 | **必需** | P1 | settings.json 读写、环境变量 |

### 2.2 建议实现的功能

| amp 功能 | BYOK 要求 | 优先级 | 说明 |
|----------|-----------|--------|------|
| 多 Provider 凭据 | **建议** | P1 | 支持 OpenAI/Anthropic/Google 等 |
| Plugin 系统 | **建议** | P1 | Tool/Command 注册、MCP |
| 子 Agent 编排 | **建议** | P1 | 并行子任务执行 |
| OAuth PKCE 流程 | **建议** | P1 | 浏览器授权回调 |
| gRPC 通信 | **建议** | P1 | 高性能后端通信 |
| 密钥链集成 | **建议** | P1 | 系统密钥链安全存储 |
| 文件/附件管理 | **建议** | P2 | 上传/下载/引用 |
| 遥测 | **建议** | P2 | 使用统计、性能监控 |

### 2.3 可后续实现的功能

| amp 功能 | BYOK 要求 | 优先级 | 说明 |
|----------|-----------|--------|------|
| Orbs 远程执行 | 可延后 | P2 | 云端持续运行 Agent |
| Runners | 可延后 | P2 | 远程线程创建 |
| TUI 完整界面 | 可延后 | P2 | 命令面板、线程侧边栏 |
| Oracle/Librarian/Painter | 可延后 | P2 | 特殊 Agent 工具 |
| 团队协作 | 可延后 | P3 | 工作区、可见性、共享 |
| Enterprise SSO | 可延后 | P3 | SAML/OIDC |

---

## 3. 后端服务接口设计

### 3.1 Thread 服务

```
service ThreadService {
  rpc CreateThread(CreateThreadRequest) returns (Thread);
  rpc GetThread(GetThreadRequest) returns (Thread);
  rpc ListThreads(ListThreadsRequest) returns (ListThreadsResponse);
  rpc UpdateThread(UpdateThreadRequest) returns (Thread);
  rpc ArchiveThread(ArchiveThreadRequest) returns (Empty);
  rpc SearchThreads(SearchThreadsRequest) returns (SearchThreadsResponse);
  rpc WatchThread(WatchThreadRequest) returns (stream ThreadEvent);
}
```

### 3.2 Session 服务

```
service SessionService {
  rpc CreateSession(CreateSessionRequest) returns (Session);
  rpc GetSession(GetSessionRequest) returns (Session);
  rpc EndSession(EndSessionRequest) returns (Empty);
  rpc Heartbeat(HeartbeatRequest) returns (HeartbeatResponse);
}
```

### 3.3 Agent 执行服务

```
service AgentService {
  rpc Execute(stream ExecuteRequest) returns (stream ExecuteResponse);
  rpc CancelExecution(CancelRequest) returns (Empty);
  rpc GetExecutionStatus(StatusRequest) returns (ExecutionStatus);
}
```

### 3.4 Actor 管理服务

```
service ActorService {
  rpc CreateActor(CreateActorRequest) returns (Actor);
  rpc StopActor(StopActorRequest) returns (Empty);
  rpc GetActorStatus(GetActorRequest) returns (ActorStatus);
  rpc ListActors(ListActorsRequest) returns (ListActorsResponse);
}
```

---

## 4. 模型路由引擎设计

### 4.1 请求流程

```
客户端请求
  POST /v1/chat/completions
  Authorization: Bearer <user-token>
  {
    model: "gpt-4o",
    messages: [...],
    stream: true
  }
       │
       ▼
① 路由解析层
  ├── 提取 model 字段 → "gpt-4o"
  ├── 查找路由表 → provider: "openai", endpoint: "..."
  └── 提取 API Key → env / keychain / config

       ▼
② 请求转换层
  ├── 注入 Authorization header
  ├── 转换请求格式 (OpenAI ↔ Provider)
  └── 转发到目标 endpoint

       ▼
③ 流式代理层
  ├── 接收 SSE stream
  ├── 透传到客户端 (零拷贝)
  └── 错误处理 / 重试逻辑

       ▼
客户端收到 SSE stream
```

### 4.2 路由表格式

```toml
[route."gpt-4o"]
provider = "openai"
endpoint = "https://api.openai.com/v1/chat/completions"
auth_header = "Authorization"
auth_scheme = "Bearer"

[route."claude-sonnet-4"]
provider = "anthropic"
endpoint = "https://api.anthropic.com/v1/messages"
auth_header = "x-api-key"
extra_headers = { anthropic-version = "2023-06-01" }

[route."*"]
# 通配兜底
provider = "openai"
endpoint = "https://api.openai.com/v1/chat/completions"
```

### 4.3 API Key 解析优先级

```
1. 请求 Header: Authorization: Bearer <key>         (透传模式)
2. 环境变量:   AMP_API_KEY                           (BYOK 主模式)
3. 配置文件:   ~/.config/amp/settings.json            (持久化配置)
4. 系统密钥链: macOS Keychain / Windows Credential    (安全存储)
```

---

## 5. 数据模型

### 5.1 Thread

```protobuf
message Thread {
  string id = 1;
  string title = 2;
  ThreadStatus status = 3;   // ACTIVE / ARCHIVED
  repeated Message messages = 4;
  map<string, string> metadata = 5;
  string project_id = 6;
  string user_id = 7;
  Timestamp created_at = 8;
  Timestamp updated_at = 9;
  repeated string labels = 10;
  Visibility visibility = 11;
}

message Message {
  string id = 1;
  string role = 2;           // user / assistant / tool / system
  repeated Content content = 3;
  Timestamp timestamp = 4;
  map<string, string> metadata = 5;
}
```

### 5.2 Session

```protobuf
message Session {
  string id = 1;
  string thread_id = 2;
  string agent_mode = 3;     // low / medium / high / ultra
  SessionStatus status = 4;  // ACTIVE / PAUSED / ENDED
  map<string, string> context = 5;
  Timestamp started_at = 6;
  Timestamp last_heartbeat = 7;
  repeated string active_tools = 8;
}
```

### 5.3 Actor

```protobuf
message Actor {
  string id = 1;
  string session_id = 2;
  string type = 3;           // main / subagent / oracle
  ActorState state = 4;      // ACTIVE / ABORTED / STOPPING
  repeated ToolCall active_calls = 5;
  Timestamp created_at = 6;
  ExecutionConfig config = 7;
}
```

---

## 6. 存储层设计

### 6.1 持久化存储

| 数据 | 存储后端 | 说明 |
|------|---------|------|
| Thread 数据 | PostgreSQL / SQLite | 线程、消息、标签 |
| Session 状态 | Redis / 内存 | 活跃会话、心跳 |
| Actor 状态 | 内存 | 运行时状态、不需要持久化 |
| 配置 | 配置文件 + DB | settings.json + 用户设置 |
| 凭据 | 系统密钥链 / 加密文件 | API Key 安全存储 |
| 历史记录 | PostgreSQL / SQLite | 线程历史、使用统计 |

### 6.2 推荐部署拓扑

```
P0 (MVP):        CLI + 本地 SQLite
P1 (单用户):     CLI + 本地 SQLite + 密钥链
P2 (多用户):     CLI + gRPC 后端 + PostgreSQL + Redis
P3 (生产):       CLI + gRPC 后端集群 + PostgreSQL + Redis + 负载均衡
```

---

## 7. 安全设计

### 7.1 API Key 安全

| 存储方式 | 安全等级 | 适用场景 |
|---------|---------|---------|
| 环境变量 `AMP_API_KEY` | ⭐⭐ | 开发、CI/CD |
| 配置文件 (600 权限) | ⭐⭐⭐ | 桌面用户 |
| 系统密钥链 | ⭐⭐⭐⭐ | 生产环境（推荐） |
| 加密存储 + 主密码 | ⭐⭐⭐⭐⭐ | 企业环境 |

### 7.2 通信安全

- CLI ↔ 后端: mTLS / HTTPS
- 模型 API: HTTPS (强制)
- 本地存储: 加密 (AES-256-GCM)

---

## 8. Rust 项目模块结构

```
amp-code/
├── Cargo.toml
├── src/
│   ├── main.rs                  # CLI 入口
│   ├── cli/                     # CLI 命令系统
│   │   ├── mod.rs
│   │   ├── args.rs              # clap 参数定义
│   │   ├── commands.rs          # 命令路由
│   │   └── repl.rs              # 交互式循环
│   │
│   ├── proxy/                   # 模型路由引擎 (核心)
│   │   ├── mod.rs
│   │   ├── router.rs            # 路由表匹配
│   │   ├── injector.rs          # API Key 注入
│   │   ├── transformer.rs       # 请求/响应格式转换
│   │   └── streamer.rs          # SSE 流式转发
│   │
│   ├── server/                  # 后端服务
│   │   ├── mod.rs
│   │   ├── thread.rs            # Thread 服务
│   │   ├── session.rs           # Session 服务
│   │   ├── agent.rs             # Agent 执行引擎
│   │   ├── actor.rs             # Actor 管理
│   │   └── auth.rs              # 认证服务
│   │
│   ├── storage/                 # 持久化层
│   │   ├── mod.rs
│   │   ├── sqlite.rs            # SQLite 存储
│   │   ├── keychain.rs          # 系统密钥链
│   │   └── config.rs            # 配置文件管理
│   │
│   ├── plugin/                  # 插件系统
│   │   ├── mod.rs
│   │   ├── registry.rs          # 工具/命令注册
│   │   ├── mcp.rs               # MCP 框架
│   │   └── event.rs             # 事件系统
│   │
│   └── types/                   # 共享类型
│       ├── mod.rs
│       ├── thread.rs
│       ├── agent.rs
│       └── config.rs
│
├── tests/                       # 集成测试
├── examples/                    # 使用示例
└── route-config.toml            # 默认路由表
```
