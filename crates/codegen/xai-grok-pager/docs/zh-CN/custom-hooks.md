# 自定义 Hooks 指南

Hooks 让你在 Grok 会话的关键时刻运行自定义脚本或 HTTP 请求，例如工具运行之前或之后、会话开始或结束时，或 agent 发送通知时。

用它们做自动化、安全检查、日志、通知，以及和你自己的工具集成。

## 为什么用 Hooks？

常见用途：

- **安全护栏**：在执行前拦截危险命令，例如 `rm -rf /`。
- **审计日志**：把每次工具使用或会话记录到文件或外部服务。
- **通知**：长任务完成时发一条 Slack/Discord 消息。
- **自动格式化**：编辑后自动运行 `cargo fmt` 或 `prettier`。
- **环境准备**：会话开始时导出密钥或设置变量。
- **自定义工作流**：在特定事件上触发构建、测试或部署。

## 快速开始

1. 创建 hooks 目录：
   ```sh
   mkdir -p ~/.grok/hooks
   ```

2. 创建一个简单的 hook 文件，例如 `~/.grok/hooks/session-start.json`：
   ```json
   {
     "hooks": {
       "SessionStart": [
         {
           "hooks": [
            { "type": "command", "command": "echo \"🚀 Grok session started in $(pwd)\"" }
           ]
         }
       ]
     }
   }
   ```

3. 启动（或重启）一个 Grok 会话。hook 会在 `SessionStart` 时自动运行。

   要确认它已加载，打开 Hooks 标签：在 VS Code 家族之外按 `Ctrl+L`，或在任意处运行 `/hooks`（在 VS Code、Cursor、Windsurf 和 Zed 上更推荐后者）。

## Hook 位置

Hooks 会从多个位置发现（全部合并）：

| 范围      | 路径                              | 受信任？     | 说明 |
|-----------|-----------------------------------|--------------|-------|
| 全局      | `~/.grok/hooks/*.json`            | 始终         | 最适合个人 hooks |
| 全局      | `~/.claude/settings.json`         | 始终         | Claude Code 兼容 |
| 项目      | `<project>/.grok/hooks/*.json`    | 需要信任     | 按仓库自动化 |
| 项目      | `<project>/.claude/settings.json` | 需要信任     | Claude 兼容 |
| 配置      | `config.toml`、`managed_config.toml`、`requirements.toml` | 始终 | 写在你（或组织）配置里的 hooks |
| 插件      | 打包在已安装插件内                | 按插件       | 团队共享 hooks |

配置文件里的 hooks 用同一套 schema 的 TOML 形式；细节见 [Hooks 用户指南](user-guide/10-hooks.md#hooks-in-config-files)。

**信任一个项目**：第一次打开带 hooks 的项目时，打开 hooks 模态框（VS Code 家族之外用 `Ctrl+L`，任意终端用 `/hooks`）或运行 `/hooks-trust`。这和 `--trust` 是同一套文件夹信任门闩，记录在 `~/.grok/trusted_folders.toml`。信任可以防止未受信任的仓库运行任意代码。

## Hook JSON 格式

每个 `.json` 文件可以定义多个 hooks：

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          { "type": "command", "command": "bin/safety-check.sh", "timeout": 10 }
        ]
      }
    ],
    "PostToolUse": [
      {
        "hooks": [
          { "type": "command", "command": "bin/log-activity.sh" }
        ]
      }
    ]
  }
}
```

关键字段：

- **事件名**（顶层键）：`SessionStart`、`UserPromptSubmit`、`PreToolUse`、`PostToolUse`、`Stop`、`Notification`、`SessionEnd` 等。
- **matcher**（可选）：对事件匹配值做正则测试：工具事件上是工具名，其他事件见用户指南的 Hooks 章节。空则匹配一切。
- **type**：`"command"`（运行脚本或 shell 一行命令）或 `"http"`（把事件 POST 到 URL）。
- **command**：可执行文件路径（相对 JSON 文件）或内联 shell 命令。
- **timeout**：杀掉 hook 前的秒数（默认：5，或对 `Stop`/`SubagentStop`/`PostToolUse` 门闩为 600）。超时失败即放行。

**工具名别名**：Claude 风格名称如 `Bash`、`Edit`、`Read` 会自动匹配 Grok 的内部名称（`run_terminal_cmd`、`search_replace`、`read_file`）。

## 编写 Hook 脚本

### 输入
完整事件以 JSON 形式从 **stdin** 送入。`PreToolUse` hook 的示例：

```json
{
  "hookEventName": "pre_tool_use",
  "hook_event_name": "PreToolUse",
  "sessionId": "abc-123",
  "cwd": "/Users/you/project",
  "workspaceRoot": "/Users/you/project",
  "toolName": "run_terminal_cmd",
  "toolInput": { "command": "npm test" },
  "timestamp": "2026-04-14T12:00:00Z"
}
```

`hook_event_name`（snake_case 键）携带 Claude 的 PascalCase 值；`hookEventName`（camelCase 键）携带 grok 的 snake_case 值。

### 输出（用于 PreToolUse 这类会拦截的 hooks）
把 JSON 写到 **stdout**：

- 允许：`{"decision": "allow"}`
- 拒绝：`{"decision": "deny", "reason": "Unsafe command detected"}`

**退出码**（行为因 hook 类型而异）：
- `0`：成功 / 允许（对会拦截的 hooks）。
- `2`：显式拒绝（`PreToolUse`）、用 stderr 作为反馈阻止停止（`Stop`/`SubagentStop`；见用户指南的 Stop Decision Control），或把反馈交给模型（`PostToolUse`，工具已经跑完，但其 stderr 仍会到达模型）。
- 其他任何情况（包括超时、崩溃或缺少环境变量）：**失败即放行**。失败会记日志，并在回滚里占一行，但不会拦截工具调用。要拦截工具调用，请在 stdout 返回 JSON `{"decision":"deny","reason":"..."}`。

### PostToolUse 输出
`PostToolUse` 在工具结束后运行，因此什么也不拦截，但其 stdout 决定模型接下来看到什么。`{"decision":"block","reason":"..."}` 会把原因连同结果一起交给模型。`hookSpecificOutput.additionalContext` 追加一条说明。`hookSpecificOutput.updatedToolOutput`（内置工具；必须匹配该工具自己的输出形态）或 `hookSpecificOutput.updatedMCPToolOutput`（MCP 工具；不检查形态）会替换模型读到的输出，而回滚和遥测仍保留原文。每个 hook 的 block 原因和上下文按调用顺序送达，每条都带上 hook 名称。只有替换是后写者胜出，非零退出的 hook 只保留其 block 原因。输出替换仅限 settings 文件：SDK 注册的 `PostToolUse` hook 可以贡献 `block` 原因和 `additionalContext`，但不能替换工具输出。见用户指南的 PostToolUse Output。

### 被动 hooks
对 `SessionStart` 或 `Notification` 这类事件，stdout 会被忽略。成功时退出 0 即可。

### 有用的环境变量

Grok 会向每个 hook 进程注入以下变量：

- `GROK_HOOK_EVENT`：事件名（例如 `pre_tool_use`、`session_start`、`post_tool_use`）。
- `GROK_HOOK_NAME`：此 hook 的完整配置名。
- `GROK_SESSION_ID`：当前会话标识。
- `GROK_WORKSPACE_ROOT`：工作区根的绝对路径。

由插件提供的 hooks 还会设置：

- `GROK_PLUGIN_ROOT`：插件安装目录的绝对路径。
- `GROK_PLUGIN_DATA`：插件可写数据目录的绝对路径。

这些由 runner 和插件注入的变量始终优先。试图通过 `env` 字段覆盖保留的 runner 键会在加载时被剥掉（并记一条警告）。对插件 hooks，`GROK_PLUGIN_ROOT` 和 `GROK_PLUGIN_DATA` 同样会覆盖用户为这些键提供的任何值。

### 自定义环境变量（`env` 字段）

每个 handler 可以声明额外的环境变量注入子进程：

```json
{
  "type": "command",
  "command": "bin/check.sh",
  "env": {
    "MY_API_TOKEN": "secret-here",
    "LOG_LEVEL": "debug"
  }
}
```

值必须是 **字符串**。JSON 数字和布尔值目前解析会失败；需要时请用引号包起来。

对插件 hooks，插件适配器还会注入
`GROK_PLUGIN_ROOT` 和 `GROK_PLUGIN_DATA`。这些键会覆盖用户为同名声明的
任何值（插件契约不可协商）。

### 变量替换

`command` 和 `url` 字符串在配置加载时支持 `$VAR` 和 `${VAR}` 替换：

```json
{
  "type": "command",
  "command": "${HOME}/.config/grok-hooks/check.sh"
}
```

每个引用的查找顺序：
1. handler 自己的 `env` 映射。
2. 当前进程环境（Grok 自己看到的环境）。

两边都未设置时，引用会 **原样保留**（例如 `${UNSET}`
仍是字面字符串）。Runner 注入的名称（`CLAUDE_PROJECT_DIR`、
`GROK_WORKSPACE_ROOT`、`GROK_HOOK_EVENT`、`GROK_HOOK_NAME`、
`GROK_SESSION_ID`）在加载时不会从 Grok 进程环境取值。Unix `sh -c` 会从子进程环境展开它们；Windows PowerShell
会把 `$VAR` 改写成 `$env:VAR`。HTTP `url` 在请求时替换它们。命令里仍未解析的引用会被拒绝，并报 "required env
var(s) not set"。

对 HTTP hooks，`url` 还会在 **请求时** 再展开一次
（恰好在 SSRF 校验之前），因此 `${GROK_PLUGIN_ROOT}/check` 这类插件注入变量会按插件的实际路径解析。

#### 参数展开修饰符

POSIX 参数展开形式在加载时 **从不** 展开。它们会原样留给运行时的 `sh -c` 分支处理：`${VAR:-default}`、
`${VAR-default}`、`${VAR:=x}`、`${VAR:?msg}`、`${VAR:+x}`、`${VAR%pat}`、
`${VAR#pat}`、`${VAR/pat/repl}`、`${VAR:N:M}`。这避免加载时展开器和 POSIX shell 语义出现细微分歧（尤其是
`:-` 对空字符串的行为）。

如果 hook 命令包含 shell 元字符（空格、管道、`&&`、
重定向、`$` 等），runner 会通过 `sh -c` 运行，你得到完整的
shell 展开语义。如果命令是没有元字符的裸路径，
runner 会直接启动它。即便如此，路径里的 `$VAR` / `${VAR}` 引用仍会在加载时解析，因此像
`${HOME}/bin/check.sh` 这样的直接执行路径无需包进 `sh -c`。

#### 什么不会被展开

- **`matcher`** 是正则（`$` 是行尾锚点）。它
  从不做环境变量展开。替换 `$VAR` 会悄悄改变正则的
  语义，并很可能得到无效模式。若需要动态
  matcher，请在写入时生成 JSON 文件。
- **`timeout`** 是数字，没有可展开的内容。
- **`env` 映射本身的值**：这些按原样存储并
  原样传给子进程，因此 `"BAR": "${HOME}/x"` 会把字面
  字符串 `${HOME}/x` 注入子进程环境。

## 在 TUI 中管理 Hooks

在 VS Code 家族之外按 `Ctrl+L`（或在任意处运行 `/hooks`）打开 Hooks 与插件模态框。

在 **Hooks** 标签里你可以：
- `l`：重新加载全部 hooks。
- `a`：按路径添加自定义 hook（很适合测试）。
- `e`：启用或禁用。
- `r`：移除。
- `Space`：展开分组。

来自 `~/.grok/hooks/` 的 hooks 出现在 **Global** 下，项目的出现在 **Project** 下，以此类推。

## HTTP Hooks

不跑本地脚本，改调远程端点：

```json
{ "type": "http", "url": "https://hooks.example.com/grok-event", "timeout": 15 }
```

完整事件信封会以 JSON POST 出去。适合 webhooks、分析或 serverless 函数。

## 最佳实践

1. **让 hooks 保持快速**：长时间运行的 hooks 会卡住 UI。尽量用后台 `&` 或异步。
2. **用显式 `deny` 来拦截**：hooks 在任何错误（超时、崩溃、缺少环境变量等）上都失败即放行，因此崩溃的 hook 不会拦截工具调用。要执行策略，hook 必须跑完并在 stdout 发出 `{"decision":"deny","reason":"..."}`。
3. **用绝对路径或相对 hook 文件的路径**：JSON 旁边 `bin/` 里的脚本便于移植。
4. **用 Hooks 标签测试**：在 VS Code 家族之外按 `Ctrl+L`，或运行 `/hooks`，在依赖它们之前核实加载和匹配。
5. **把项目 hooks 纳入版本控制**：提交 `.grok/hooks/`（但永远不要提交密钥）。

## 安全说明

- 全局 hooks（`~/.grok/...`）以你的用户权限运行。把它们当 shell 脚本对待。
- 项目 hooks 需要显式信任（运行 `/hooks-trust` 或用模态框），以防恶意仓库的供应链攻击。
- HTTP hooks 会发送会话数据。只使用受信任的端点。

## 排障

- **Hook 没跑？** 在 VS Code 家族之外按 `Ctrl+L`（或在任意处运行 `/hooks`），看它是否已加载并匹配。
- **项目 hooks 被忽略？** 先信任该项目。
- **找不到脚本？** 检查路径是否相对 `.json` 文件，以及是否可执行（`chmod +x`）。
- **`The argument '/.claude/hooks/….ps1' to the -File parameter does not exist`？** PowerShell 把 `$CLAUDE_PROJECT_DIR` 当成了空。除非 `GROK_SHELL=cmd`，Grok 会把它改写成 `$env:CLAUDE_PROJECT_DIR`。
- **看到错误？** 查看 pager 日志（通常在 tracing 窗格或 `~/.grok/logs`）。

## 更多示例

见 `xai-grok-hooks` crate 里的内置示例：

- [Safe Shell Guard](../../../xai-grok-hooks/examples/hooks/safe-shell.json)
- [No Recursive Grep](../../../xai-grok-hooks/examples/hooks/no-recursive-grep.json)：硬拦截 `grep -r`/`grep -R`/`rgrep`（OOM 护栏）
- [Session Audit Log](../../../xai-grok-hooks/examples/hooks/session-log.json)
- [Tool Activity Logger](../../../xai-grok-hooks/examples/hooks/tool-logger.json)

把它们复制到 `~/.grok/hooks/` 再按需改。

## 完整参考

完整事件列表、matcher 语义、信任模型和进阶细节，见 [Hooks 用户指南](user-guide/10-hooks.md)。

---

*玩得开心！* 如果你做出了不错的东西，考虑作为插件分享。
