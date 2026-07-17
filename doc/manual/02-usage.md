# 使用指南

> 来源：[https://ampcode.com/manual](https://ampcode.com/manual)

## Agent 模式

通过 `Ctrl+O` → `mode` 切换四种模式：

| 模式 | 用途 |
|------|------|
| **low** | 快速低消耗，适合小型明确定义的任务 |
| **medium** | 智能/速度/成本的平衡，适合大多数任务 |
| **high** | 深度推理，适合困难任务 |
| **ultra** | 最强模式，适合困难且开放式的任务 |

## 提示词技巧

### 基本原则

- **明确指令**："做 X" 而非 "你能做 X 吗？"
- **单线程单任务**：不要混入无关内容（如 DB 迁移 + CSS 修改混在一个线程里）
- **提供上下文**：如果知道相关文件/命令，直接包含在提示词中
- **仅做研究不改代码**：明确说 "不要修改任何文件"
- **使用 AGENTS.md**：指导测试命令、构建步骤、常见错误
- **告知验证方式**：告诉 agent 如何验证工作结果（命令、URL、日志）

### 示例提示词

```text
"修复这个文件里所有 TypeScript 错误"
"运行测试并修复所有失败的测试"
"给这个 React 组件添加暗色模式开关"
"找出这个代码库中用户认证在哪里处理"
"规划如何给这个应用添加实时聊天，但先不写代码"
"用 3 个子 agent 把这些 CSS 文件转成 Tailwind"
"审查这个 API 设计并提出改进建议"
"对文件执行 git blame 告诉我谁加了那个函数"
"amp -x '这个目录里哪些文件是 markdown？'"
"查看 localhost:3000 让 header 更简约"
```

### 扩展使用场景

- 运行构建命令并修复错误
- 截图本地开发 URL 检查 UI 变化
- git blame 查找谁加了某个特性
- 用子 agent 逐文件转换 CSS 到 Tailwind
- 分析 git diff 寻找改进机会
- 检查 staged diff 删除调试语句
- 通过 git log 找到添加某功能的提交
- 用图表解释类间关系
- 通过 psql 修改数据库记录

## 线程

### 引用线程

- 使用 URL 或 ID 引用：`@T-7f395a45...`
- 输入 `@@` 搜索要提及的线程
- amp 自动读取并提取引用线程中的相关信息

### 查找历史线程

amp 可搜索历史线程及工作区成员的线程：

- **Web feed URL 过滤：**
  - `/feed?time=7d` — 时间窗口（`24h`、`72h`、`7d`、`all`）
  - `/feed?q=label:bug` — 搜索语法：裸词、引号短语
  - 过滤标签：`label:`、`file:`、`project:`、`repo:`、`ref:`、`author:`、`archived:`、`after:`、`before:`

- **自然语言查询：**
  - "找我们讨论过 monorepo 迁移的线程"
  - "显示修改过 src/server/index.ts 的线程"
  - "找 Thorsten 关于索引逻辑的线程"

### 归档线程

归档的线程不会出现在活跃列表中，但仍可查看和通过 `@` 引用。通过命令面板 `thread: archive` 操作。

### 分享线程

线程可通过 <https://ampcode.com/feed> 查看。提供四种可见性级别：

- **Unlisted（未列出的）**：有链接的人 + 工作区成员可见
- **Workspace-shared（工作区共享）**：所有工作区成员可见
- **Group-shared（群组共享）**：特定群组 + 工作区管理员可见（仅 Enterprise）
- **Private（私有）**：仅自己可见（工作区内，工作区管理员也可查看）

设置方式：CLI 中 `Ctrl+O` → `thread: set visibility`；Web 端通过分享菜单。

默认值：不在工作区 → 私有；在工作区 → 共享给工作区成员。工作区管理员可更改默认值。

## 附加图片

- **CLI 中**：按 `Ctrl+V` 从剪贴板粘贴（macOS 也是 Ctrl+V 而非 Cmd+V）
- **Windows 备选**：`Ctrl+O` → `paste image from clipboard`
- 也可通过 `@` 提及图片文件路径
- Windows 上 WezTerm 或 Alacritty 比 Windows Terminal 更可靠

## 编辑与消息排队

- **编辑前一条消息**：按 Tab 键导航到前一条消息，按 `e`
- **排队消息**：在 agent 工作时发送消息，消息会排队直到 agent 完成
- **提前介入**：在命令/思考块边界处按 `Enter Enter` 发送消息
- **中断**：按 `Esc Esc` 强制停止并立即发送

## 远程控制

从 ampcode.com（手机或桌面）继续 CLI 线程。

- 在 CLI 中启动 amp，在 Web 上打开线程，从任何设备发送消息
- 需要开启 **Passkey**（在用户安全设置中）
- 工作区管理员可要求所有成员启用
