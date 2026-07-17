# 快速开始

> 来源：[https://ampcode.com/manual](https://ampcode.com/manual)

## 安装

### Mac / Linux / WSL

```bash
curl -fsSL https://ampcode.com/install.sh | bash
```

### Windows (PowerShell)

```powershell
powershell -c "irm https://ampcode.com/install.ps1 | iex"
```

### Homebrew

```bash
brew install ampcode/tap/ampcode
```

### npm（不推荐）

```bash
npm install -g @ampcode/cli
```

### 更新

```bash
amp update
```

## IDE 集成

| IDE | 步骤 |
|-----|------|
| **VS Code / Cursor / Windsurf** | 安装 CLI，确保编辑器在运行，执行 `amp` |
| **JetBrains** | 安装 CLI，执行 `amp --jetbrains` |
| **Neovim** | 安装 CLI + [Amp Neovim 插件](https://github.com/ampcode/amp.nvim)，执行 `amp` |
| **Zed** | 安装 CLI，确保 Zed 在运行，执行 `amp` |

通用连接方式：命令面板（`Ctrl+O`）→ `ide connect`

## 登录

访问 <https://ampcode.com/install> 登录以保持登录状态。
