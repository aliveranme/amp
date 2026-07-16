# 提取的 JS Bundle 分析

> 使用 [unbuned](https://github.com/vibheksoni/unbuned) 工具从 amp 二进制中提取。
> 提取日期：2026-07-17

## 基本信息

| 属性 | 值 |
|------|-----|
| 工具 | [unbuned v0.1](https://github.com/vibheksoni/unbuned) |
| 源文件 | `resource/amp` (68MB Mach-O arm64) |
| 提取结果 | 7,311,418 bytes / 6,123 行 |
| 位置 | `resource/binary/extracted/amp.js` |

## 提取结构

从 JS bundle 中识别出的模块依赖：

### 核心依赖

| 包名 | 版本 | 用途 |
|------|------|------|
| `@napi-rs/keyring` | 1.1.10 | 系统密钥链（macOS Keychain） |
| `@grpc/grpc-js` | 1.14.4 | gRPC 通信 |
| `pino` | 9.14.0 | 日志系统 |
| `win-ca` | 3.5.1 | Windows CA 证书 |

### Bun API 引用

```
Bun.spawn       — 子进程创建
Bun.spawnSync   — 同步子进程
Bun.file        — 文件读写
Bun.stdin       — 标准输入
Bun.plugin      — Bun 插件系统
Bun.Image       — 图片处理
bun:ffi         — FFI 外部函数接口
```

### 导出模块

```
findCredentials     — 凭据查找
findCredentialsAsync— 异步凭据查找
pino / levels       — 日志等级
stdSerializers      — 标准序列化
transport           — 日志传输
multistream         — 多流输出
Ajv / fastUri       — JSON Schema 验证
RequestHandler      — gRPC 请求处理
```

## 构建来源

- CI 构建路径: `/home/runner/work/amp/amp/`
- 包管理器: `pnpm` (`.pnpm` 目录结构)
- 打包方式: Bun Bake (JSC 字节码编译)

## 安全提示

提取的 JS 是经过 minify 的，变量名已被压缩为单字母。
建议使用 JS beautifier 处理后再分析逻辑结构。
