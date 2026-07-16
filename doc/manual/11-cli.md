# CLI 参考

> 来源：[https://ampcode.com/manual](https://ampcode.com/manual)

## 交互模式

```bash
amp
```

管道输入的内容将成为第一条消息：

```bash
echo "commit all my changes" | amp
```

## 执行模式（`-x` / `--execute`）

发送消息，等待 agent 完成，打印最终消息，退出：

```bash
amp -x "what files in this folder are markdown files?"
```

管道输入也可与 `-x` 配合：

```bash
echo "what package manager is used here?" | amp -x
```

同时提供管道输入和 `-x` 时，agent 会看到两者：

```bash
cat ~/.vimrc | amp -x "which colorscheme is used?"
```

## `--mcp-config` 与 `-x`

```bash
amp --mcp-config '{"everything": {"command": "npx", "args": ["-y", "@modelcontextprotocol/server-everything"]}}' -x "What tools are available?"
```

## `--plugin-ready-timeout`

使执行模式等待插件加载后再运行（确保 `agent.start`/`agent.end` 事件触发）：

- 裸标志：最多等待 10 秒
- `--plugin-ready-timeout 30`：自定义秒数（最大 300，`0` 禁用）

## 输出重定向

执行模式在 stdout 被重定向时自动激活：

```bash
echo "what is 2+2?" | amp > response.txt
```

## `--help`

完整 CLI 详情：`amp --help`

## `--stream-json`

与 `--execute` 模式配合，每行输出一个 JSON 对象到 stdout。

```bash
amp --execute "what is 3 + 5?" --stream-json
```

### 带线程延续

```bash
amp threads continue --execute "now add 8" --stream-json
```

### 多轮 stdin 交互

```bash
send_message() {
  local text="$1"
  echo '{"type":"user","message":{"role":"user","content":[{"type":"text","text":"'$text'"}]}}'
}
{ send_message "what's 2+2?"; sleep 10; send_message "now add 8"; } | \
  amp --execute --stream-json --stream-json-input
```

### 附加标志

- `--stream-json-thinking`：包含思考块（扩展了 schema）
- `--stream-json-input`：stdin 接受逐行 JSON 对象实现多轮对话
- `--stream-json-thinking` 隐含 `--stream-json`

### 提前介入属性

`--stream-json-input` 消息可包含顶层 `"steer": true` 标记为提前介入（在下一个中断点处理）。

### 退出行为

使用 `--stream-json-input` 时，amp 仅在 assistant 完成且 stdin 关闭后退出。

## 非交互环境

用于 CI/CD 或脚本：

```bash
export AMP_API_KEY=your-access-token-here
```

## 提示词编辑

- **Enter** 提交
- **Shift+Enter** 插入换行（需要终端支持：Ghostty、Wezterm、Kitty、iTerm2，或带 `extended-keys` 的 tmux）
- **Ctrl+J** 在任何终端中插入换行
- **`\` + 回车** 插入换行
- **`editor` 命令** 从命令面板触发（需要 `$EDITOR`）

## 键位绑定

| 快捷键 | 操作 |
|--------|------|
| **Ctrl+O** | 命令面板（最重要的） |
| **Ctrl+G** | 在 `$EDITOR` 中打开提示词 |
| **Ctrl+S** | 切换 agent 模式 |
| **Ctrl+R** | 提示词历史 |
| **↑/↓** | 导航排队/上一条消息、编辑 |
| **Alt+T** | 展开/折叠思考/工具块 |
| **Alt+D** | 切换推理强度 |
| **Alt+R** | 切换当前模型的快速模式 |
| **Ctrl+\** | 显示/聚焦/隐藏线程侧边栏 |
| **Ctrl+C Ctrl+N** | 归档线程 + 开始新线程 |
| **Ctrl+C Ctrl+E** | 归档线程 + 退出 |
| **Ctrl+C Ctrl+C** | 退出 |
| **@** | 提及文件 |

### 自定义键位

```json
// ~/.config/amp/settings.json
{
  "amp.keymap": {
    "thread.copyURL": "<leader> u",
    "thread.archive": "<leader> e"
  }
}
```

- 和弦：`"ctrl+c ctrl+e"`
- `<leader> u` 意为 Ctrl+X 然后 u
- 通过键位映射条目配置 `<leader>`：`"leader": "ctrl+x"`
- 设为 `null` 解绑
