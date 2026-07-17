# Orbs 与远程执行

> 来源：[https://ampcode.com/manual](https://ampcode.com/manual)

## Orbs

远程机器，可在断开本地连接后继续运行 amp 线程。更多细节见 [Orbs 手册](https://ampcode.com/manual/orbs)。

## Runners

从 ampcode.com 远程创建线程到任意运行 `amp` 的机器上。

### 启用 Runner

```json
// 设置或命令面板 → "amp: enable remote thread creation"
"amp.remoteThreadCreation.enabled": true
```

- 每个 TUI 实例在其启动目录接受新线程
- **纯 Runner 模式**：`amp --no-tui` 不打开 TUI 等待远程线程
- **稳定 Runner ID**：`amp --no-tui --runner-id grandmas-garage-server`
  - Runner ID 须为有效主机名，不区分大小写（保留原样）
