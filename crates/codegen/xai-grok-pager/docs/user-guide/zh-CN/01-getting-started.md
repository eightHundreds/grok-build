# 快速开始

Grok Build 是 SpaceXAI 出品的终端 AI 编程助手。它以 TUI（终端用户界面）运行，能理解你的代码库、执行 shell 命令、编辑文件、搜索网页并管理工作。

你可以把它当作全屏 TUI 交互使用，也可以无界面运行以配合脚本和 CI/CD，或通过 Agent Client Protocol (ACP) 接入编辑器。

---

## 安装

本 fork 安装的是**本仓库**的 GitHub Releases，不是官方 `x.ai/cli`。
fork 特有行为见仓库根目录的 [`FORK.md`](https://github.com/eightHundreds/grok-build/blob/main/FORK.md)。

```bash
curl -fsSL https://github.com/eightHundreds/grok-build/releases/latest/download/install.sh | bash
```

安装指定版本：

```bash
curl -fsSL https://github.com/eightHundreds/grok-build/releases/latest/download/install.sh | bash -s 1.0.24-fork.1
```

在 **Windows (PowerShell)** 上，使用原生 PowerShell 安装脚本：

```powershell
irm https://github.com/eightHundreds/grok-build/releases/latest/download/install.ps1 | iex
```

安装指定版本：

```powershell
$env:GROK_VERSION="1.0.24-fork.1"; irm https://github.com/eightHundreds/grok-build/releases/latest/download/install.ps1 | iex
```

PowerShell 安装脚本会自动把 `%USERPROFILE%\.grok\bin` 加入用户 PATH。也可以通过 [Git for Windows](https://gitforwindows.org/)（Git Bash）或 MSYS2，使用上面的 bash 脚本安装。WSL 用户会自动得到 Linux 二进制。

验证安装：

```bash
grok --version
```

随时更新到最新版本：

```bash
grok update
```

在 Grove 配置里设置 `[clone] enabled = true` 之后，通过 Grove（macOS 上是 NFS，Linux 上是 FUSE）拉取仓库：

```bash
grok clone <url> [dir]
```

默认是对所选分支做 depth-1 检出。传入 `--full-history`
可做完整克隆。是否启用 clone 与会话 / `-w` 的 Grove
工作树相互独立。远程 Git 凭据来自 Grove daemon，不是下面
grok.com 登录那一套——见 [grok clone](27-grok-clone.md#authentication)
和 [配置参考](26-config-reference.md)。

---

## 首次启动

运行以下命令启动 Grok：

```bash
grok
```

首次启动时，Grok 会打开浏览器，让你在 grok.com 上认证。登录后，Grok 把凭据存到 `~/.grok/auth.json`，跨会话保留。Grok 会自动刷新凭据；无法续期时会再提示你登录。

如果更想用 API key 认证（例如 CI/CD 或没有浏览器的环境），改为设置 `XAI_API_KEY` 环境变量：

```bash
export XAI_API_KEY="xai-..."
grok
```

完整认证选项（含 OIDC、外部认证提供方、设备码流程）见 [认证](02-authentication.md)。

---

## 基本交互

认证完成后，Grok 呈现全屏 TUI，主要有两块区域：

- **回看区** —— 对话历史，显示你的提示、Grok 的回复、工具调用、文件编辑等。
- **提示框** —— 底部输入区，在这里输入消息。

输入消息后按 `Enter` 发送。Grok 会按需读文件、跑命令、改代码。每次工具运行都会实时流入回看区。

按 `Tab` 在提示框和回看区之间切换焦点。回合进行中，输入框为空时 `Ctrl+C` 取消该回合——若有草稿，第一次按只会清空草稿。`Esc` 从不取消回合；回合中按它会提示改用 `Ctrl+C`。空闲时，800ms 内连按两次 `Esc` 可清空非空提示框，或（提示框为空且已有对话消息时）打开 rewind——见 [键盘快捷键](03-keyboard-shortcuts.md#escape)。回看区获得焦点时，用方向键选中条目并折叠或展开。若要用 `j`/`k` 导航、`h`/`l` 折叠，请开启 Vim 模式。

### 文件引用

在提示里用 `@` 附加文件：

```
@src/main.rs              # 附加一个文件
@src/main.rs:10-50        # 附加第 10–50 行
@src/                     # 浏览目录
```

`@` 操作符会打开模糊文件选择器。默认遵守 `.gitignore` 并隐藏点文件。加前缀 `!` 可搜索隐藏文件：

```
@!.github                 # 搜索隐藏文件
@!.env                    # 附加 .env 文件
```

### 权限

默认情况下，Grok 在执行 shell 命令或编辑文件前会征求许可。你可以逐条批准，也可以切换始终批准模式：

- 按 `Ctrl+O` 切换始终批准模式
- 启动时加 `--yolo` 标志：`grok --yolo`
- 在提示框里输入 `/always-approve` 切换该模式

---

## 核心概念

### 会话

每一段对话都是一个**会话**。会话会自动保存到 `~/.grok/sessions/`，之后可以恢复。每个会话跟踪完整对话历史、工具调用、文件编辑和任务状态。

- 开始新会话：`Ctrl+N` 或 `/new`
- 恢复先前会话：在 TUI 里用 `/resume`，或从 CLI 用 `--resume <ID>`
- 继续最近一次会话：`grok -c`

### 回看区

回看区是主显示区域。它显示：

- **用户提示** —— 你的消息，渲染为粘性标题
- **Agent 消息** —— Grok 的回复，带完整 markdown 渲染和语法高亮
- **思考块** —— Grok 的推理过程（可折叠）
- **工具调用** —— 文件编辑（含行内 diff）、命令执行、搜索结果等
- **任务列表** —— 跟踪进度的 TODO 项

用 `Left`/`Right` 方向键折叠或展开选中条目（Vim 模式下也可用 `h`/`l` 和 `e`）。Vim 模式下，按 `y` 复制其内容，按 `Y` 复制其元数据（例如实际跑过的命令）。按 `Enter` 在全屏查看器中打开（任意模式均可）。

### 工具

Grok 内置这些工具：

| 工具 | 说明 |
|------|-------------|
| `read_file` / `search_replace` | 读取并编辑文件，精确到行 |
| `grep` | 用正则搜索整个代码库（由 ripgrep 驱动） |
| `list_dir` | 列出目录内容 |
| `run_terminal_command` | 执行 shell 命令 |
| `web_search` / `web_fetch` | 搜索网页并抓取 URL |
| `todo_write` | 创建并管理任务列表 |
| `spawn_subagent` | 生成并行子 agent 会话 |
| `memory_search` | 搜索跨会话记忆 |

工具可通过 [MCP 服务器](05-configuration.md#mcp-servers) 扩展，接入 GitHub、数据库等。

### 斜杠命令

在提示框里输入 `/` 可访问命令。这些命令提供快捷操作，不必写完整提示：

```
/model grok-4.6                 # 切换模型
/compact                          # 压缩对话历史
/always-approve                   # 切换始终批准模式
/new                              # 开始新会话
```

完整参考见 [斜杠命令](04-slash-commands.md)。

---

## 常用启动选项

```bash
# 启动交互式 TUI，并把初始提示作为第一回合提交
grok "fix the failing auth test and run it"

# 在新的 git 工作树里带初始提示。用 --worktree=<name>（带 `=`），以免
# 提示被当成工作树名吞掉 —— `grok -w "refactor module X"`
# 会把 "refactor module X" 当作工作树标签，而不是提示。
grok --worktree=feat "refactor module X"

# 以指定分支（例如 main）而不是当前 HEAD 作为工作树基底：
grok -w --ref main "implement feature from main"


# 在指定项目目录启动
grok --cwd ~/projects/my-app

# 添加项目级规则
grok --rules "Always use TypeScript. Prefer functional components."

# 自动批准所有工具执行
grok --yolo

# 使用指定模型
grok -m grok-4.6

# 恢复先前会话
grok --resume <session-id>

# 继续最近一次会话
grok -c

# 实验性的回看区原生渲染模式。会记住：之后直接跑 `grok` 会按上次
# 通过 --minimal/--fullscreen（或 /minimal//fullscreen）选择的模式打开。
grok --minimal

# 回到标准全屏 TUI（并再次记住）
grok --fullscreen

# 无界面模式（给脚本用）
grok -p "Explain this codebase"
```

---

## 无界面模式

以非交互方式运行 Grok，用于脚本、CI/CD 和自动化：

```bash
grok -p "Your prompt here"
```

输出格式：

| 格式 | 标志 | 说明 |
|--------|------|-------------|
| `plain` | （默认） | 人类可读文本 |
| `json` | `--output-format json` | 单个 JSON 对象，含 `text`、`stopReason`、`sessionId` 和 `requestId` |
| `streaming-json` | `--output-format streaming-json` | NDJSON 事件流，便于实时处理 |

CI/CD 用法示例：

```bash
grok -p "Review changes for bugs" --output-format json --yolo | jq -r '.text'
```

---

## 项目规则（AGENTS.md）

在仓库里创建 `AGENTS.md` 即可添加按项目的说明。Grok 会读取这些文件，并在对话开始时把内容注入为项目说明消息：

```
~/.grok/AGENTS.md           # 全局规则（适用于所有项目）
<repo-root>/AGENTS.md       # 仓库级规则
<cwd>/AGENTS.md             # 目录级规则（优先级最高）
```

更深层的文件优先。为了兼容，Grok 也会读取 `CLAUDE.md`。

---

## 接下来看什么

| 文档 | 你会学到什么 |
|----------|-------------------|
| [认证](02-authentication.md) | 浏览器登录、API key、OIDC、外部认证、设备码流程 |
| [键盘快捷键](03-keyboard-shortcuts.md) | 全部按键绑定的完整参考 |
| [斜杠命令](04-slash-commands.md) | 所有可用的 `/` 命令 |
| [配置](05-configuration.md) | config.toml、pager.toml、环境变量 |
