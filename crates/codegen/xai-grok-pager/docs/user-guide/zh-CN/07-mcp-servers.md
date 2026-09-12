# MCP 服务器

MCP（Model Context Protocol）服务器用外部工具集成扩展 Grok。它们让 Grok 能与任何实现 MCP 标准的服务交互。

---

## 什么是 MCP 服务器？

MCP 服务器是一个通过标准化协议向 Grok 暴露工具的进程。配置 MCP 服务器后，其工具会与 Grok 的内置工具一起提供给模型。模型可以在会话中发现并调用这些工具。

例如，GitHub MCP 服务器可能暴露 `create_issue`、`list_pull_requests` 和 `search_code` 这类工具。数据库服务器可能暴露 `query`、`list_tables` 和 `describe_schema`。

协议细节见 [MCP specification](https://modelcontextprotocol.io)。

---

## 配置

MCP 服务器在 `~/.grok/config.toml` 的 `[mcp_servers.<name>]` 节中配置。

要把 MCP 服务器分发给团队，或限制用户可运行的服务器（`requirements.toml` / `managed_config.toml` 中的 `allowedMcpServers` / `deniedMcpServers`，对外来定义的服务器还有 Claude `managed-settings.json` 的建议），见插件指南中的 [在组织内分发](09-plugins.md#distribute-across-an-organization)。

### stdio 传输（本地进程）

Grok 派生子本地进程，并通过 stdin/stdout 通信：

```toml
[mcp_servers.my-server]
command = "/path/to/server"           # 服务器可执行文件
args = ["--flag", "value"]            # 命令参数
env = { API_KEY = "sk-..." }          # 环境变量
enabled = true                        # 启用或禁用该服务器（默认：true）
startup_timeout_sec = 30              # 服务器启动超时，秒（默认：30）
tool_timeout_sec = 6000               # 每次工具调用超时回退，秒（默认：6000）
tool_timeouts = { slow_op = 120 }     # 按工具的超时覆盖，秒
```

> **全局启动超时覆盖：** 不必按服务器设置 `startup_timeout_sec`，
> 你可以通过 `MCP_TIMEOUT`
> 环境变量（毫秒，与 Claude Code 兼容）或
> `GROK_MCP_STARTUP_TIMEOUT_SECS`（秒）更改所有服务器的默认。按服务器的 `startup_timeout_sec`
> 仍优先于两者。首次启动时下载
> 包的冷启动 `npx`/`uvx` 服务器常常需要这个；默认是 30 秒。
>
> **MCP 工具结果大小上限：** 大型 MCP / `use_tool` 结果会就地截断
> （完整载荷溢出到会话 `mcp/` 文件夹下）。默认是
> **20_000 字节**。通过以下方式覆盖：
>
> - 环境变量 `GROK_MAX_MCP_OUTPUT_BYTES` 或 `MAX_MCP_OUTPUT_BYTES`（字节；两者都设置时 Grok 原生
>   优先；名字是 Claude 风格，但我们按**字节**而不是 token 限制）
> - `config.toml` —— 用户级（`~/.grok/config.toml`）**或仓库级**
>   （cwd → git 根链上任意位置的 `.grok/config.toml`；最深的
>   文件赢，仓库值仅在该文件夹受信任后应用）：
>
> ```toml
> [mcp]
> max_output_bytes = 40000
> ```
>
> 优先级：requirements.toml > 环境 > 仓库 `.grok/config.toml` >
> 用户/托管配置 > 默认。仓库编辑通过配置热重载应用于该
> 目录中正在运行的会话。

### HTTP/SSE 传输（远程服务器）

对于可通过 HTTP 访问的远程 MCP 服务器：

```toml
[mcp_servers.remote-api]
url = "https://mcp.example.com/api"
headers = { "Authorization" = "Bearer token" }
```

MCP 数据面请求（JSON-RPC 和 SSE）以及匿名访问探测会带上
默认的 `User-Agent: grok-cli/<version>` 头，其中 `<version>` 是 Grok 二进制
版本。OAuth 发现、客户端注册和 token 请求由
rmcp OAuth 客户端发出，并保持其自身行为（没有默认 `User-Agent`）。服务器 `headers` 中有效的
`User-Agent` 条目会覆盖默认；无效的
已配置 `User-Agent` 值会被头解析丢掉（并记警告），因此这样的
服务器仍会收到默认。例外：Figma MCP 服务器（服务器名 `figma`、
遗留托管名 `grok_com_figma`，或 `figma.com` 主机——都不区分大小写）
发送不带版本的裸 token `grok-cli`，除非配置提供了自己的
`User-Agent`。

### 带会话 ID 的可流式 HTTP

```toml
[mcp_servers.my-streamable-server]
url = "https://mcp.example.com/api/mcp"
headers = { "x-mcp-session-id" = "{{session_id}}" }
```

---

## CLI 管理

从命令行管理 MCP 服务器，不必编辑配置文件：

```bash
# 列出已配置的 MCP 服务器
grok mcp list
grok mcp list --json          # 机器可读输出

# 添加 stdio 服务器。-- 之后的一切都是服务器命令，因此
# -y 这类标志会到达服务器，而不是被 grok 解析。
grok mcp add filesystem -- npx -y @modelcontextprotocol/server-filesystem /path/to/dir

# 添加带环境变量的 stdio 服务器（-e 可重复）
grok mcp add postgres -e DATABASE_URL=postgres://localhost/mydb -- npx -y @modelcontextprotocol/server-postgres

# 添加远程 HTTP 服务器
grok mcp add --transport http sentry https://mcp.sentry.dev/mcp

# 添加带认证头的远程服务器（--header 可重复）
grok mcp add --transport http api https://mcp.example.com/mcp --header "Authorization: Bearer YOUR_TOKEN"

# 添加远程 SSE 服务器
grok mcp add --transport sse linear https://mcp.linear.app/sse

# 移除服务器
grok mcp remove github

# 启用或禁用本地/TOML（或兼容来源）服务器
grok mcp enable github
grok mcp disable github

# 诊断服务器的配置和连通性
grok mcp doctor               # 检查每个已配置服务器
grok mcp doctor github        # 检查一台服务器
grok mcp doctor --json        # 机器可读输出
```

传输默认是 `stdio`；对远程服务器传入 `--transport http` 或 `--transport sse`。

默认情况下 `grok mcp add` 写到 `~/.grok/config.toml`（`--scope user`）。用 `--scope project` 改为写到当前目录的 `.grok/config.toml`，可以提交并与团队共享（见 [项目范围 MCP 服务器](#project-scoped-mcp-servers)）。头和环境变量值按原样存储，因此把秘密写成 `${VAR}`，而不是粘进已提交的项目配置（见 [示例配置](#example-configurations)）。`grok mcp list` 显示两个范围的服务器，项目范围的标为 `(project)`，禁用的标为 `(disabled)`。

`grok mcp remove` 搜索两个范围，移除服务器后以 0 退出。找不到名称时，或名称同时定义在用户和项目范围时以 1 退出——传入 `--scope` 说明要移除哪一个。

`grok mcp enable` / `disable` 把个人开/关状态持久化到用户 `~/.grok/config.toml`（`disabled_mcp_servers`，以及该条目存在时的 `[mcp_servers.<name>].enabled`）。范围：

- **已知名称：** 用户/项目 Grok TOML、已在禁用列表上的名称、兼容来源（`.mcp.json`、Claude、Cursor），以及 **插件** MCP 服务器（与 doctor/`/mcps` 相同的发现）。
- **仅启用：** 若 cwd 最近的项目定义有粘性的 `enabled = false`，会清掉那一个键（保留注释）；禁用从不改写项目配置。
- **不是完整的 `/mcps` 对等：** 网关连接器（`managed_gateway:…`，存在 `disabled_mcp_tools.__managed_gateway_connectors` 下）在 TUI 里仍仅限 Space。幂等；未知名称以 1 退出。

相对早期版本的破坏性变更：`--env` 现在每个标志只接受一个 `KEY=value`（用 `-e A=1 -e B=2`，而不是 `--env A=1 B=2`），服务器名只能包含字母、数字、连字符和下划线。

---

## 项目范围 MCP 服务器

通过在仓库中放置 `.grok/config.toml`，可以按项目配置 MCP 服务器：

```
my-project/
  .grok/
    config.toml
  src/
  ...
```

```toml
# .grok/config.toml
[mcp_servers.linear]
url = "https://mcp.linear.app/mcp"
enabled = true
```

当服务器暴露原生 HTTP/SSE 端点时，优先用 `url` 形式，而不是把它包在 `npx mcp-remote <url>` 这类 stdio 代理里。Grok 直接处理 HTTP/SSE 和 OAuth，因此原生形式避免每个会话多一个子进程。它也会向提供方注册 Grok 自己的 OAuth 客户端。

Grok 从当前目录走到 git 仓库根，在每一层加载 `.grok/config.toml`：

| 位置 | 范围 | 优先级 |
|----------|-------|----------|
| `~/.grok/config.toml` | 所有项目 | 最低 |
| `<repo-root>/.grok/config.toml` | 本仓库 | 中 |
| `<cwd>/.grok/config.toml` | 当前目录 | 最高 |

若项目定义了与全局同名的服务器，项目版本会完全替换它（字段不合并）。

项目范围文件贡献 `[mcp_servers]`、`[plugins]` 和 `[permission]` 条目。Grok 大多数其他配置节只从 `~/.grok/config.toml` 读取。

---

## 工具命名

MCP 工具用服务器名做命名空间，以避免冲突：

- 服务器 `filesystem` 的工具 `read_file` 变成 `filesystem__read_file`
- 服务器 `github` 的工具 `create_issue` 变成 `github__create_issue`

---

## 运行时开关服务器

你可以启用或禁用 MCP 服务器而不重启 Grok（TUI `/mcps` 或 CLI——见 [CLI 管理](#cli-management)）。

### /mcps 模态框

在 TUI 中打开 MCP 服务器模态框：

- 作为斜杠命令运行 `/mcps`
- 或按 `Ctrl+L`（非 VS Code 家族）并导航到 MCP Servers 标签；在 VS Code 家族上用 `/plugins` 或 `/mcp` 并打开 MCP Servers 标签

从模态框你可以：

- 查看每台服务器的来源、启用状态和工具数量
- 用 `Space` 启用或禁用一台服务器
- 展开一台服务器以查看它提供的工具
- 编辑 `config.toml` 后用 `r` 刷新列表
- 用 `i` 认证 OAuth 服务器
- 用 `a` 添加服务器，或用 `x` 移除本地服务器（模态框会要求确认；按小写 `y` 移除，或其他任意键取消）

### 工具发现

模型有两个内置工具用于处理 MCP 服务器：

- `search_tool` —— 跨所有已启用 MCP 服务器发现可用的集成工具。用它按名称或描述查找工具。
- `use_tool` —— 调用通过 `search_tool` 发现的集成工具。指定完全限定的工具名（例如 `github__create_issue`）。

---

## 兼容性

Grok 为了兼容会从多个来源加载 MCP 服务器配置：

| 来源 | 格式 | 位置 | 可配置 |
|--------|--------|----------|-------------|
| `config.toml` | 原生 Grok 配置 | `~/.grok/config.toml`、`.grok/config.toml` | 始终开启 |
| `.claude.json` | Claude Code 格式 | `~/.claude.json` | `[compat.claude] mcps` |
| `.cursor/mcp.json` | Cursor 格式 | `~/.cursor/mcp.json`、`<project>/.cursor/mcp.json` | `[compat.cursor] mcps` |
| `.mcp.json` | MCP 标准格式 | 项目根（cwd 到 git 根） | 除非你已导入或关掉 Claude 导入提示（导入标记已设置），否则加载 |

所有来源按优先级合并：config.toml > Claude > Cursor > `.mcp.json`。名称冲突时，更高优先级来源的服务器优先。

默认扫描 Claude 和 Cursor 的 MCP 来源。要关闭某个厂商的扫描，在 `~/.grok/config.toml` 中设 `[compat.<vendor>] mcps = false`，或设对应的环境变量（`GROK_CURSOR_MCPS_ENABLED`、`GROK_CLAUDE_MCPS_ENABLED`）。详见 [配置](05-configuration.md#harness-compatibility)。用 `grok inspect` 查看加载了哪些 MCP 服务器及其厂商来源（`[cursor]`、`[claude]`）。

---

## MCP OAuth

对于需要 OAuth 认证的 MCP 服务器，Grok 自动处理凭据流程。当 MCP 服务器请求 OAuth 凭据时，Grok 打开基于浏览器的授权流程，并存储得到的 token 供将来使用。

---

## 示例配置

对托管 MCP 服务器使用 `url` 形式，对本地 stdio 工具使用 `command` / `args` 形式。

### 原生 HTTP（托管服务）

基于 OAuth 的 MCP 服务器必须先认证才能使用。Grok 把得到的 token 以仅所有者可读写的本地明文（Unix 上是 `0600`）存在 `~/.grok/mcp_credentials.json`。主机上优先开启全盘加密。编辑 `config.toml` 后，在 `/mcps` 模态框按 `r` 刷新服务器列表。

```toml
[mcp_servers.linear]
url = "https://mcp.linear.app/mcp"
enabled = true

[mcp_servers.sentry]
url = "https://mcp.sentry.dev/mcp"
enabled = true

[mcp_servers.mixpanel]
url = "https://mcp.mixpanel.com/mcp"
enabled = true
```

对于用静态 bearer token 而不是 OAuth 认证的内部或自托管服务器，显式设置 `Authorization` 头：

```toml
[mcp_servers.internal-tools]
url = "https://mcp.internal.example.com/mcp"
enabled = true

[mcp_servers.internal-tools.headers]
Authorization = "Bearer <token>"
```

为避免把秘密放进配置文件，用 `${VAR}`（或 `${VAR:-default}`）引用环境变量。Grok 在加载时展开 `[mcp_servers.*]` 中的字符串字段——`url`、`command`、`args`，以及 `env` 和 `headers` 中的值：

```toml
[mcp_servers.internal-tools]
url = "https://mcp.internal.example.com/mcp"
enabled = true
headers = { "Authorization" = "Bearer ${INTERNAL_MCP_TOKEN}" }
```

### 本地 stdio

对必须本地运行的工具（文件系统访问、本地数据库、内部服务器）使用 stdio。

```toml
# 限定到某个目录的文件系统访问
[mcp_servers.filesystem]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/path/to/allowed/directory"]

# 本地 Postgres
[mcp_servers.postgres]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-postgres", "postgresql://user:pass@localhost/db"]

# 带更长启动超时和按工具调优超时的自定义服务器
[mcp_servers.my-tools]
command = "/usr/local/bin/my-mcp-server"
args = ["--config", "/etc/my-mcp.json"]
startup_timeout_sec = 30
tool_timeout_sec = 120
tool_timeouts = { slow_analysis = 300, quick_lookup = 10 }
```

在 Windows 上，npm 把 `npx`、`npm`、`pnpm` 和 `yarn` 这类启动器安装为 `.cmd` 批处理垫片（没有 `npx.exe`）。Grok 在派生子之前把裸 `command`（例如 `npx`）解析为 `PATH` 上的真实启动器路径（尊重 `PATHEXT`），因此不必手工用 `cmd /c` 包裹。作为绝对路径或包含路径分隔符给出的 `command` 会原样使用。

---

## 可用的 MCP 服务器

一份可用 `url` 或 `command` 形式配置的 MCP 服务器部分列表。使用前向各提供方确认当前端点或包名：

| 服务器 | 传输 | 端点 / 包 |
|--------|-----------|--------------------|
| Linear | HTTP（OAuth） | `https://mcp.linear.app/mcp` |
| Sentry | HTTP（OAuth） | `https://mcp.sentry.dev/mcp` |
| Mixpanel | HTTP（OAuth） | `https://mcp.mixpanel.com/mcp` |
| Filesystem | stdio | `@modelcontextprotocol/server-filesystem` |
| Git | stdio | `@modelcontextprotocol/server-git` |
| GitHub | stdio | `@modelcontextprotocol/server-github` |
| GitLab | stdio | `@modelcontextprotocol/server-gitlab` |
| PostgreSQL | stdio | `@modelcontextprotocol/server-postgres` |
| SQLite | stdio | `@modelcontextprotocol/server-sqlite` |
| Puppeteer | stdio | `@modelcontextprotocol/server-puppeteer` |

社区服务器完整列表见 [MCP Server Registry](https://github.com/modelcontextprotocol/servers)，协议细节见 [MCP specification](https://modelcontextprotocol.io)。

---

## 子 agent 与 MCP

子 agent 默认继承父会话已连接的 MCP 服务器，包括来自插件的 agent。用 agent frontmatter 的 `mcpInheritance` 限制该集合（`all`、`none`、`named` 或 `except`）。细节见 [子 agent — MCP 继承](16-subagents.md#mcp-inheritance)。

若子项列出了 `search_tool` / `use_tool` 但返回空目录，检查：

1. 父会话确实连接了该服务器（见 Extensions / `grok inspect`）
2. 该 agent 的 `mcpInheritance` 不是 `none` 或排除该服务器的过滤器
3. 插件 agent 不能在 frontmatter 中声明自己的 `mcpServers`——它们只看见父会话已连接的服务器

---

## 排障

### 服务器未启动

```bash
# 手工测试服务器命令
npx -y @modelcontextprotocol/server-filesystem /path

# 增加启动超时
# 在 config.toml 中：
[mcp_servers.filesystem]
startup_timeout_sec = 30
```

对 stdio 服务器，Grok 把进程的标准错误捕获到 `~/.grok/logs/mcp/<server>.stderr.log`，每次启动截断。服务器启动了但握手失败时检查此文件：

```bash
tail -f ~/.grok/logs/mcp/filesystem.stderr.log
```

### 被组织策略挡住

若原生 TOML 策略或 Claude `managed-settings.json` 设置了 `deniedMcpServers`、非空的 `allowedMcpServers`，或 `allowManagedMcpServersOnly`，Grok 会在合并时丢掉不匹配的服务器，并记录 `MCP server blocked by managed settings policy`。原生 grok 层绑定每台服务器；Claude 文件只绑定外来定义的服务器。`grok inspect` 显示这些列表、锁定范围以及每台剩余服务器。细节和示例：[限制哪些 MCP 服务器可以运行](09-plugins.md#restrict-which-mcp-servers-can-run)。

### 查看服务器状态

用 `grok inspect` 查看所有已加载的 MCP 服务器及其来源：

```bash
grok inspect          # 人类可读
grok inspect --json   # 机器可读
```

### 调试日志

```bash
RUST_LOG=debug GROK_LOG_FILE=/tmp/grok.log grok
tail -f /tmp/grok.log
```

查找包含 `mcp` 的日志条目，以跟踪服务器启动、工具发现和工具调用执行。
