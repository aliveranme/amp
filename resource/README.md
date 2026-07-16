# resource — amp CLI 本体与逆向工程

> 本目录存放 amp CLI 二进制文件及其逆向工程分析结果。

## 目录结构

```
resource/
├── amp                          # amp CLI 二进制 (68MB, darwin-arm64)
├── install.sh                   # 官方安装脚本
├── amp-version.txt              # 版本号
├── amp.sha256                   # SHA256 校验和
├── README.md                    # 本文件
│
├── binary/                      # 二进制分析
│   ├── analysis.md              # 二进制整体分析
│   ├── load-commands.txt        # Mach-O load commands 原始输出
│   │
│   ├── sections/
│   │   └── analysis.md          # 段（Section）布局分析
│   │
│   ├── symbols/
│   │   └── analysis.md          # 符号表分析
│   │
│   └── strings/
│       ├── catalog.md           # 字符串分类目录
│       ├── graphql-operations.txt  # GraphQL 操作
│       ├── embedded-files.txt   # 嵌入文件引用
│       ├── errors.txt           # 错误消息
│       └── vfs-paths.txt        # Bun VFS 路径
│
└── structure/                   # 结构分析（预留）
    └── (待补充)
```

## 版本

| 项目 | 值 |
|------|-----|
| 版本 | `0.0.1784219219-g3e9560` |
| 架构 | arm64 |
| 运行时 | Bun (JavaScriptCore) |
| 下载来源 | `https://static.ampcode.com/cli/` |

## 分析状态

- [x] 文件类型识别
- [x] 段（Section）布局分析
- [x] 运行时识别 (Bun)
- [x] 嵌入 VFS 路径提取
- [x] API 端点提取
- [x] 符号表分析
- [x] 插件系统还原
- [x] OAuth/BYOK 流程分析
- [ ] JSC bytecode 反编译（待探索）
- [ ] 网络协议逆向（待深入）
- [ ] 插件 SDK 完整还原（待探索）
