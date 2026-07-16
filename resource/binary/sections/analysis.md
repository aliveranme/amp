# 二进制段（Sections）分析

基于 `otool -l` 输出分析 Mach-O 段布局。

## 内存映射

| Segment | Section | 虚拟地址 | 大小 | 偏移 | 说明 |
|---------|---------|----------|------|------|------|
| `__PAGEZERO` | — | `0x0` | 4 GB | 0 | JSC JIT 空映射 |
| `__TEXT` | — | `0x100000000` | 58.4 MB | 0 | 代码段起始 |
| | `__text` | | 52.7 MB | 18560 | 主代码（JSC BC + Bun C++） |
| | `__jsc_int` | | 405 KB | 52838912 | JavaScriptCore 内部函数名 |
| | `__stubs` | | 8 KB | 53253072 | 动态链接桩 |
| | `__init_offsets` | | 4 B | 53261436 | 初始化偏移 |
| | `__cstring` | | 3.5 MB | 53261440 | C 字符串常量 |
| | `__const` | | 4.4 MB | 56819712 | 常量数据 |
| | `__oslogstring` | | 1.6 KB | 61272548 | OS 日志字符串 |
| | `__ustring` | | 122 B | 61274178 | Unicode 字符串 |
| `__DATA_CONST` | — | `0x103dec000` | 1.2 MB | — | 常量数据段 |
| | `__got` | | 5.7 KB | 61276160 | 全局偏移表 |
| | `__const` | | 1.2 MB | 61281888 | 只读数据 |
| `__DATA` | — | `0x103f1c000` | ~6.5 MB | — | 读写数据段 |

## 关键发现

### `__jsc_int` 段（JavaScriptCore Intrinsics）

JavaScriptCore 使用 intrinsics 表来快速访问内置函数。此段包含函数名映射，用于 JIT 编译时的内联优化。存在此段确认了 amp 使用 WebKit JSC 作为 JS 引擎。

### `__text` 段 (52.7 MB)

包含三种类型的代码混合：
1. Bun C++ runtime 编译代码（JavaScriptCore 绑定、Bun API 实现）
2. WebKit JavaScriptCore JIT 编译代码（JIT 生成的机器码）
3. JavaScript 应用的 bytecode 编译结果

### PAGEZERO (4 GB)

arm64 上 4 GB 的空地址映射，为 JSC JIT 编译器预留虚拟地址空间。Bun 应用的标准特性。
