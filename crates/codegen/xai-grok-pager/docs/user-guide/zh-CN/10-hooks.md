# 钩子

钩子让你在 Grok 会话的关键时刻运行脚本或发送 HTTP 请求。用它们自动化任务、强制安全检查、记录活动、发送通知，并接入你自己的工具。

---

## 什么是钩子？

钩子是 Grok 在特定生命周期事件发生时调用的 shell 命令或 HTTP 端点。钩子可以：

- **拦截操作**：`PreToolUse` 钩子可以在危险命令运行前拒绝它。
- **让 agent 继续工作**：`Stop` 钩子可以在条件成立之前（例如测试套件通过）阻止 agent 结束本回合，并把原因反馈给模型。
- **响应事件**：`PostToolUse` 钩子可以把每一次工具执行记到文件。
- **在调用跑完后纠正**：`PostToolUse` 钩子可以告诉模型工具结果意味着什么，或替换模型读到的输出——打码密钥、裁掉一大墙日志——而真实结果仍留在记录上。
- **准备上下文**：`SessionStart` 钩子可以导出环境变量或运行设置脚本。

---

## 常见用例

- **安全护栏**：在 `rm -rf /` 这类命令运行前拦截。
- **审计日志**：把工具使用和会话记录到文件或外部服务。
- **通知**：任务完成时发一条消息。
- **自动格式化**：编辑后运行 `cargo fmt` 或 `prettier`。
- **环境准备**：会话开始时导出变量。
- **自定义工作流**：在特定事件上触发构建、测试或部署。

---

## 快速开始

1. 创建钩子目录：

   ```sh
   mkdir -p ~/.grok/hooks
   ```

2. 创建钩子文件，例如 `~/.grok/hooks/session-start.json`：

   ```json
   {
     "hooks": {
       "SessionStart": [
         {
           "hooks": [
             { "type": "command", "command": "echo 'Grok session started in '$(pwd)" }
           ]
         }
       ]
     }
   }
   ```

3. 启动（或重启）一个 Grok 会话。钩子会在 `SessionStart` 时自动运行。

4. 在非 VS Code 系列终端按 `Ctrl+L`（或在任意处运行 `/hooks`——VS Code 系列首选），并检查钩子标签页以确认已加载。

---

## 钩子位置

钩子从多个位置发现（全部合并）：

| 作用域 | 路径 | 受信任？ | 说明 |
|-------|------|----------|-------|
| 全局 | `~/.grok/hooks/*.json` | 始终 | 个人钩子 |
| 全局 | `~/.claude/settings.json`（以及 `settings.local.json`） | 始终 | Claude Code 兼容（可配置） |
| 全局 | `~/.cursor/hooks.json` | 始终 | Cursor 兼容（可配置） |
| 项目 | `<project>/.grok/hooks/*.json` | 需要信任 | 按仓库的自动化 |
| 项目 | `<project>/.claude/settings.json`（以及 `settings.local.json`） | 需要信任 | Claude 兼容（可配置） |
| 项目 | `<project>/.cursor/hooks.json` | 需要信任 | Cursor 兼容（可配置） |
| 配置 | `~/.grok/config.toml` | 始终 | 与其余配置放在一起的钩子 |
| 配置 | `managed_config.toml`（`$GROK_HOME` 和 `/etc/grok`） | 始终 | 组织分发的钩子（服务器同步和本机） |
| 配置 | `requirements.toml`（用户和系统） | 始终 | 需求层中的组织分发钩子 |
| 插件 | 捆绑在已安装插件内 | 按插件 | 团队共享钩子 |

配置文件钩子住在你的组织已经控制的同一份 TOML 里；格式见 [配置文件中的钩子](#hooks-in-config-files)。兼容厂商的钩子源默认会扫描。要禁用某个厂商的扫描，在 `~/.grok/config.toml` 中设置 `[compat.<vendor>] hooks = false`，或使用对应的环境变量。细节见 [配置](05-configuration.md#harness-compatibility)。

**信任项目**：第一次打开带钩子的项目时，必须先信任它，项目钩子才会运行；在此之前它们被静默跳过。运行 `/hooks-trust`（或用 `--trust` 启动）授予信任；决定记在统一的文件夹信任存储（`~/.grok/trusted_folders.toml`）中，与约束仓库本地 MCP/LSP 服务器的是同一道门闩。`~/.grok/hooks/` 中的全局钩子始终受信任，无需条目。这防止不受信任的仓库运行任意代码。

因为钩子统一在文件夹信任之下，`--trust` / `/hooks-trust` 授权会一并信任该文件夹的 **MCP、LSP、钩子、项目指令和项目技能**，并覆盖同一仓库的子目录。该文件夹下嵌套的 git 检出是单独的工作区，不在覆盖范围内。反过来，关闭文件夹信任（`GROK_FOLDER_TRUST=0` 或 `[folder_trust] enabled = false`）会一并放开这些表面。

---

## 钩子事件

事件按三种节奏触发：每个会话一次（`SessionStart`、`SessionEnd`），每个回合一次（`UserPromptSubmit`、`Stop`、`StopFailure`），以及回合内每一次工具调用（`PreToolUse`、`PostToolUse`、`PostToolUseFailure`）。

| 事件 | 何时触发 | 可拦截？ |
|-------|---------------|-----------|
| `SessionStart` | 会话开始。不会为子 agent 自己的会话触发。 | 否 |
| `UserPromptSubmit` | 你提交一条提示。 | 是：可以拦截该提示 |
| `PreToolUse` | 工具即将运行。 | 是：可以拒绝 |
| `PostToolUse` | 工具运行结束（包括内置逻辑错误，例如 `run_terminal_command` 非零退出；派发失败或 MCP 错误结果改为触发 `PostToolUseFailure`）。 | 否，但它可以把反馈喂给模型，并替换模型看到的输出 |
| `PostToolUseFailure` | 工具派发失败，或 MCP 工具返回错误结果。 | 否，但它可以把 `additionalContext` 喂给模型 |
| `PermissionDenied` | 权限系统拒绝一次工具调用。 | 否 |
| `Stop` | agent 回合以真正完结结束（中断改为触发 `StopCancelled`）。 | 是：可以拦截停止 |
| `StopFailure` | 回合因 API 错误结束。 | 否 |
| `StopCancelled` | 回合未完结就结束时，代替 `Stop` 运行：用户中断（Ctrl+C / 客户端停止）、被拒绝的权限提示、`--max-turns` 限制，或无进展退出。 | 否 |
| `Notification` | 需要用户注意的事件（`idle_prompt`、`permission_prompt`、`task_complete`，…）。 | 否 |
| `SubagentStart` | 子 agent 启动。 | 否 |
| `SubagentStop` | 子 agent 的回合结束（触发一次，在子 agent 内，带停止决策控制）。 | 是：可以拦截停止 |
| `PreCompact` | 对话压缩即将运行。 | 否 |
| `PostCompact` | 对话压缩完成。 | 否 |
| `SessionEnd` | 会话结束。子会话带 `subagentType`，因此宿主能区分子会话拆除和自身拆除。 | 否 |

`SubagentEnd` 被接受为 `SubagentStop` 的别名。`PreToolUse` 可以拦截工具调用，`UserPromptSubmit` 可以拦截提示（见下），`Stop`/`SubagentStop` 可以阻止 agent 停止（见 [停止决策控制](#stop-decision-control)）。`PostToolUse` 跑得太晚，拦不住任何东西，但它的 stdout 会被读取：可以把反馈喂给模型，并替换模型看到的工具输出（见 [PostToolUse 输出](#posttooluse-output)）。其他每个事件都是被动的。

### UserPromptSubmit 决策控制

`UserPromptSubmit` 钩子可以拒绝一条提示：退出码 2 拦截（stderr 成为消息），stdout 上的 JSON `{"decision": "block", "reason": "..."}` 在任意退出码下也会拦截。原因会显示给你，永远不会加入模型的上下文。只有你键入的提示可以被拦截：自动唤醒回合（任务和子 agent 完成、调度器触发）以及子 agent 会话以只观察方式运行该钩子。该事件的默认超时是 30 秒；超时或崩溃的钩子失败放行，提示继续。

拦截之后，排在被拦截提示后面的已排队提示不会自动运行：队列会等到你采取行动（发送提示，或编辑 / 移除 / 重排 / 强制运行一行排队项）。被拦截的提示不会被记录：它永远不会进入模型在后续回合看到的对话历史、磁盘上的会话记录，或会话摘要。它仍可见于实时回看，并停在队列最前面供你编辑、重发或丢弃——但会话重启后，被拦截的气泡会从回看消失，正是因为什么都没存。一个有意的例外：客户端的本地提示历史（上箭头回忆）会保留文本，在提交时、钩子运行之前记录，因此丢弃的提示仍可找回。当前限制：放行钩子的 stdout 会被丢弃（没有 `additionalContext`）。

### Cursor 钩子兼容

Grok 接受 Cursor 的 camelCase 钩子事件名，因此 `~/.cursor/hooks.json` 可以原样加载：

| Cursor 事件 | 映射到 |
|---|---|
| `sessionStart`、`sessionEnd` | `SessionStart`、`SessionEnd` |
| `preToolUse`、`postToolUse`、`postToolUseFailure` | `PreToolUse`、`PostToolUse`、`PostToolUseFailure` |
| `beforeShellExecution`、`beforeMCPExecution`、`beforeReadFile` | `PreToolUse` |
| `afterShellExecution`、`afterMCPExecution`、`afterFileEdit` | `PostToolUse` |
| `afterAgentResponse`、`afterAgentThought` | `PostToolUse` |
| `beforeSubmitPrompt` | `UserPromptSubmit` |
| `subagentStart`、`subagentStop` | `SubagentStart`、`SubagentStop` |
| `preCompact`、`stop` | `PreCompact`、`Stop` |

Cursor 的按操作钩子（`beforeShellExecution`、`afterFileEdit` 等）映射到通用的 `PreToolUse`/`PostToolUse` 事件。钩子脚本在 JSON 输入中收到工具名，可以据此过滤，或使用 `matcher` 字段。

---

## 钩子 JSON 格式

每个 `.json` 文件可以为多个事件定义钩子：

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

### 关键字段

- **事件名**（顶层键）：[钩子事件](#hook-events) 中列出的任意事件。Grok 跳过无法识别的事件名，因此共享的 Claude 或 Cursor 设置文件仍能加载。
- **matcher**（可选）：选择哪些调用触发该钩子的正则表达式。它测试的内容取决于事件：工具事件（`PreToolUse`、`PostToolUse`、`PostToolUseFailure`、`PermissionDenied`）上是工具名，`Notification` 上是通知类型，`SubagentStart`/`SubagentStop` 上是子 agent 类型（例如 `explore`），`SessionStart` 上是启动来源（`startup`、`resume`，…），`SessionEnd` 上是结束原因，`PreCompact`/`PostCompact` 上是压缩触发（`manual` 或 `auto`），`StopFailure` 上是错误类型（`rate_limit`、`authentication_failed`、`invalid_request`、`server_error`、`max_output_tokens` 或 `unknown`），`StopCancelled` 上是原因（`user_interrupt`、`permission_rejected`、`permission_cancelled`、`max_turns`、`no_progress` 或 `unknown`）。`Stop` 或 `UserPromptSubmit` 上的 matcher 会被忽略并警告（这些事件总会触发）。空的或省略的 matcher 匹配一切。结束思考的提示音应在 `Notification` 上把 `matcher` 设为 `idle_prompt`（任意回合结束，然后持续空闲）；`permission_prompt` 仅在权限 UI 真正在等待时触发。matcher 测试真实工具名；经内部 `use_tool` 调度器路由的 MCP 调用以限定的 `server__tool` 名出现（例如 `linear__save_issue`），因此匹配那个，而不是调度器名。
- **type**：`"command"`（运行脚本或 shell 单行）或 `"http"`（把事件 POST 到 URL）。
- **command**：可执行文件路径（相对 JSON 文件）或内联 shell 命令。
- **timeout**：杀死钩子前的秒数（默认：5，或对 `Stop`/`SubagentStop`/`PostToolUse` 门闩为 600）。所有钩子失败（超时、崩溃、格式错误的输出、缺失必需环境变量）都是失败放行：失败会记入 UI 回看，但工具调用不会被拦截。只有钩子返回的显式 `deny` 决策才会拦截工具调用。

### 工具名别名

在 `matcher` 中，Grok 把 Claude 风格的工具名映射到自己的名称，因此从 Claude 迁移的钩子能正确触发。常见别名包括：

- `Bash` → `run_terminal_command`
- `Read` → `read_file`
- `Edit`、`Write` 和 `MultiEdit` → `search_replace`
- `Grep` → `grep`
- `Glob` 和 `ListDir` → `list_dir`
- `WebSearch` → `web_search`
- `Task` → `spawn_subagent`

matcher 也保留其原始名称，因此 `Bash` 同时匹配 `Bash` 和 `run_terminal_command`。

---

## 钩子如何解析

事件触发时，Grok 分四步解析：

1. **选择匹配的组。** 对该事件，每个 `matcher` 匹配事件字段的 matcher 组都会运行。matcher 在工具事件上测试工具名，在 `Notification` 上测试通知类型，以此类推（见 [关键字段](#key-fields)）。空的或省略的 matcher 匹配一切。
2. **按顺序运行处理程序。** 选中组中的处理程序按配置顺序运行，每个在 stdin 上以 JSON 收到事件，直到有一个返回 `deny`（这会停止链条）。来自不同源（全局、项目、插件、配置）的处理程序会合并，相同的处理程序会去重。每个处理程序看到的都是模型原来的工具输入；`PreToolUse` 的 `updatedInput` 只在所有处理程序结束后才应用，因此一个处理程序看不到另一个的改写（最后一次改写胜出）。
3. **应用决策。** 对 `PreToolUse` 门闩，第一个 `deny` 拦截调用并把原因显示给模型，`updatedInput` 改写工具输入，否则调用继续。对 `Stop` 和 `SubagentStop`，`block` 让 agent 继续工作。对 `PostToolUse`，工具已经跑完，因此什么都拦不住，每个钩子都会运行：`block` 原因和任何 `additionalContext` 会随工具结果交给模型，输出替换会改写模型对该结果的副本。其他每个事件都是被动的：其输出会被记录，但不改变控制流。
4. **失败放行。** 超时、崩溃或发出格式错误输出的处理程序会记入回看，但从不拦截操作。唯一例外是未能通过工具 schema 的 `PreToolUse` `updatedInput`：改写无法安全运行，因此调用被拦截，并作为无效输入错误报告。除此之外，只有显式 `deny` 才会拦截工具调用。

---

## 配置文件中的钩子

钩子也可以直接写在 Grok 配置中，这样团队可以把它们随其余配置一起分发，而不必单独交付 JSON 文件。同一份 `hooks` 对象从三份 TOML 文件读取：

| 文件 | 层级 | 谁设置 |
|------|------|-------------|
| `~/.grok/config.toml` | 用户 | 你 |
| `managed_config.toml`（`$GROK_HOME`、`/etc/grok`） | 托管 / 系统 | 你的组织 |
| `requirements.toml`（用户和系统） | 需求 | 你的组织 |

TOML 在结构上与 JSON 钩子对象相同，因此现有钩子可以直接转写：

```toml
[[hooks.PreToolUse]]
matcher = "Bash|Write|Edit"
hooks = [
  { type = "command", command = "/opt/guard/pretooluse.sh", timeout = 10 },
]
```

每个 matcher 组是一条 `[[hooks.<Event>]]` 条目，带可选的 `matcher` 和内部 `hooks` 处理程序数组。处理程序字段（`type`、`command`、`url`、`timeout`、`env`）和事件名与 [JSON 格式](#the-hook-json-format) 完全相同。

TOML 为内部处理程序提供两种等价写法，解析成同一结构。上面展示的内联表数组是推荐写法：对常见的单处理程序情况最好读。嵌套的表数组形式也被接受：

```toml
[[hooks.PreToolUse]]
matcher = "Bash|Write|Edit"
[[hooks.PreToolUse.hooks]]
type = "command"
command = "/opt/guard/pretooluse.sh"
timeout = 10
```

优先用内联形式，以免为每个处理程序重复 `[[hooks.<Event>.hooks]]` 头。

- **跨层累加。** 每一层的钩子都会运行；较低优先级的层添加钩子，但从不替换另一层的拦截。在多层中定义完全相同的钩子会去重，保留权威最高的副本。
- **来源标签。** 配置钩子在 `/hooks` 中按来源标记（`managed:`、`requirements/user:`、`user:` 等），因此你能看到每一层贡献了哪一个。
- **读取时不展开。** `command` 或 `url` 中字面的 `${VAR}` 原样到达钩子运行器，与 JSON 钩子文件语义一致；运行器执行那一次展开。

---

## 编写钩子脚本

### 输入

事件作为 JSON 发到 **stdin**（例如一次 `PreToolUse` 事件；负载也始终包含 `toolUseId` 和 `toolInputTruncated`）：

```json
{
  "hookEventName": "pre_tool_use",
  "hook_event_name": "PreToolUse",
  "sessionId": "abc-123",
  "cwd": "/Users/you/project",
  "workspaceRoot": "/Users/you/project",
  "permissionMode": "default",
  "toolName": "run_terminal_command",
  "toolInput": { "command": "npm test" },
  "timestamp": "2026-04-14T12:00:00Z"
}
```

每个事件都带相同的公共字段：`hookEventName`、`sessionId`、`cwd`、`workspaceRoot`、`timestamp`、`permissionMode`（`default`、`auto`、`plan` 或 `bypassPermissions`），以及 `promptId`（事件所属回合；会话作用域事件没有），外加上面 `toolName` 这类事件特定字段。`hook_event_name`（snake_case 键）携带 Claude 的 PascalCase 值；`hookEventName`（camelCase 键）携带 grok 的 snake_case 值。

### 输出（拦截钩子）

对 `PreToolUse` 钩子，把 JSON 写到 **stdout**：

- **允许**：`{"decision": "allow"}`
- **拒绝**：`{"decision": "deny", "reason": "Unsafe command detected"}`
- **询问用户**：`{"decision": "ask", "reason": "Confirm this deploy"}`
- **不表态**：`{"decision": "defer"}`
- **改写工具输入**：`{"hookSpecificOutput": {"hookEventName": "PreToolUse", "updatedInput": {"command": "npm test"}}}`
- **告诉模型一些事**：`{"hookSpecificOutput": {"hookEventName": "PreToolUse", "additionalContext": "This repo builds with xb, not cargo"}}`

决策可以写成顶层 `decision`，或写成 `hookSpecificOutput.permissionDecision`。两者都接受 `allow`、`deny`、`ask` 或 `defer`（遗留的 `approve` 和 `block` 拼写也有效），各自带自己的原因字段——`reason` 和 `permissionDecisionReason`。规范的 `permissionDecision` 在存在时决定；顶层 `decision` 仅在它缺失时生效。拒绝或询问消息优先用 `permissionDecisionReason`（若存在），否则用 `reason`。`allow` 只表示「未被拦截」——它不会自动批准一个本会询问用户的调用。该集合之外的决策值是钩子失败，失败放行，除非钩子还以 2 退出，此时拒绝成立，并把错误带在原因里。

`ask` 让调用到达权限提示：任何本会不问就批准的路径——始终批准模式、auto 模式、已保存的「始终允许」授权、安全命令——都不适用，提示会点出你的钩子并显示你的原因。永远不会有第二次提示：本就会询问你的地方，ask 只是给那一次重新贴标签。批准则运行；拒绝则作为普通权限拒绝拦截。以完全始终批准/YOLO 模式运行的客户端（自动回答每一个提示）仍会自动批准该调用，与 Claude Code 的 `bypassPermissions` 一致：ask 覆盖管理器的始终批准、auto、已保存授权和安全命令路径，而不是一刀切批准每个提示的客户端。

`ask` 不能放宽任何东西，因此权限策略拒绝、auto 模式拦截或计划模式仍决定该调用。在 auto 模式中，钩子 `ask` 仍会在提示出现前运行分类器：分类器可以拒绝该调用，但永远不能静默批准钩子询问过的调用。`dontAsk` 模式拒绝它本必须提示的一切，因此在那里 ask 会把本会批准的调用变成拒绝。

只有在设置文件中配置的钩子（命令和 HTTP 钩子）可以 ask、defer 或发送 `additionalContext`：通过 grok-agent-sdk 注册的 `PreToolUse` 钩子可以允许或拒绝，其余会被丢掉——那里的 `ask` 或 `defer` 把调用留给正常权限流，并记为无法识别的决策，`additionalContext` 永远到不了模型。

`updatedInput` 在工具运行前静默替换其输入：模型不会被告知，回看也不会写入任何东西，因此改写的唯一迹象就是改写后的参数本身，若调用到达权限提示，用户会看到。计划模式门闩、权限提示、工具本身，以及之后的 `PostToolUse` 负载都会看到改写后的输入，因此钩子可以规范化或加固一次调用，而不只是允许或拒绝。钩子在计划模式门闩之前运行，因此有副作用的钩子即使计划模式稍后拒绝该调用也会触发。

值必须是 JSON 对象；非对象会使钩子失败。若改写后的输入未能通过工具 schema，调用作为钩子拒绝被拦截——回看注解会点出该钩子——而不是回退到原来的输入。改写可以改变调用的参数，但不能改变运行哪个工具，因此重定向 `use_tool` 调用的改写也会被拦截。非零退出的钩子保留其 `deny`，但失去其 `updatedInput` 和 `additionalContext`。

`deny` 会丢弃任何 `updatedInput`；多个钩子都返回一个时，最后一个胜出。省略 `decision` 同时返回 `updatedInput` 会允许调用并应用改写。

`defer` 既不拦截也不批准调用：调用走正常权限流，就像你的钩子没有回答一样，日志会记一条点出该钩子的警告。它对你发送的其他东西也不生效——`defer` 旁边的 `updatedInput` 或 `additionalContext` 会被忽略并在日志中点名。跨钩子时 `defer` 排在 `ask` 之下，因此一个钩子 defer、另一个 ask 时，grok 会提示。

`additionalContext` 是给模型的备注。它在调用已经运行之后到达——永远不会在之前——随该调用所属批次的结果一起，包在你的 harness 提醒标签中（默认 `<system-reminder>`），并点出写下它的钩子，因此模型能把你的文字和用户的分开。每一个发送备注的钩子都会按钩子运行顺序投递（与 `updatedInput` 不同，那里是最后写入者胜出）。`deny` 会丢掉全部，因为调用从未运行，并在日志中点名这次丢弃。超过 10,000 字符的文本会被裁切，与 `Stop` 反馈的上限相同。

### PostToolUse 输出

`PostToolUse` 在工具完成后运行，因此它拦不住任何东西。它的 stdout 仍会被读取，因为它决定模型接下来看到什么。把 JSON 写到 **stdout**：

```json
{
  "decision": "block",
  "reason": "The diff still contains a debug print",
  "hookSpecificOutput": {
    "hookEventName": "PostToolUse",
    "additionalContext": "This file is generated; edit the template instead",
    "updatedToolOutput": { "type": "Bash", "command": "…", "exit_code": 0, "output_for_prompt": "[redacted]" }
  }
}
```

| 字段 | 效果 |
|-------|--------|
| `decision: "block"` + `reason` | 把 `reason` 紧挨工具结果交给模型。工具自己的输出仍会到达；「block」表示「告诉模型出了问题」，不是「停下这次调用」。 |
| `additionalContext` | 在工具结果旁给模型加一条备注。 |
| `updatedToolOutput` | 替换模型对该结果的副本。通用键；对每个工具都有效。 |
| `updatedMCPToolOutput` | `updatedToolOutput` 的仅 MCP 别名。在内置工具上忽略。 |

- **投递。** 拦截原因和 `additionalContext` 在工具结果之后到达，包在你的 harness 提醒标签中，并点出写下它们的钩子，因此模型可以在同一回合行动。每个钩子的拦截原因和 `additionalContext` 按钩子运行顺序投递，因此一个钩子的发现不会丢掉另一个的。只有替换是最后写入者胜出：两个钩子都返回一个时，最后一个留下，丢弃会在日志中点名。
- **构建 `updatedToolOutput`。** 在内置工具上，它必须携带 grok 对该已运行工具的输出形态，例如 `{"type": "Bash", …}` 这样的带标签对象。拿事件交给你的 `toolResult`，编辑它，再发回去——这正是它被校验所对照的形态。解析失败或解析成另一工具输出的替换会被忽略，原件保留，但钩子的运行会记为 `Failed` 并带原因，因此退出 0 却显示「failed」的钩子是在报告被丢掉的替换，而不是它从未运行。拼错的 `decision`（只有 `"block"` 被认可）以同样方式报告。先检查 `toolResultTruncated`：过大的负载以纯字符串到达钩子，不能原样回传。
- **MCP 工具。** 没有要强制的形态，因此 `updatedToolOutput` 和 `updatedMCPToolOutput` 都未经检查地通过——JSON 字符串原样成为面向模型的文本，其他任何值会被序列化——跨两个键都是最后写入的钩子胜出。
- **上限。** 拦截原因和 `additionalContext` 在 10,000 字符处裁切，与 `Stop` 反馈和 `PreToolUse` 上下文共享的上限。替换得到 64 K 字符。上限按渲染后面向模型的文本计量，并在替换渲染后应用，因此很长的 `updatedToolOutput` 会像字符串一样被裁切，而不是因体积被丢掉。结构化替换仅在不匹配工具自身输出形态时被丢掉。
- **坏掉的钩子。** 非零退出——包括退出 2——保留拦截原因并丢掉其他一切：`additionalContext` 和替换被丢掉，丢弃在日志中点名，与 `PreToolUse` 对 `updatedInput` 的规则相同。拦截是故障安全方向。
- **记录 vs. 模型。** 替换只改写模型的副本。回看、转录和遥测保留原件，因此给密钥打码是对模型隐藏，不是对你隐藏；被改写成成功的失败在记录上仍是真的。替换下不投递图片，因此被替换的截图或 PDF 读取到达模型时只剩你的文字。钩子发送的一切（备注、拦截原因、替换）都会转义，以免它关闭提醒标签、冒充 harness 或用户撰写的指令。
- **输出替换仅限设置文件。** 命令和 HTTP 钩子可以做上述全部。通过 grok-agent-sdk 注册的 `PostToolUse` 钩子可以贡献 `block` 原因和 `additionalContext`，但不能替换工具输出。
- **何时触发。** `PostToolUse` 对每一个实际运行过的工具触发，包括结果是内置逻辑错误（例如 `run_terminal_command` 非零退出）的。派发失败的工具，或返回错误结果的 MCP 工具，改为触发 `PostToolUseFailure`——仅上下文：它可以把 `additionalContext` 喂给模型，但不能拦截或替换输出。钩子继承 600 秒门闩默认（它常跑 linter 或测试）；仅在检查需要更长或更短时显式设置 `timeout`。超时的钩子记为失败，且不贡献任何东西。

### 退出码

| 退出码 | 含义 |
|-----------|---------|
| `0` | 成功 / 允许（对拦截钩子） |
| `2` | 显式拒绝（`PreToolUse`）、以 stderr 为反馈拦截停止（`Stop`/`SubagentStop`），或向模型反馈（`PostToolUse`）。对 `PreToolUse`，当 JSON 没有携带原因时，stderr 的第一行（有上限）成为拒绝原因；`Stop`/`SubagentStop` 和 `PostToolUse` 把完整 stderr 喂给模型，JSON `reason` 优先于它。 |
| 其他 | 失败放行——失败会被记录（形式为 `exit code N: <first stderr line>`），但什么都不会被拦截。对 `PreToolUse`，stdout JSON 中的 `deny` 决策不论退出码都会被认可。对 `Stop`/`SubagentStop`，stdout 上有效的决策 JSON 优先于退出码；仅当 stdout 没有可用 JSON 时，退出码才决定，此时退出 2 以 stderr 为反馈拦截。对 `PostToolUse`，工具已经跑完，因此无论怎样都拦不住；失败仍会被记录，钩子保留其拦截原因，但失去其 `additionalContext` 和输出替换。 |

**`PostToolUse` 退出 2 是行为变更。** 它曾经是普通的已记录失败，什么也不改；现在它把钩子的 stderr 喂给模型。因此写成 `run_checker; exit $?` 的日志钩子，会在检查器以 2 退出时把检查器打印的一切交给模型——`mypy`、`grep`、`pytest` 和 `argparse` 都用退出 2 表示「无匹配」或「用法错误」。给这类钩子显式 `exit 0` 结尾以保持静默。

把人类可读的诊断写到 **stderr**：它是钩子的反馈通道。失败时，stderr 的第一行出现在回看条目和日志中，而不是光秃秃的退出码。

### 停止决策控制

`Stop` 和 `SubagentStop` 钩子在 agent 即将结束本回合时运行，可以让它继续工作（兼容 Claude Code）。把 JSON 写到 **stdout**：

- **拦截停止**：`{"decision": "block", "reason": "The test suite hasn't been run yet"}`。原因作为用户消息反馈给模型，agent 在同一回合再跑一轮。
- **非错误反馈**：`{"hookSpecificOutput": {"hookEventName": "Stop", "additionalContext": "Run the linter before finishing"}}`。同样让 agent 继续工作，但作为钩子反馈而不是钩子错误呈现。
- **强制停止**：`{"continue": false, "stopReason": "Budget exhausted"}`。结束本回合，覆盖任何拦截。
- **允许停止**：以退出 0 且无输出（或任何非 JSON 输出）。

以码 `2` 退出也会拦截停止，以 **stderr** 为反馈。

钩子输入包含 `stopHookActive` 和 `lastAssistantMessage`。当 agent 已经因本回合先前的停止钩子拦截而继续时，`stopHookActive` 为 true；检查它或转录，以免在永远无法解决的条件上拦截。`lastAssistantMessage` 携带 agent 本回合最终响应的文本，因此钩子可以据此行动，而不必解析转录。携带该字段的每个事件都在 32,768 字符处裁切，标记与其他自由文本字段相同：`… [+N chars]`。它远宽于应用到 `errorDetails` 等的 1,000，因为它携带整段回答而不是标签，并且按与工具负载上限相同的尺度设定。同一回合经过 **8 次继续**（拦截或非错误反馈）后门闩被覆盖，回合结束；那一次最终的强制停止不会咨询钩子。计数器按回合：下一条用户提示重新开始，因此长期目标可以跨回合。钩子失败失败放行：agent 正常停止。

`Stop`、`SubagentStop` 和 `PostToolUse` 钩子默认 600 秒超时，因为这些门闩常跑构建或测试套件，而超时的钩子失败放行，因此检查反正拦不住。其他每个事件保持 5 秒默认。门闩需要更多时间时显式设置 `timeout`：`{ "type": "command", "command": "bin/verify.sh", "timeout": 1200 }`。

门闩只对真正完结运行。被中断（Ctrl+C）、拒绝或在回合上限处切断的回合跳过 Stop 门闩，不过在 Stop 钩子已经运行时落下的 Ctrl+C 会把它中途杀死（见下）；API 错误回合触发 `StopFailure`，取消的回合触发 `StopCancelled`。`Esc` 从不取消正在运行的回合。会话结束时还会触发一次单独的 Stop（`reason: "channel_closed"` 或 `"shutdown"`）；其决策输出会被解析但忽略，因为已经没有可继续的回合。按 Stop 触发计数或设门闩的脚本应检查 `reason == "end_turn"`，以免会话结束触发让它偏斜。

`StopFailure` 仅观察（用它记录失败或发送告警；输出和退出码被忽略）。其输入携带 `error`（matcher 测试的分类类型：`rate_limit`、`authentication_failed`、`invalid_request`、`server_error`、`max_output_tokens`，或运行时无法区分的任何东西为 `unknown`；容量错误分类为 `rate_limit`）、`errorDetails`（原始错误细节，若可用，在 1000 字符处裁切；拒绝对话没有该字段，其解释只骑在 `lastAssistantMessage` 上）、`lastAssistantMessage`（对话中显示的渲染错误文本；对该事件它是错误字符串，不是助手输出），以及 `subagentType`（回合在子 agent 内运行时的子 agent 类型）。

`StopCancelled` 也仅观察。**回合未完结就结束时，它代替 `Stop` 运行**，就像 API 错误时 `StopFailure` 代替 `Stop` 运行一样。

一个回合最多报告这三者中的**一个**，下面注明的一个例外除外：已经运行到完成的 `Stop` 钩子，若用户在门闩期间中断，仍可能后跟 `StopCancelled`，因为那时钩子已经被告知回合结束了。每一个运行了模型然后结束、出错或被取消的回合都会报告一个，下列情况除外。

若你的宿主绝不能错过空闲转换，也要监听 `idle_prompt` `Notification`。它覆盖会话仍存活的每一个例外，有一个缺口：唯一活动是跑到完成的 bash 模式命令的会话，既得不到报告也得不到 ping，不过中断一个会两者都有。`SessionEnd` 覆盖拆除。`idle_prompt` ping 大约在会话安定一分钟后触发，至少需要一个回合已经结束，若你先发送另一条消息则取消。

被取消回合的报告在会话命令循环之外派发，因此中断永远不会被你的钩子拖延。报告因此可能在下一回合的 `UserPromptSubmit` **之后**到达，回合结束报告跨路径彼此也不排序。

跟踪忙碌和空闲的脚本应按下面的规则以 `promptId` 为键。grok 每个回合铸造一个，但在 `_meta` 中提供自己的客户端拥有其唯一性，因此把 id 当作不透明，并限定到会话。

每一个回合结束报告都经过一个 worker，因此慢钩子会推迟下一个报告，但从不推迟它所属的回合。保持观察钩子超时较短。

在 `Stop` 钩子运行时中断会把它中途杀死，然后该回合报告 `StopCancelled`：已经开始的 `Stop` 钩子并不承诺回合已完结。`StopFailure` 钩子在回合之外运行，因此中断杀不了它，而且该回合已经报告过，因此不会再跟 `StopCancelled`。

`Stop` 是门闩，因此当停止钩子拦截时，它会为每一轮继续再次触发；只有让回合结束的那一次触发才是报告，在被拦截的 `Stop` 之后以取消或失败结束的回合改为报告那个。被动观察者无法区分继续触发和最终触发（两者的 `stopHookActive` 都为 true），因此仅以 `Stop` 为门闩的 UI 会从第一次继续触发起显示虚假空闲，直到用户的下一条提示，因为没有 `UserPromptSubmit` 标记继续轮。当你也运行拦截门闩时，把 `Stop` 留出状态脚本，改为依赖 `idle_prompt` `Notification`。

有些回合三者都不报告：

- 跑到完成的 bash 模式（`!`）和内置斜杠命令。中断一个仍报告 `user_interrupt`，且没有在前的 `UserPromptSubmit`。
- 取消并发送、回退，或在运行前被移除的排队提示。
- 会话拆除，由会话结束的 `Stop` 和 `SessionEnd` 报告。
- 停止钩子让 agent 一直工作，直到每回合继续上限强制停止的回合。
- 没有停止钩子运行到完成的回合，因为它们全部被禁用、未受信任或失败，或者在子 agent 中，因为它们的 matcher 全部未命中。让回合不报告是有意的：之后的取消或失败仍可以报告它。
- 在其报告仍在构建时被下一回合取代的回合。
- 会话退出时仍在排队的报告：拆除给排队的回合结束钩子半秒，然后丢掉剩下的，并中止仍在运行的任何钩子。
- 已完结、跑过 `Stop`、之后才未能写入磁盘的回合：它报告了 `Stop`，因此不会再跟 `StopFailure`。失败仍会在对话中呈现。

`StopCancelled` 的输入携带：

- `reason`：分类后的原因，也是 matcher 测试的值。`user_interrupt`（Ctrl+C、客户端停止按钮，或客户端 `session/cancel`）、`permission_rejected`（你拒绝了工具调用）、`permission_cancelled`（你关掉了提示）、`max_turns`、`no_progress`（agent 在重复空操作轮后退出），或 `unknown`（运行时无法分类的取消，以及向前兼容的回退）。matcher 只测试该字段，因此想要每一次用户发起停止的钩子应匹配它关心的原因，并从负载读取 `cancelledBy`。以后可能添加新原因，因此把无法识别的值当作 `unknown`。
- `cancelledBy`：中断、被拒绝的工具调用或关掉的提示为 `user`；agent 自己决定的一切（如 `max_turns` 和 `no_progress`）为 `runtime`；`reason` 为 `unknown` 时为 `unknown`，因为运行时无法分类的取消不能声称用户未参与。由 `reason` 派生，因此新原因会自动分类。这里也可能添加值：把你不认识的当作 `unknown`，而不是假设凡不是 `user` 的就是运行时。
- `cancelTrigger`：手势，当客户端点名了一个时，在 64 字符处裁切，因为手势名是一个 token。捆绑的 pager 发送三者之一：`ctrl_c`、`mouse`（屏幕上的停止按钮）或 `dashboard_stop`。它从不发送 `esc`（Esc 不取消回合）。另一个客户端可能发送任意字符串，包括 `esc`，并原样传递。这里的每一个值都分类为 `user_interrupt`，包括碰巧拼成 `shutdown` 这类内部名的，因为客户端要求取消就是用户要求；从负载读取 `cancelledBy`，而不是解析这个字符串。对裸 `session/cancel` 以及每一个运行时发起的原因省略。
- `reasonDetails`：与 `StopFailure` 放在 `errorDetails` 中同类的细节，当运行时有一份时。对被拒绝的工具调用，它是 `<tool>: <why>`。在 1000 字符处裁切，与 `StopFailure` 的 `errorDetails` 一样。
- `lastAssistantMessage`：中断时该回合已提交到对话的任何内容（若有）。最终回答期间的 Ctrl+C 留下最后提交的文本，若回合从未提交任何内容则为空。裁切方式与 `Stop` 和 `StopFailure` 上的同一字段相同。
- `subagentType`：回合在子 agent 内运行时的子 agent 类型，因此钩子能区分嵌套 agent 的停止和会话的停止。主会话中没有。

信封的 `timestamp` 在钩子派发时盖章，不是回合结束时。三个回合结束事件排在一个 worker 后面，因此等在慢钩子前面的报告会带上比它所描述时刻更晚的时间戳。用 `promptId` 关联，不要用时钟。

`StopCancelled` 不能拦截：回合已经结束，让钩子重新打开用户故意停下的回合会跟用户对着干。想让 agent 继续工作时用 `Stop`。

「取消并发送」（在回合运行时键入新消息）**不**触发 `StopCancelled`，因为回合是被替换而不是被停止，且 agent 仍在忙碌。在子 agent 内，`user_interrupt` 也不触发：它跟随父会话的取消，会话级信号才有用。子 agent 自己的 `max_turns`、`no_progress` 或被拒绝的权限会触发。matcher 只测试 `reason`，因此报告会话是否空闲的脚本应在存在 `subagentType` 时提前退出。

完整的忙碌和空闲指示需要五次注册。`UserPromptSubmit` 标记会话忙碌；
`Stop`、`StopFailure` 和 `StopCancelled` 按回合如何结束来安定它；`idle_prompt`
`Notification` 是三者都不报告的那些回合的后盾。只注册
`StopCancelled` 会让宿主在每一个正常回合之后保持忙碌。

```json
{
  "hooks": {
    "UserPromptSubmit": [{ "hooks": [{ "type": "command", "command": "bin/turn-started.sh" }] }],
    "Stop": [{ "hooks": [{ "type": "command", "command": "bin/turn-ended.sh", "timeout": 10 }] }],
    "StopFailure": [{ "hooks": [{ "type": "command", "command": "bin/turn-ended.sh" }] }],
    "StopCancelled": [{ "hooks": [{ "type": "command", "command": "bin/turn-ended.sh" }] }],
    "Notification": [
      { "matcher": "idle_prompt", "hooks": [{ "type": "command", "command": "bin/turn-ended.sh" }] }
    ]
  }
}
```

两个脚本必须做对的事：

- **跟踪最新的 `promptId`，忽略更旧回合的报告。** 被取消回合的报告
  在命令循环之外派发，因此可能在下一回合的 `UserPromptSubmit` 之后到达。
- **没有 `promptId` 时无条件安定。** 那是 grok 在报告会话
  而不是回合：`idle_prompt` ping 和会话结束的 `Stop`。正是它让
  后盾对回退或被取代的回合生效，那些什么也不报告。
- **把你从未见过开始的 `promptId` 当作空闲。** 被中断的 bash 模式回合会报告，且
  没有在前的 `UserPromptSubmit`。
- **存在 `subagentType` 时提前退出。** 子 agent 的停止不是会话的。
- **在把回合记为已处理之前先安定宿主，** 这样中途被杀的钩子会让
  回合可纠正。先重读那条记录，这样你只清除自己记录过的回合。
- **只做本地写入。** 拆除给整队回合结束报告半秒，
  然后每个 `SessionEnd` 钩子受自己的超时约束（默认 1.5s；设置
  `GROK_SESSION_END_HOOKS_TIMEOUT_MS`，单位毫秒，以更改该默认，上限 60s）。

`Stop` 是门闩，因此该条目跑在回合的关键路径上：保持它快，给它一个 `timeout`，
并以 0 退出，因为退出 2 会拦截停止并让 agent 继续工作。若你也
运行拦截的 `Stop` 门闩，就把 `Stop` 留出，因为继续触发会在 agent 仍在
进行时安定宿主；改为注册 `SessionEnd`，它是唯一能安定在 ping 之前
退出的会话的东西。

两个脚本也在子 agent 自己的会话内运行。任一脚本读取的每个事件在那里都携带
`subagentType`，在主会话中省略，因此 `[ -n "$subagentType" ] && exit 0`
把子会话从两半都过滤掉。这对后台子 agent 最重要，它比
父回合活得更久，否则会在父会话已空闲后仍让宿主保持忙碌。

`Stop` 输入还携带 `backgroundTasks` 和 `sessionCrons`，因此钩子可以区分「会话已完成」和「会话暂停，等待后台工作再把它唤醒」。两者在没有进行中或已调度的工作时都是空数组。每个 `backgroundTasks` 条目描述一个进行中的任务：`id`、`type`（`shell`、`monitor` 或 `subagent`）、`status`，以及（取决于类型）`command`（仅 shell 任务）、`description`（monitor 监视的命令行，或子 agent 的任务描述）和 `agentType`（子 agent）。每个 `sessionCrons` 条目描述一次已调度的唤醒（`scheduler_create` 或 `/loop`）：`id`、`schedule`、`recurring` 和 `prompt`。`schedule` 值是人类可读的间隔，例如 `every 5 minutes`；grok 调度是间隔，不是 cron 表达式。自由文本条目标在 1000 字符处封顶，带串内 `… [+N chars]` 标记。

在子 agent 内，门闩以 `SubagentStop` 触发（agent frontmatter 的 `Stop` 钩子会自动重映射）。`Stop` 钩子只门闩主 agent。

`SubagentStop` 每个子 agent 触发一次，在子 agent 自己的回合结束时，与 Claude Code 一致。其输入携带 `phase` 字段（目前始终为 `"gate"`），为向前兼容保留。

**移植 Claude Code 停止钩子**：输出词汇（`decision`、`reason`、`continue`、`stopReason`、`additionalContext`）原样可用。对照此列表查看与 Claude 不一致之处：

- **camelCase 输入**：grok 的 stdin 信封全程使用 camelCase 键，而 Claude 使用 snake_case。读取 `.stop_hook_active` 或 `.background_tasks[].agent_type` 的脚本必须改成 `.stopHookActive` 和 `.backgroundTasks[].agentType`（snake_case 键 `hook_event_name` 携带 Claude 的 PascalCase 值，例如 `"Stop"`；camelCase 键 `hookEventName` 携带 grok 的 snake_case 值，例如 `"stop"`）。通过 grok-agent-sdk 注册的钩子会把顶层键以及 `backgroundTasks`/`sessionCrons` 条目标都转成 snake_case，因此线上的 `.backgroundTasks[].agentType` 在 SDK 中读作 `.background_tasks[].agent_type`。
- **`toolResult` 字段**：`PostToolUse` 工具输出是 `toolResult`（SDK：`tool_result`）；grok 也发出复制 `toolResult` 的 `tool_response` snake 别名，因此读取 Claude 的 `.tool_response` 的钩子原样可用。
- **`updatedToolOutput` 在内置工具上携带 grok 自己的输出形态**：对内置工具的 `PostToolUse` 替换会对照 grok 序列化该工具输出的形态校验——该事件 `toolResult` 中的带标签对象——因此按另一运行时字段名写的会解析成错误形态并被忽略。在 MCP 工具上没有要强制的形态，因此 `updatedToolOutput` 像其 `updatedMCPToolOutput` 别名一样通过。见 [PostToolUse 输出](#posttooluse-output)。
- **会话结束触发**：会话结束时多一次仅观察的 Stop；按 `reason == "end_turn"` 过滤（见上）。
- **间隔调度**：`sessionCrons[].schedule` 是人类可读间隔，从来不是 cron 表达式。
- **任务类型**：`backgroundTasks[].type` 只有 `shell`、`monitor` 或 `subagent`；Claude 的其他标签（`workflow`、`teammate`，…）不会发出。
- **StopFailure 类别**：grok 发出六种（`rate_limit`、`authentication_failed`、`invalid_request`、`server_error`、`max_output_tokens`、`unknown`）。容量错误（503/529）分类为 `rate_limit`。匹配 grok 不发出的错误类别的 matcher 永远不会触发。
- **默认超时**：grok 把观察钩子默认设为 5 秒，比多数更短。对做实际工作的导入钩子显式设置 `timeout`。
- **`UserPromptSubmit` 会拦截，有一个缺口**：退出 2 和 `decision: "block"` 像 Claude 一样拒绝提示，被拦截的提示永远不会进入对话历史——但放行钩子的 stdout / `additionalContext` 会被丢弃，而不是加为上下文。
- **`StopCancelled` 是 grok 特有的**：使用它的配置不能移植到没有中断钩子的运行时。
- **`idle_prompt` 在任意回合结束时触发**：grok 在中断或出错的回合之后也会触发，不只是完结的，因为它报告的是状态而不是结果。其 `message` 是展示文本，可能随版本变化，因此改为匹配 `notificationType`。
- **子 agent 身份是 `subagentType`，不是 `agent_type`**：grok 把它放在可以在子 agent 内触发的事件的负载中，与自己的 `SubagentStart`/`SubagentStop` 一致，而不是放在公共字段中。
- **permission_mode 值**：grok 发出 `default`、`auto`、`plan` 或 `bypassPermissions`。Claude 的 `acceptEdits`/`dontAsk` 没有 grok 等价项（grok 的 `auto` 最接近），因此 `permission_mode === "acceptEdits"` 这类检查永远匹配不上。
- **客户端（SDK）门闩超时**：SDK `Stop`/`SubagentStop` 门闩默认 600 秒，与文件钩子相同；`PreToolUse` 客户端门闩默认 30 秒（交互热路径）。两者都可以通过 `timeoutS` 按 matcher 组覆盖，上限 600。
- **`/goal`**：grok 的目标循环是在停止门闩之前运行的单独功能；它不是提示类型的 Stop 钩子。

一个脚本里的完整继续工作策略：

```bash
#!/bin/bash
input=$(cat)
# Gate only genuine turn ends, not the session-end observe fire.
if [ "$(echo "$input" | jq -r '.reason')" != "end_turn" ]; then exit 0; fi
if ! bin/verify.sh >/dev/null 2>&1; then
  echo '{"decision": "block", "reason": "verify.sh failed; fix the failures before finishing"}'
fi
```

注册为 `{ "type": "command", "command": "bin/stop-gate.sh", "timeout": 300 }`，`timeout` 按校验步骤的需要设定。钩子在每次继续后再次触发，内置上限在 8 次后结束回合；检查 `stopHookActive`，以便在 agent 显然无法据此行动的反馈上更早放弃。

### 被动钩子

对 `SessionStart` 或 `Notification` 这类事件，stdout 被忽略。成功时以 0 退出即可。例外是 `PreToolUse`（见 [输出（拦截钩子）](#output-blocking-hooks)）、`Stop`/`SubagentStop`（见 [停止决策控制](#stop-decision-control)），以及 `PostToolUse`，其 stdout 会被读取，尽管它拦不住任何东西（见 [PostToolUse 输出](#posttooluse-output)）。

### 环境变量

Grok 在每一个钩子进程上设置若干环境变量。编写感知上下文或感知插件的钩子脚本时很有用。

#### 运行器注入的变量（始终可用）

这些变量由钩子运行器为**每一个**钩子设置：

| 变量              | 说明 |
|-----------------------|-------------|
| `GROK_HOOK_EVENT`     | 触发该钩子的事件名（例如 `pre_tool_use`、`session_start`、`post_tool_use`、`session_end`、`stop`、`notification`）。 |
| `GROK_HOOK_NAME`      | 该特定钩子的配置名（插件提供的钩子包含插件前缀）。 |
| `GROK_SESSION_ID`     | 当前 Grok 会话的唯一标识符。 |
| `GROK_WORKSPACE_ROOT` | 当前工作区根的绝对路径。 |
| `CLAUDE_PROJECT_DIR`  | 工作区根的绝对路径。`GROK_WORKSPACE_ROOT` 的 Claude Code 兼容别名，为每一个钩子设置。 |

这些变量是**保留**的。你试图通过钩子 JSON 的 `env` 字段为它们设置的任何值都会在加载时被剥掉（记一条警告），运行器始终在生成时注入真实值。

#### 插件钩子变量

当钩子来自插件时，Grok 额外注入以下变量：

| 变量             | 说明 |
|----------------------|-------------|
| `GROK_PLUGIN_ROOT`   | 插件已安装目录的绝对路径。 |
| `GROK_PLUGIN_DATA`   | 插件可写数据目录的绝对路径（用于存储插件状态、缓存等）。 |

这些值由插件系统提供。对四个与插件相关的键（`GROK_PLUGIN_ROOT`、`GROK_PLUGIN_DATA` 及其 Claude 别名），插件适配器确保官方插件值始终胜过钩子 `env` 映射中任何用户声明的值。

#### 用户定义的环境变量

你可以用 `env` 字段为单个钩子处理程序提供额外环境变量：

```json
{
  "type": "command",
  "command": "bin/my-hook.sh",
  "env": {
    "MY_SECRET": "value",
    "LOG_LEVEL": "debug"
  }
}
```

这些变量会传给钩子进程，但不能覆盖上面列出的保留运行器或插件变量。

#### 在 `command` 和 `url` 字段中使用变量

`command` 和 `url` 都支持 `${VAR}` 和 `$VAR` 展开。在 Windows PowerShell 上，已知的 `$VAR` 引用会改写成 `$env:VAR`，以便读取子环境。加载时 vs 运行时展开、`env` 映射查找顺序，以及参数展开修饰符（例如 `${VAR:-default}`）见自定义钩子参考。

---

## HTTP 钩子

不跑本地脚本，改为调用远程端点：

```json
{ "type": "http", "url": "https://hooks.example.com/grok-event", "timeout": 15 }
```

完整事件信封作为 JSON POST。

---

## 在 TUI 中管理钩子

### 钩子标签页

在非 VS Code 系列终端按 `Ctrl+L` 打开扩展弹窗（插件标签页），或运行 `/hooks`（任意终端；VS Code 系列必需，因为 `Ctrl+L` 是插话）以在钩子标签页打开它。在 **钩子** 标签页：

| 按键 | 动作 |
|-----|--------|
| `r` | 从磁盘重新加载所有钩子 |
| `a` | 按路径添加自定义钩子 |
| `x` | 移除选中的钩子源（要求确认；按小写 `y` 确认） |
| `Space` | 启用或禁用选中的钩子 |
| `f` | 循环状态过滤器（全部 / 已启用 / 已禁用） |

钩子按来源分组：**全局**、**项目**、**插件** 和 **自定义**。

每个钩子显示：
- 它触发的 **事件**
- 运行的 **命令** 或 **URL**
- **超时** 时长
- **状态**：已启用或 `[disabled]`

### 斜杠命令

```
/hooks-list           # 显示本会话加载的钩子
/hooks-trust          # 信任本项目以执行钩子
/hooks-add <path>     # 添加自定义钩子文件或目录
/hooks-remove <path>  # 移除自定义钩子
/hooks-untrust        # 撤销对本项目的信任
```

在 TUI pager 中，各个 `/hooks-*` 命令不出现在斜杠命令列表里。`/hooks` 弹窗覆盖列出、添加、移除以及启用或禁用钩子；项目信任通过 `/hooks-trust`（或弹窗的 Trust 操作）管理，它写入上面描述的统一文件夹信任存储。

### 按钩子启用/禁用

在钩子标签页按 `Space`，即可在运行时启用或禁用单个钩子。变更立即生效，无需重启会话。

### 会话中途重新加载

在钩子标签页按 `r`，从磁盘重新加载所有钩子。Grok 重新读取每一个钩子源，因此会拾取你在会话期间对钩子文件所做的变更。

---

## 状态行与回看中的钩子

钩子保持安静，除非它们拖住回合或改变其走向：

- 当回合阻塞在钩子批次上时（工具前的 `PreToolUse` 门闩、`UserPromptSubmit` 门闩、`Stop` 门闩），批次大约跑了 300 ms 后，状态行显示 `Running pre_tool_use hook…`（或 `Running 3 stop hooks…`）。计时器从批次开始时算起，因此慢钩子会显示完整等待；快的根本不显示。
- 运行并放行的钩子不留痕迹。它的 stdout 不显示。
- 拒绝工具调用、拦截提示，或停止或继续 agent 的钩子会得到一行带原因的注解。来自 `~/.grok`、项目和插件文件的钩子会被点名；来自托管配置的钩子读作「a managed policy hook」。
- 失败的钩子（非零退出、超时、崩溃、格式错误的输出）得到一行：`<event> hook (<name>) failed, ignored: <reason>`，其中原因是带第一行 stderr 的退出码，或超时。「Ignored」是字面意思：失败是失败放行，因此工具调用或回合继续，就像钩子已经允许一样。

拒绝和失败行带与工具行相同的圆点，因此它们读作上方工具调用的一部分。

这些行仅在插件 UI 启用时出现（默认）。

---

## 示例：安全 Shell 护栏

拦截危险的 shell 命令：

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          { "type": "command", "command": "bin/safe-shell.sh", "timeout": 5 }
        ]
      }
    ]
  }
}
```

其中 `bin/safe-shell.sh`：

```bash
#!/bin/sh
INPUT=$(cat)
CMD=$(echo "$INPUT" | jq -r '.toolInput.command // empty')

# Block destructive patterns
if echo "$CMD" | grep -qE '(rm -rf /|mkfs|dd if=|:(){ :|& };:)'; then
  echo '{"decision": "deny", "reason": "Blocked potentially destructive command"}' 
  exit 2
fi

echo '{"decision": "allow"}'
```

---

## 安全说明

- 全局钩子（`~/.grok/hooks/`）以你的用户权限运行；像对待 shell 脚本一样对待它们。
- 项目钩子需要文件夹信任（`/hooks-trust` 或 `--trust`，与仓库本地 MCP/LSP 同一道门闩），以防止恶意仓库的供应链攻击。
- HTTP 钩子发送会话数据；只使用受信任的端点。
- `PostToolUse` 钩子决定模型为该工具调用读到什么——它可以添加指令或直接替换输出——因此像信任 `PreToolUse` 门闩一样信任它。回看和转录保留真实输出，因此替换对你始终可见。

---

## 最佳实践

1. **保持钩子快**：长时间运行的钩子会阻塞 UI。尽可能使用后台进程（`&`）或异步。
2. **用显式 `deny` 拦截**：钩子在任何错误上都失败放行，因此崩溃的钩子不会拦截工具。要强制策略，钩子必须运行到完成，并在 stdout 上发出 `{"decision":"deny","reason":"..."}`。始终在脚本内处理错误，以便它能返回显式决策。
3. **使用绝对路径或相对钩子文件的路径**：JSON 文件旁 `bin/` 中的脚本可移植。
4. **用弹窗测试**：按 `Ctrl+L`（非 VS Code 系列）或运行 `/hooks`，在依赖钩子之前确认它们已加载并匹配。
5. **把项目钩子纳入版本控制**：提交 `.grok/hooks/`（但永远不要提交密钥）。

---

## 故障排除

- **钩子没在运行？** 在非 VS Code 系列按 `Ctrl+L`（或在任意处运行 `/hooks`），查看它是否已加载并匹配。
- **项目钩子被忽略？** 文件夹可能未受信任。运行 `/hooks-trust`（或用 `--trust` 重新启动）。
- **找不到脚本？** 检查路径是相对 `.json` 文件的，并且可执行（`chmod +x`）。
- **看到错误？** 用 `RUST_LOG=debug GROK_LOG_FILE=/tmp/grok.log grok` 启动以捕获日志，然后检查 `/tmp/grok.log`。
