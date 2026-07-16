# 配置参考

> 来源：[https://ampcode.com/manual](https://ampcode.com/manual)

## 配置文件位置

### 用户设置

- macOS/Linux：`~/.config/amp/settings.json` 或 `.jsonc`
- Windows：`%USERPROFILE%\.config\amp\settings.json` 或 `.jsonc`

### 工作区设置

最近的 `.amp/settings.json` 或 `.jsonc`，从 cwd 向上搜索到仓库根目录。

### 自定义用户设置

```bash
amp config edit        # 编辑用户设置
amp config edit --workspace # 编辑工作区设置
```

工作区相同设置会覆盖用户设置。所有设置使用 `amp.` 前缀。

## 设置项参考

| 设置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `amp.fuzzy.alwaysIncludePaths` | `array` | `[]` | 始终包含在模糊搜索中的 glob 模式（即使被 gitignore） |
| `amp.showCosts` | `boolean` | `true` | 在 CLI 中显示线程成本 |
| `amp.git.commit.ampThread.enabled` | `boolean` | `true` | 在 agent 提交中添加 `Amp-Thread: <url>` 尾注 |
| `amp.git.commit.coauthor.enabled` | `boolean` | `true` | 添加 `Co-authored-by: Amp <amp@ampcode.com>` 尾注 |
| `amp.keymap` | `object` | `{}` | 自定义键位绑定（用户覆盖工作区） |
| `amp.mcpServers` | `object` | — | MCP 服务器定义 |
| `amp.defaultVisibility` | `object` | — | 每个仓库来源的默认线程可见性 |
| `amp.notifications.enabled` | `boolean` | `true` | 任务完成或阻塞时播放声音 |
| `amp.remoteThreadCreation.enabled` | `boolean` | `false` | 允许 ampcode.com 在此机器上创建线程 |
| `amp.skills.disableClaudeCodeSkills` | `boolean` | `false` | 禁止从 Claude Code 目录加载 skill |
| `amp.skills.path` | `string` | — | 额外的 skill 目录（冒号分隔） |
| `amp.terminal.copyOnSelect` | `boolean` | `true` | 选择文本时自动复制 |
| `amp.terminal.detailsExpandedByDefault` | `boolean` | `false` | 默认展开思考/工具调用详情 |
| `amp.thread.autoArchiveOnQuit` | `boolean` | `false` | 退出时自动归档打开的 CLI 线程 |
| `amp.tools.disable` | `array` | `[]` | 按名称禁用工具（支持 glob） |
| `amp.mcpPermissions` | `array` | `[]` | MCP 服务器允许/拒绝规则 |
| `amp.updates.mode` | `string` | `"auto"` | 更新模式：`auto`、`warn`、`disabled` |

## MCP 权限格式

```json
"amp.mcpPermissions": [
  { "matches": { "command": "npx", "args": "* @playwright/mcp@*" }, "action": "allow" },
  { "matches": { "url": "https://mcp.trusted.com/mcp" }, "action": "allow" }
]
```

拒绝所有服务器：
```json
[
  { "matches": { "command": "*" }, "action": "reject" },
  { "matches": { "url": "*" }, "action": "reject" }
]
```

## 企业托管设置

企业管理员通过机器级文件强制执行策略：

- **macOS**：`/Library/Application Support/ampcode/managed-settings.json`
- **Linux**：`/etc/ampcode/managed-settings.json`
- **Windows**：`%ProgramData%\ampcode\managed-settings.json`

附加字段：`amp.admin.compatibilityDate`（YYYY-MM-DD 格式的字符串）用于设置迁移。

## 代理与证书

标准 Node.js 环境变量：
- `HTTP_PROXY`
- `HTTPS_PROXY`
- `NODE_EXTRA_CA_CERTS`
