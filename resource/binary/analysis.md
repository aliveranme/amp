# amp CLI 二进制分析

> 逆向工程分析记录，基于 amp v0.0.1784219219-g3e9560 (darwin-arm64)，分析日期 2026-07-17。

## 二进制概要

| 属性 | 值 |
|------|-----|
| 文件 | `amp` |
| 大小 | 68 MB |
| 类型 | Mach-O 64-bit executable arm64 |
| 架构 | arm64（单架构，非 universal） |
| 运行时 | **Bun** (JavaScriptCore + Bun C++ runtime) |
| 构建工具 | Bun Bake (compile to binary) |

## 文件结构

### Link Libraries

| 库 | 用途 |
|----|------|
| `/usr/lib/libicucore.A.dylib` | Unicode/国际化支持 |
| `/usr/lib/libresolv.9.dylib` | DNS 解析 |
| `/usr/lib/libc++.1.dylib` | C++ 标准库 |
| `/usr/lib/libSystem.B.dylib` | 系统核心库 |
| CommonCrypto (CCCryptor, CCHmac) | 加密/哈希 |

### Segment 布局

| Segment | 大小 | 内容 |
|---------|------|------|
| `__PAGEZERO` | 4 GB | 空映射，JSC JIT 所需的大虚拟地址空间 |
| `__TEXT` | 58.4 MB | 代码段 |
| `├─ __text` | 52.7 MB | **主代码** — JSC 字节码 + JIT 编译代码 + Bun runtime C++ |
| `├─ __jsc_int` | 405 KB | JavaScriptCore 内部函数表（intrinsics） |
| `├─ __cstring` | 3.5 MB | C 字符串常量 |
| `├─ __const` | 4.4 MB | 常量数据 |
| `├─ __stubs` | 8 KB | 桩代码（动态链接） |
| `__DATA_CONST` | 1.2 MB | 数据常量段 |
| `├─ __got` | 全局偏移表 |
| `├─ __const` | 1.2 MB | 常量数据 |
| `__DATA` | ~6.5 MB | 可读写数据段 |

### 字符串统计

- **总字符串数**: 131,867
- 关键分类见 `strings/` 目录。

## 运行时架构

```
┌────────────────────────────────────────────┐
│            Bun Runtime (C++)               │
│  ┌──────────┐  ┌──────────────────────┐    │
│  │ JSCore   │  │ Bun JavaScript APIs   │   │
│  │ (JIT VM) │  │ (fetch, Bun.file, …)  │    │
│  └──────────┘  └──────────────────────┘    │
├────────────────────────────────────────────┤
│           Bytecode (__jsc_int)             │
│  ┌────────────────────────────────────┐    │
│  │ amp Application Code (JSC BC)      │   │
│  │  - Agent runtime                   │    │
│  │  - Plugin system                   │    │
│  │  - CLI interface                   │    │
│  │  - MCP integration                 │    │
│  │  - OAuth / auth flows              │    │
│  │  - TUI components                  │    │
│  └────────────────────────────────────┘    │
├────────────────────────────────────────────┤
│         Bun-VFS (Node Polyfills)           │
│  23 modules (assert, buffer, crypto, …)    │
├────────────────────────────────────────────┤
│         Embedded Resources                 │
│  - TLS root CAs                            │
│  - MIME types                             │
│  - Build configuration                     │
└────────────────────────────────────────────┘
```

## 编译模型

amp 使用 **Bun Bake**（Bun 的 `--compile` 功能）将应用打包为单二进制：

1. **源码阶段**: TypeScript/JavaScript 源码
2. **Bundling**: Bun 将应用打包为单文件 JS bundle
3. **Bytecode 编译**: Bun 将 JS bundle 编译为 JavaScriptCore bytecode
4. **嵌入**: bytecode 被嵌入到 `__TEXT.__text` 段中
5. **运行时**: Bun 启动时加载 bytecode 到 JSC VM 执行

## 关键技术特征

| 特征 | 证据 | 说明 |
|------|------|------|
| Bun 运行时 | `bake://server-runtime.js`, `Bun__*` C++ 符号 | 基于 WebKit JavaScriptCore |
| JSC Bytecode | `__jsc_int` 段 405KB | 预编译的 JS 字节码 |
| Node.js 兼容 | 23 个 bun-vfs polyfill 模块 | 提供 `assert`, `crypto`, `buffer` 等 |
| 插件系统 | `PluginAPI`, `registerTool`, `registerCommand` | TypeScript 插件框架 |
| Agent 模式 | low/medium/high/ultra | 四种执行模式 |
| MCP 集成 | MCP 注册表、`amp.mcpServers` | Model Context Protocol |
| OAuth/BYOK | `auth.openai.com` 端点 | 自带密钥流程 |
| 线程系统 | thread CRUD, session lifecycle | 持久化会话 |
| HMR 支持 | `bun:hmr` 运行时 | 开发模式下热替换 |

## 关键发现：BYOK 实现路径

二进制中包含完整的 OAuth 2.0 流程实现：

1. **OpenAI OAuth**: `https://auth.openai.com/oauth/authorize` + `/token`
2. **Access Token 管理**: 存储、刷新、吊销
3. **API Key 传递**: 通过 `AMP_API_KEY` 环境变量或设置存储
4. **模型准入**: 支持 GPT-5.6、Claude Fable 5 等多模型

这对实现 amp code 风格 BYOK CLI 工具提供了参考。
