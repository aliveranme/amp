# amp code BYOK — 使用指南

> BYOK (Bring Your Own Key) LLM 代理 CLI 工具 — MVP

## 快速开始

### 前置条件

- Rust 工具链 (`cargo`, `rustc` 1.75+)
- Node.js 18+ (前端)
- npm 9+ (前端)

### 1. 启动后端

```bash
# 设置你的 API Key
export AMP_API_KEY=sk-your-openai-key-here

# 启动服务（默认 127.0.0.1:8080）
cargo run -- --server

# 或指定端口和数据库
cargo run -- --server --port 8090 --db /tmp/amp-code.db
```

### 2. 验证服务

```bash
# 健康检查
curl http://127.0.0.1:8080/health
# → {"status":"ok"}

# 创建线程
curl -X POST http://127.0.0.1:8080/api/threads \
  -H 'Content-Type: application/json' \
  -d '{"title":"我的线程"}'
# → {"id":"...","title":"我的线程","status":"Active",...}

# 查看线程列表
curl http://127.0.0.1:8080/api/threads
# → [...]
```

### 3. 启动前端

```bash
cd web
npm install
npm run dev
# → http://localhost:3000
```

打开 `http://localhost:3000` 即可看到 Dashboard。

### 4. 发送聊天请求

```bash
curl -X POST http://127.0.0.1:8080/v1/chat/completions \
  -H 'Content-Type: application/json' \
  -d '{
    "model": "gpt-4o",
    "messages": [{"role": "user", "content": "Hello!"}],
    "stream": true
  }'
# → SSE stream: data: {"id":"...","choices":[{"delta":{"content":"Hi!"}}]}
```

## 配置

### 环境变量

| 变量 | 必需 | 默认值 | 说明 |
|------|------|--------|------|
| `AMP_API_KEY` | 是 | — | LLM Provider 的 API 密钥 |
| `AMP_MODEL_DEFAULT` | 否 | `gpt-4o` | 默认模型 |
| `AMP_MODEL_ROUTE` | 否 | 内置路由表 | 自定义路由配置文件路径 |
| `AMP_URL` | 否 | — | 自定义后端地址（路由未匹配时转发） |

### 模型路由

默认路由表 (`route-config.toml`)：

| 模型名 | Provider | 端点 |
|--------|----------|------|
| `gpt-4o` | OpenAI | `api.openai.com/v1/chat/completions` |
| `gpt-4o-mini` | OpenAI | `api.openai.com/v1/chat/completions` |
| `claude-sonnet-4` | Anthropic | `api.anthropic.com/v1/messages` |
| `claude-fable-5` | Anthropic | `api.anthropic.com/v1/messages` |
| `*` (兜底) | OpenAI | `api.openai.com/v1/chat/completions` |

自定义路由文件：

```toml
# my-routes.toml
[route."gpt-4o"]
provider = "openai"
endpoint = "https://api.openai.com/v1/chat/completions"
auth_header = "Authorization"
auth_scheme = "Bearer"

[route."*"]
provider = "openai"
endpoint = "https://api.openai.com/v1/chat/completions"
auth_header = "Authorization"
auth_scheme = "Bearer"
```

使用：`export AMP_MODEL_ROUTE=./my-routes.toml`

## 项目结构

```
├── Cargo.toml              # Rust 工作区
├── route-config.toml       # 默认路由表
├── cmd/amp-code/           # CLI 入口
├── pkg/amp-core/           # 核心类型
├── pkg/amp-proxy/          # 代理引擎
├── pkg/amp-server/         # HTTP 服务
├── pkg/amp-storage/        # 持久化层
└── web/                    # 前端 (Next.js + shadcn/ui)
    ├── app/page.tsx        # Dashboard
    └── lib/                # API 客户端 + 类型
```

## API 参考

### POST /v1/chat/completions

OpenAI 兼容的聊天补全接口。

**请求体：**
```json
{
  "model": "gpt-4o",
  "messages": [{"role": "user", "content": "Hello"}],
  "stream": true,
  "temperature": 0.7
}
```

**响应：** SSE stream（OpenAI 格式）

### GET /api/threads

返回所有线程列表。

### POST /api/threads

创建新线程。

**请求体：**
```json
{
  "title": "线程标题"
}
```

### POST /api/sessions

创建新会话。

**请求体：**
```json
{
  "thread_id": "uuid-from-thread",
  "agent_mode": "medium"
}
```

`agent_mode` 可选值：`low`, `medium`, `high`, `ultra`

## 开发

```bash
# 编译
cargo build

# 热编译（开发）
cargo watch -x run -- --server

# 前端开发
cd web && npm run dev
```
