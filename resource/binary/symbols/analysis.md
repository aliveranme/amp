# 符号表分析

基于 `nm` 输出的 716 个符号。

## 符号概览

二进制已被剥离（stripped）大部分符号，剩余 716 个符号主要为：

| 类型 | 数量 | 说明 |
|------|------|------|
| 外部函数 (U) | ~700 | 动态链接的系统库符号 |
| 本地符号 | ~16 | C++ 符号、Objective-C 类 |

## 关键系统库引用

### CommonCrypto

```
CCCryptorCreateWithMode    CCCryptorReset    CCCryptorUpdate
CCHmacFinal                CCHmacInit
```

用途：amp 使用 CommonCrypto 进行数据加密和 HMAC 认证。可能用于：
- API 密钥的安全存储
- WebSocket 帧的加密
- 本地数据的加密缓存

### ICU (国际化)

```
ucal_setGregorianChange    ucfpos_constrainCategory
ucfpos_getCategory         unumsys_isAlgorithmic
```

用途：International Components for Unicode，提供日期/时间/数字/文本的国际化支持。

### libc++ (C++ 标准库)

大量 `std::*` 符号，用于 Bun runtime 的 C++ 实现。

## 导出函数模式

从剩余的 C++ 符号可以推断出 Bun runtime 的代码组织方式：

```
Bun::BakeLoadInitialServerCode  // Bake 服务器端初始化
Bun::BakeLoadServerHmrPatch     // HMR 热补丁加载
Bun::BakeLoadServerHmrPatchWithSourceMap // 带 SourceMap 的 HMR
BunInspectorConnection          // JavaScriptCore 调试器
Bun__initJSDebuggerThread      // 调试线程初始化
```

## 二进制剥离程度

- 应用代码（JavaScript/TypeScript）编译为 JSC bytecode → 在 nm 中不可见
- Bun runtime C++ 层大部分已剥离
- 仅保留必要的系统链接符号和少数调试符号
- 脱壳难度：**高**（JSC bytecode 反编译工具有限）
