# 无头模式与脚本

无头模式从命令行以非交互方式运行 Grok。它接受一条提示，带着完整工具访问执行，并返回结果。用它自动化任务、编写工作流脚本、构建集成，以及以编程方式解析输出。

---

## 基本用法

以非交互方式传入提示会触发无头模式。最常见的方式是 `-p` 标志（`--single` 的短形式）；`--prompt-json` 和 `--prompt-file` 也会触发：

```bash
grok -p "Your prompt here"
```

Grok 处理提示，运行任何必要的工具，并把结果打印到 stdout。响应完成后进程退出。

---

## 命令行选项

| 标志                    | 说明                                           |
| ----------------------- | ----------------------------------------------------- |
| `-p, --single <PROMPT>` | 要发送的提示（或使用 `--prompt-json` / `--prompt-file`） |
| `-m, --model <MODEL>`   | 要使用的模型（例如 `grok-4.6`）              |
| `-s, --session-id <ID>` | 用此 **UUID** 创建**新**会话（在目标会话目录下若不是有效 UUID 或已被占用则报错；不恢复，请用 `-r`/`-c`） |
| `--fork-session`        | 与 `-r`/`-c` 一起使用时，分叉到新的会话 ID，而不是追加到原来的 |
| `-r, --resume <ID_OR_TITLE>` | 按 ID 恢复现有会话，或按当前目录的标题恢复，忽略字母大小写（重复项中唯一被手动重命名的匹配胜出；其余重复项报错并给出其 ID；形如 UUID 的值始终走 ID 路径；脚本应优先使用 ID） |
| `-c, --continue`        | 继续当前目录中最近的会话  |
| `--cwd <PATH>`          | 设置工作目录                                 |
| `--output-format <FMT>` | 输出格式：`plain`、`json`、`streaming-json`、`streaming-messages-json` |
| `--include-partial-messages` | 发出原始 `stream_event` 增量。只影响 `--output-format streaming-messages-json`；否则忽略（并警告）。 |
| `--yolo`                | 自动批准所有工具执行                      |
| `--rules <TEXT>`        | 系统提示的自定义规则                    |
| `--tools <TOOLS>`       | 内置工具的允许列表（逗号分隔）。MCP 元工具除非被拒绝否则仍可用。仅无头。 |
| `--disallowed-tools <TOOLS>` | 要移除的内置工具拒绝列表（逗号分隔）。支持 `Agent` 条目。仅无头。 |
| `--max-turns <N>`       | 停止前的最大 agent 回合数。仅无头。 |
| `--reasoning-effort` / `--effort <LEVEL>` | 推理模型的推理力度。规范级别：`none`、`minimal`、`low`、`medium`、`high`、`xhigh`、`max`（每一级都是独立档位；模型只接受其菜单宣传的级别）。也接受按模型的菜单选项 id（例如 `deep` → 映射后的线上值），与 `/effort` 相同。TUI 和无头都可用。 |
| `--permission-mode <MODE>` | 权限模式。`bypassPermissions` 启用始终批准（见 [权限与安全](22-permissions-and-safety.md#permission-modes)）；默认拒绝请用 `.claude/settings.json` 中的 `defaultMode`。 |
| `--allow <RULE>`        | 带 glob 模式的权限允许规则（可重复）。TUI 和无头都可用。 |
| `--deny <RULE>`         | 带 glob 模式的权限拒绝规则（可重复）。TUI 和无头都可用。 |
| `--prompt-json <JSON>`  | 以 JSON 内容块作为提示                         |
| `--prompt-file <PATH>`  | 从文件读取提示                                    |
| `--verbatim`            | 原样发送提示                          |
| `--no-auto-update`      | 为本会话禁用更新检查                |
| `--sandbox <PROFILE>`   | 文件系统/网络访问的沙箱配置档         |

> **注意：** `--tools`、`--disallowed-tools`、`--max-turns` 和 `--agents` 是仅无头标志。若在交互式 TUI 中使用，会打印警告并忽略该标志。`--reasoning-effort`/`--effort`、`--permission-mode`、`--allow` 和 `--deny` 在两种模式都可用。更多标志（agent 和工作树）见 [额外的无头标志](#additional-headless-flags)。

### 工具过滤

用 `--tools` 把 agent 限制到显式的工具集合（允许列表），或用 `--disallowed-tools` 从默认集合中移除特定工具（拒绝列表）。两者都接受逗号分隔的工具名。

工具名是内部工具 ID（例如 shell 工具是 `run_terminal_cmd`，不是 `bash`）。

```bash
# 只允许只读工具
grok -p "Explain this codebase" --tools "read_file,grep,list_dir"

# 移除网络访问和文件编辑
grok -p "Review this code" --disallowed-tools "web_search,web_fetch,search_replace"

# 移除 shell 访问
grok -p "Review this code" --disallowed-tools "run_terminal_cmd"
```

`--disallowed-tools` 也支持特殊的 `Agent` 条目，以控制子 agent 生成：

| 条目                  | 效果                                  |
| ---------------------- | --------------------------------------- |
| `Agent`                | 拦截所有子 agent 生成             |
| `Agent(explore)`       | 只拦截 `explore` 子 agent 类型  |
| `Agent(explore, plan)` | 拦截多个特定类型           |

```bash
# 阻止 agent 生成任何子 agent
grok -p "Fix this bug" --disallowed-tools "Agent"

# 只拦截 explore 子 agent
grok -p "Refactor this module" --disallowed-tools "Agent(explore)"
```

`--tools` 保留所选 agent 配置档的注入策略：库存配置档在应用允许列表之前注入已启用的可选工具，精选配置档保持严格。最终工具集保留请求的工具以及始终开启的 MCP 元工具。两个标志都存在时，`--disallowed-tools` 胜出。

### 权限规则（`--allow` / `--deny`）

权限规则控制特定工具调用是自动批准、拒绝，还是需要用户确认。与完全移除工具的 `--disallowed-tools` 不同，权限规则让工具保持可用，但对其执行设门闩。

规则使用 `ToolPrefix(glob_pattern)` 语法：

| 前缀        | 控制什么                   |
| ------------- | ---------------------------------- |
| `Bash(...)`   | Shell 命令执行            |
| `Edit(...)`   | 文件编辑（路径 glob）           |
| `Write(...)`  | 文件写入（路径 glob）           |
| `Read(...)`   | 文件读取（路径 glob）           |
| `Grep(...)`   | 搜索操作（路径 glob）      |
| `WebFetch(...)` | URL 抓取（glob 或 `domain:host`） |
| `MCPTool(...)` | MCP 工具调用              |

对路径规则（`Read`、`Edit`、`Write`、`Grep`），`*` 是单层通配符，`**` 是递归。对 `Bash` 规则，`*` 匹配包括空格在内的任意字符。不带括号的裸前缀匹配该类型的所有调用，`Bash(cmd:*)` 等价于对 `cmd` 的前缀匹配。完整匹配语义见 [22-permissions-and-safety.md](22-permissions-and-safety.md#rule-matching-reference)。

```bash
# 拒绝匹配 "rm*" 的 shell 命令
grok -p "Clean up this project" --deny "Bash(rm*)"

# 允许 npm 命令，拒绝 sudo
grok -p "Set up the project" --allow "Bash(npm*)" --deny "Bash(sudo*)"

# 允许所有 bash 命令（不问就自动批准）
grok -p "Build the project" --allow "Bash"
```

`--allow` 和 `--deny` 可以重复。拒绝规则优先于允许规则。

---

## 输出格式

无头模式支持四种输出格式，用 `--output-format` 选择。

### plain（默认）

人类可读文本，适合直接显示或管道：

```
Here's a summary of the codebase...
```

### json

响应完成后发出的单个 JSON 对象：响应文本、
停止原因、会话 ID、请求 ID（存在推理时还有 `thought`）。
当提示到达模型时，同一对象还携带花费字段
（`usage`、`num_turns`、`modelUsage`、费用）。`stopReason` 是 snake_case
的 ACP/Messages 标记（`end_turn`、`max_tokens`，…）。

```json
{
  "text": "Here's a summary of the codebase...",
  "stopReason": "end_turn",
  "sessionId": "abc123",
  "requestId": "xyz789",
  "num_turns": 7,
  "usage": {
    "input_tokens": 7210,
    "cache_read_input_tokens": 41000,
    "cache_creation_input_tokens": 0,
    "output_tokens": 1893,
    "reasoning_tokens": 412,
    "total_tokens": 50103
  },
  "modelUsage": {
    "grok-4.6": {
      "inputTokens": 7210,
      "outputTokens": 1893,
      "cacheReadInputTokens": 41000,
      "modelCalls": 7,
      "costUSD": 0.01268905
    }
  },
  "total_cost_usd": 0.01268905,
  "total_cost_usd_ticks": 126890500
}
```

用量说明：

- `usage` 汇总该提示的 token，包括在回合结束前完成的
  子 agent（也在它们自己的 `modelUsage` 键下）。压缩和
  其他旁路模型调用排除在外。
- **Token 字段策略（无头结果 / `end` / 错误花费）：**
  - `usage.input_tokens` 和 `modelUsage.*.inputTokens` 是**仅未缓存**的。
  - `cache_read_input_tokens` / `cacheReadInputTokens` 是缓存命中。
  - `total_tokens` 是完整输入 + 输出（包含两个缓存桶）：
    `total_tokens = input_tokens + cache_read_input_tokens + cache_creation_input_tokens + output_tokens`。
  - ACP `_meta.usage.inputTokens`（PromptUsage）仍是**完整**提示
    合计；只有无头投影器减去缓存。花费自动化请优先用无头字段。
- `num_turns` 统计提示账本上记录的主 agent 模型轮次
  （报告了用量的工具循环轮）。子 agent 采样器调用不
  增加它。按模型的调用计数（包括子 agent）留在
  `modelUsage.*.modelCalls`。这与 `--max-turns` 是同一计数器家族，
  当轮次缺少用量或撞上门闩时，不保证精确相等。
- `total_cost_usd` 仅在服务器报告了**完整**费用时出现。
  缺失表示未报告或不完整，从来不是免费。今天费用会为
  API 密钥流量盖章；池/OAuth 路径常常省略，直到服务器
  盖上费用。当部分调用缺少费用时，`cost_is_partial` 为 true，并且
  **所有**费用浮点数都被省略（`total_cost_usd` 以及每一个
  `modelUsage.*.costUSD`），以免消费者把模型行加总成虚假的
  完整账单。
- `total_cost_usd_ticks` 是同一值的精确整数 tick
  （1 USD = 10^10 ticks），并在相同条件下出现。用它做
  账单对账：按调用加总 tick 与服务器的
  用量导出精确匹配，浮点美元无法保证这一点。
- 当子 agent 用量无法应用、嵌套子 agent 用量不完整，
  或成功路径排空超时（回合任务上最多 120s）时，
  `usage_is_incomplete` 为 true，费用浮点数以同样方式省略
  （token 总计可能少计子 agent）。取消快照会不经过那次长
  排空，并在子 agent 仍存活时标为不完整。不完整且
  没有已记录 token 时只发出 `usage_is_incomplete`（没有全零 `usage` 对象）。
- 从未到达模型的提示省略花费字段。

`sessionId` 字段便于稍后恢复对话。

失败时，Grok 发出错误对象（进程非零退出）。提示级
失败在记录了用量时也可能包含冻结的花费字段：

```json
{"type":"error","message":"Couldn't start session: ..."}
```

### streaming-json

换行分隔的 JSON，每行一个带 `type` 标签的对象，派生自 agent 的 ACP 会话更新。叶子字段名（`toolCallId`、`kind`、`rawInput`、`rawOutput`）遵循 ACP；`toolName` 和 `usage` 行是 xAI 添加项。按 `type` 切换来消费。

```json
{"type":"thought","data":"Analyzing the directory structure..."}
{"type":"tool_call","toolCallId":"call_1","title":"Read","kind":"read","status":"in_progress","toolName":"read_file","rawInput":{"path":"src/main.rs"},"content":[],"locations":[]}
{"type":"tool_call_update","toolCallId":"call_1","status":"completed","content":[],"rawOutput":{"lines":42},"locations":[]}
{"type":"text","data":"Here's a summary"}
{"type":"usage","messageId":"resp_1","stopReason":"end_turn","usage":{"input_tokens":812,"output_tokens":45,"cache_read_input_tokens":0,"cache_creation_input_tokens":0,"reasoning_tokens":0},"signature":"..."}
{"type":"end","stopReason":"end_turn","sessionId":"abc123","requestId":"xyz789","usage":{...},"num_turns":7,"modelUsage":{...}}
```

事件类型：

| 类型               | 说明                                                                                  |
| ------------------ | ------------------------------------------------------------------------------------------- |
| `text`             | agent 响应文本的一块                                                          |
| `thought`          | 内部推理（思考 token）                                                          |
| `tool_call`        | agent 开始的一次工具调用（`toolCallId`、`toolName`、`kind`、`status`、`rawInput`、`content`、`locations`） |
| `tool_call_update` | 工具调用的进度或结果（`status`、`rawOutput`、`content`、`locations`）            |
| `usage`            | 每次响应边界（`messageId`、`stopReason`、`usage`、`signature`），每个模型响应一条 |
| `plan`             | agent 当前的计划（`entries`）                                                          |
| `available_commands` | 工具和斜杠命令列表（`tools`、`commands`）                                          |
| `end`              | 带元数据和花费字段（若可用）的最终事件                                    |
| `error`            | 发生错误（携带 `message`，以及花费字段若有）                               |

`end` 始终是最后一个事件。`end` 上的花费字段与 json 对象
形态匹配（snake_case 的未缓存 `input_tokens`、安全的费用浮点数）。`end.stopReason`
是 snake_case 的回合停止原因（`end_turn`、`max_tokens`、
`max_turn_requests`、`refusal`、`cancelled`）；逐字的每次响应提供商
原因（例如 `tool_use`、`pause_turn`）在 `usage` 行的 `stopReason` 上。
每次响应的 `message_id`/`stopReason`/`signature` 在 Messages
API 后端上填充；其他后端报告它们携带的内容。

Grok 也可能发出 `max_turns_reached` 和 `auto_compact_*` 事件；把该列表视为非穷尽，并按 `type` 切换。

### streaming-messages-json

换行分隔的 JSON，采用 Messages API `stream-json` 线上格式。承载数据的表面与 Messages 形态完全匹配。这包括 `assistant`/`user` 消息体、`usage`、`tool_use`/`tool_result`、内联网络搜索、`stop_reason`，以及 `--include-partial-messages` 事件框架。重建消息、读取花费或检测错误的消费者无需改动即可工作。

`system`/`init` 和终端 `result` 行携带元数据。Grok 发出它有真实数据的字段，并省略它填不上的纯占位字段，而不是填零。因此这两行可能无法通过严格的 `init`/`result` schema 校验。各个字段列在下面。在把任何一个字段当作权威之前，先读保真说明。若要干净的、无占位形态的 xAI 原生流，使用 `streaming-json`。

流以 `system`/`init` 行开始，然后是 `message.content[]` 中持有 `text`、`thinking` 和 `tool_use` 块的 `assistant` 消息、携带 `tool_result` 块的 `user` 消息，以及终端 `result`：

```json
{"type":"system","subtype":"init","session_id":"abc123","apiKeySource":"user","model":"grok-4.6","cwd":"/repo","permissionMode":"default","tools":["read_file","bash"],"slash_commands":["review"],"mcp_servers":[{"name":"linear","status":"connected"}],"skills":[],"uuid":"..."}
{"type":"assistant","message":{"id":"msg_0","type":"message","role":"assistant","model":"grok-4.6","content":[{"type":"text","text":"Let me read the file."},{"type":"tool_use","id":"call_1","name":"read_file","input":{"path":"src/main.rs"}}],"stop_reason":"tool_use","stop_sequence":null,"usage":{...}},"parent_tool_use_id":null,"session_id":"abc123","uuid":"..."}
{"type":"user","message":{"role":"user","content":[{"type":"tool_result","tool_use_id":"call_1","content":"fn main() {}","is_error":false}]},"parent_tool_use_id":null,"session_id":"abc123","uuid":"..."}
{"type":"result","subtype":"success","is_error":false,"duration_ms":0,"duration_api_ms":0,"num_turns":7,"result":"Here's a summary...","stop_reason":"end_turn","total_cost_usd":0.0127,"usage":{"input_tokens":812,"output_tokens":210,"cache_read_input_tokens":0,"cache_creation_input_tokens":0,"server_tool_use":{"web_search_requests":0}},"modelUsage":{},"session_id":"abc123","uuid":"..."}
```

消息类型：

| 类型        | 说明                                                              |
| ----------- | ---------------------------------------------------------------------- |
| `system`    | 会话前导（`subtype: "init"`），带模型、cwd、权限模式、工具、斜杠命令和 MCP 服务器。`subtype: "compact_boundary"` 标记一次自动压缩 |
| `assistant` | 一条模型消息；`message.content[]` 持有 `text`/`thinking`/`tool_use`，外加内联后端网络搜索的 `server_tool_use`/`web_search_tool_result` |
| `user`      | 工具结果，作为 `message.content[]` 内的 `tool_result` 块         |
| `result`    | 带最终文本、停止原因和花费字段的终端消息         |

`assistant` 和 `user` 消息携带 `session_id`、`uuid` 和 `parent_tool_use_id`（主对话为 `null`）。`system`/`init` 和终端 `result` 行携带 `session_id` 和 `uuid`，但没有 `parent_tool_use_id`。

每行上的 `uuid` 是每发出一行新生成的。它不是提供商、消息或事件 id，也不是关联键。它不匹配提供商 `message.id`（该值骑在 `assistant.message.id` 上）。即使描述同一条消息的行，它也按行唯一，并且不携带跨行或跨运行的身份。不要用它来关联或去重。

文本和推理块按每个模型响应分组到一条 assistant 消息。一次响应的并行 `tool_result` 块分组到单条 `user` 消息。`result.result` 是最终 assistant 消息文本。不产生内容块的模型响应在默认模式下不发出 `assistant` 行。只有 `--include-partial-messages` 会把这样的响应以其空的 `message_start` … `message_stop` 信封呈现出来。

在 `init` 上，`skills` 是实时的。它列出会话可被用户调用的技能名，是来自会话已宣传命令的 `slash_commands` 子集，或当会话没有呈现技能时为 `[]`。`init` 行发出一次，推迟到第一条输出行，以便捕获会话已宣传的 `tools`、`slash_commands` 和 `skills`。Messages schema 没有定义第二次 `init`，因此流开始后变化的命令列表不会再宣传。

其他 `init` 字段携带真实数据：

- `apiKeySource` 对 API 密钥认证是 `user`，否则是 `oauth`。Grok 不区分 schema 的 `project`、`org` 和 `temporary` 来源。
- `permissionMode` 是映射到 Messages 枚举的生效无头模式：`--permission-mode` 的值，或 `--yolo` 下的 `bypassPermissions`，否则 `default`。`auto` 这类仅 grok 的模式折叠为 `default`。
- `mcp_servers[].status` 反映配置，而不是实时连接状态。已配置的服务器始终报告 `"connected"`，因为发出 `init` 时还没有解析到按服务器的握手状态。

Grok 省略它没有数据的 schema 纯占位 `init` 字段，而不是发出哑值：`claude_code_version`、`output_style` 和 `plugins`。

`result` 包含 `duration_ms`、`duration_api_ms`、`num_turns`、`stop_reason`、`total_cost_usd`、`usage`（Messages API `message.usage` 形态）和 `modelUsage`。它也在错误子类型上包含 `errors[]`。Grok 省略 schema 中始终为空的 `permission_denials`，因为它不收集权限拒绝。`structured_output`（配合 `--json-schema`）是 snake_case，与 schema 一致。

`model` 出现在 `init` 和每一个 `assistant` 帧上。已知时它是真实模型 id，仅在发出时不知道任何模型时才是字面 `"unknown"`。

assistant 帧的 `stop_sequence` 端到端接通。当模型停在已配置的停止序列上时（`stop_reason: "stop_sequence"`），它携带提供商匹配到的停止序列；在其他每一种停止原因和后端上为 `null`。在 `--include-partial-messages` 框架中，匹配到的序列同时骑在刷出的 `assistant` 帧和部分 `message_delta.stop_sequence` 上，因此部分重建与帧匹配。只有部分 `message_start.stop_sequence` 保持 `null`，因为消息打开时还不知道匹配到的序列。

发出的错误子类型是 `error_max_turns`、`error_during_execution` 和 `error_max_structured_output_retries`。schema 的 `error_max_budget_usd` 子类型永远不会发出，因为 grok 没有预算功能。

`result.usage` 报告 Messages `message.usage` 形态，三个 token 桶互不相交：`input_tokens`（未缓存）、`cache_read_input_tokens` 和 `cache_creation_input_tokens`。Grok 从该回合的聚合账本派生这些，再塑成那些桶。子 agent 缓存创建包含在 `cache_creation_input_tokens` 中。聚合账本把它作为自己的桶跟踪，因此不再折进 `input_tokens`。

`result.usage` 始终发出数字桶，即使数据缺失。这发生在回合的用量账本不完整时（与 `json` 格式中呈现 `usage_is_incomplete` 的同一条件），或根本没有聚合账本到达归约器时。grok 无法入账的任何桶回退到 `0`，因为 Messages API schema 没有不完整或缺失用量的标记。两种情况下归约器都会向 stderr 记一条警告。这里把全零 `usage` 读作「未知」，不是「免费」。

嵌套的 `server_tool_use` 计数器已填充。`web_search_requests` 是本次运行发出的*成功*后端网络搜索次数。失败的搜索以及 open_page 这类非搜索 `WebSearch` 操作排除在外，与 Messages API 一致——它对出错的搜索不计费。失败的后端搜索仍以错误形态发出 `web_search_tool_result`（`content.type: "web_search_tool_result_error"`），但不计数。其 `error_code` 是固定的 `"unavailable"` 占位，不是从后端转发的码。没有 `web_fetch_requests` 键，因为 grok 没有服务端 `web_fetch`，因此省略该占位。

后端网络搜索是内联的。它折进与周围文本同一条 `assistant` 帧。该帧携带一个 `server_tool_use` 块（`name: "web_search"`、`input.query`），紧接着一个 `web_search_tool_result` 块。该结果块的 `tool_use_id` 匹配 `server_tool_use.id`，其 `content` 是 `{type, url, title}` 的 `web_search_result` 命中数组。这匹配 Messages API 的内联服务端工具形态，而不是把响应拆到多帧。

X 搜索和代码解释器是已记录的分歧。它们保持通用，呈现为客户端 `tool_use` 块加上 `user` `tool_result`，因为 Messages API 没有为它们定义内联块类型。其他每一个客户端工具同样保持 `tool_use`/`tool_result` 拆分。

`--include-partial-messages` 发出原始事件框架，以便消费者用 Messages 流式累加器重建每条消息。框架是 `message_start`、`content_block_start`/`content_block_delta`/`content_block_stop`、`message_delta` 和 `message_stop`。它携带累加器需要的结构事件。增量比 Messages API 的 token 级流更粗：工具输入作为单个 `input_json_delta` 到达，并且从不产生 `citations_delta`（见下）。结果是每条消息的忠实重建，而不是逐 token 回放。

在 Messages API 后端上，框架是忠实的。`message_start` 携带真实的提供商 `message.id` 以及输入侧 `usage`。思考块按顺序发出其 `signature_delta`，在该块的 `content_block_stop` 之前。`message_start.usage` 输入侧报告消息打开时已知的全部三个提示侧桶：`input_tokens`（未缓存部分）、`cache_read_input_tokens` 和 `cache_creation_input_tokens`。因此缓存命中在 `message_start` 上可见，而不是只在稍后的 `message_delta`/`result` 上出现。`output_tokens` 在那里播种为 `0`，并在 `message_delta` 上敲定。开始但未产生内容的响应仍发出没有内容块的 `message_start` … `message_stop` 信封。

有些后端只在回合结束时呈现每次响应的元数据。那些后端回退到合成的 `message_start.id` 和零播种的输入 `usage`。它们把推理 `signature` 推迟到最终 `assistant` 行，那种情况下该行才是权威的。

工具调用输入作为单个携带完整参数 JSON 的 `input_json_delta` 发出，然后是 `content_block_stop`。它不是一串 token 级片段。这是与 Messages API 增量 `partial_json` 流的有意分歧。Grok 的 ACP 工具调用路径在参数完全解析后，把每次工具调用作为一份已校验的 JSON 对象交付，因此单个增量才是准确表示。拼接 `partial_json` 的消费者无论哪种方式都会重组成同一对象。后端网络搜索 `server_tool_use` 块的 `input.query` 以同样方式发出，作为一个 `input_json_delta`。

Messages API 的 `citations_delta` 为被引用的文本跨度携带内联引用，例如来自网络搜索的。本流不产生它。Grok 的 Messages 内容增量限于文本、思考、签名和工具输入 JSON，因此没有可作为 `citations_delta` 呈现的引用数据。后端网络搜索的来源 URL 改为在已完成的 `web_search_tool_result` 块上内联报告（见上），而不是按跨度的文本引用。

保真注意事项适用于少数字段。

`duration_ms` 是提示执行的墙钟。`duration_api_ms` 是加总的*已报告*每次调用模型时间。不报告自己时长的模型调用贡献 `0`，因此 `duration_api_ms` 可能少计真实 API 时间。

`num_turns` 和 `total_cost_usd` 在已知时是权威的。未知时，`num_turns` 回退到本回合已完成模型响应的计数，`total_cost_usd` 回退到 `0`。已完成但无内容的响应不发出 `assistant` 行，但仍计为一回合。花费从不超额报告。

`modelUsage` 携带 grok 跟踪的按模型 token 和费用字段，外加归到活动模型的 `webSearchRequests`。归约器跟踪的是单个全局网络搜索计数而不是按模型，因此整个计数落在当前或最后一个模型上，其他行保持 `0`。当该模型的费用未知或被扣留时，按模型的 `modelUsage.*.costUSD` 为 `0`。这与顶层 `total_cost_usd` 相同的失败即归零行为。`json` 格式在部分时完全省略费用浮点数，但本流保持字段存在且为 `0`。`contextWindow` 是当前模型的真实总上下文窗口（与 grok 用于自动压缩的同一值），并且只出现在当前模型的行上。其他行省略它，当前行在窗口未知时也省略。`maxOutputTokens` 没有 grok 目录，因此该键完全省略。没有按模型分解时，`modelUsage` 为 `{}`。

与 `streaming-json` 一样，本流只读。工具批准和其他双向流使用 ACP 接口（`grok agent`）。

---

## 无头模式中的会话管理

默认情况下，每次 `grok -p` 调用都创建新会话。要在调用之间保持上下文，使用会话标志。

### 具名会话（`-s`）

要在无头调用之间携带上下文，使用 `-r/--resume` 或 `-c/--continue`。仅在用 **UUID** 创建**新**会话时使用 `-s/--session-id`（若不是 UUID 或在目标目录下已被占用则报错）。更旧的隐藏 `-s` upsert/恢复行为已移除。继续请用 `-r`/`-c`。与 `-r`/`-c` 一起时，`-s` 需要 `--fork-session`：

```bash
# 启动无头会话并捕获其 ID
grok -p "Review the changes in this PR" --output-format json | jq -r '.sessionId'

# 在同一会话中继续
grok -p "Now check for security issues" --resume "<id>"

# 可选：用客户端选定的 UUID 创建（必须尚不存在）
grok -p "hello" --session-id "$(uuidgen | tr '[:upper:]' '[:lower:]')" --output-format json
```

> **注意：** `-s/--session-id` 只创建新会话（有效 UUID；若已被占用则报错）。恢复请用 `-r`。

### 恢复（`-r`）

`-r/--resume` 标志按 ID 恢复特定会话，或当值不是 ID 时按当前目录的标题恢复，忽略字母大小写（重复项中唯一被手动重命名的匹配胜出；其余重复项报错并给出其 ID；形如 UUID 的值始终走 ID 路径，因此脚本应优先使用 ID）。会话不存在则报错：

```bash
# 从先前的 JSON 响应获取会话 ID
grok -p "Remember: the secret number is 42" --output-format json
# 输出包含 "sessionId": "abc123"

# 恢复那一个精确会话
grok -p "What's the secret number?" --resume abc123
```

### 继续（`-c`）

`-c/--continue` 标志继续当前工作目录中最近的会话：

```bash
grok -p "Continue where we left off" -c
```

### 提取会话 ID

使用 `--output-format json` 并解析 `sessionId` 字段：

```bash
grok -p "Hello" --output-format json | jq -r '.sessionId'
```

---

## 管道输入与输出

无头模式与 Unix 管道和重定向自然配合。

### 标准输出

```bash
# 把输出管道到文件
grok -p "Generate a README" > README.md

# 用 jq 解析 JSON 输出
grok -p "List files" --output-format json | jq -r '.text'
```

### 标准输入

无头模式不会把管道 stdin 读进提示。通过命令替换或 `--prompt-file` 传入外部内容：

```bash
# 通过命令替换把 git diff 作为上下文纳入
grok -p "Write a concise commit message for these changes:

$(git diff --staged)"

# 或从文件读取提示
grok --prompt-file ./prompt.txt
```

---

## CI/CD 集成示例

### 自动代码审查

```bash
grok -p "Review changes for bugs and security issues." \
  --output-format json --yolo | jq -r '.text' > review.md
```

### 提交前钩子

```bash
grok -p "Review staged changes for obvious bugs. Reply OK if fine, or list issues." \
  --yolo --output-format json | jq -r '.text' | grep -q "^OK" || exit 1
```

### 批处理

```bash
for file in src/*.js; do
  grok -p "Migrate $file from CommonJS to ES modules." --yolo
done
```

---

## 脚本模式

### Python 包装

Grok 的无头模式可以包装成 OpenAI 兼容的聊天补全 API：

```python
import asyncio
import json
import os

class GrokChat:
    """Simple OpenAI-compatible wrapper using headless mode."""

    def __init__(self, cwd="."):
        self.cwd = cwd
        self.env = {**os.environ}

    def _build_cmd(self, prompt, model, stream):
        return ["grok", "-p", prompt, "-m", model, "--cwd", self.cwd,
                "--output-format", "streaming-json" if stream else "json",
                "--yolo"]

    async def create(self, messages, model="grok-4.6", stream=False):
        prompt = messages[-1]["content"] if len(messages) == 1 else "\n".join(
            f"{m['role']}: {m['content']}" for m in messages
        )
        cmd = self._build_cmd(prompt, model, stream)

        if stream:
            return self._stream(cmd)

        proc = await asyncio.create_subprocess_exec(
            *cmd, env=self.env, stdout=asyncio.subprocess.PIPE
        )
        stdout, _ = await proc.communicate()
        data = json.loads(stdout.decode()) if stdout else {"text": ""}
        return {
            "choices": [{
                "message": {"role": "assistant", "content": data.get("text", "")},
                "finish_reason": "stop"
            }]
        }

    async def _stream(self, cmd):
        proc = await asyncio.create_subprocess_exec(
            *cmd, env=self.env, stdout=asyncio.subprocess.PIPE
        )
        async for line in proc.stdout:
            if not line.strip():
                continue
            event = json.loads(line)
            if event.get("type") == "text":
                yield {"choices": [{"delta": {"content": event["data"]}}]}
            elif event.get("type") == "end":
                yield {"choices": [{"delta": {}, "finish_reason": "stop"}]}


async def main():
    client = GrokChat(cwd=".")
    response = await client.create(
        [{"role": "user", "content": "What files are here?"}]
    )
    print(response["choices"][0]["message"]["content"])

asyncio.run(main())
```

### Shell 脚本

```bash
#!/bin/bash
# Run a code review and exit with failure if issues are found

RESULT=$(grok -p "Review this PR for bugs. Output JSON with 'issues' array." \
  --output-format json --yolo | jq -r '.text')

ISSUE_COUNT=$(echo "$RESULT" | jq '.issues | length' 2>/dev/null || echo "0")

if [ "$ISSUE_COUNT" -gt 0 ]; then
  echo "Found $ISSUE_COUNT issues"
  echo "$RESULT" | jq '.issues[]'
  exit 1
fi

echo "No issues found"
```

---

## 自动化的始终批准

`--always-approve`（别名 `--yolo`，与 `--permission-mode bypassPermissions` 相同）在没有交互式权限提示的情况下运行工具调用。拒绝规则、钩子和管理员锁仍然生效（见 [权限与安全](22-permissions-and-safety.md#permission-modes)）。

```bash
grok -p "Format all files" --always-approve
grok -p "Run the tests and fix any failures" --cwd ~/projects/my-app --always-approve
```

对 agent 服务器和 SDK，见 [Agent 模式](15-agent-mode.md#automation-and-sdks)。
---

## 无头环境变量

影响无头模式的关键环境变量：

| 变量                        | 说明                                                   |
| ------------------------------- | ------------------------------------------------------------- |
| `XAI_API_KEY`        | 认证用的 API 密钥（没有浏览器登录时必需）   |
| `GROK_HOME`                    | 覆盖配置目录（默认：`~/.grok`）                |
| `GROK_LOG_FILE`                | 日志文件路径（原样用作路径；无头和 TUI 都可用，尊重 `RUST_LOG`） |
| `RUST_LOG`                     | 日志级别过滤器（例如 `debug`）。无头日志写到 stderr。     |

对没有浏览器访问的 CI 环境，用 [console.x.ai](https://console.x.ai) 的 API 密钥设置 `XAI_API_KEY`：

```bash
export XAI_API_KEY="xai-..."
grok -p "Run the test suite" --yolo
```

---

## 退出码

| 码 | 含义                              |
| ---- | ------------------------------------ |
| `0`  | 成功。提示正常完成 |
| `1`  | 错误。认证失败、网络错误或运行时错误 |
| `130` | 被 SIGINT（Ctrl+C）中断                                   |
| `143` | 被 SIGTERM 终止                                            |

---

## 无头环境的认证

无头使用时，用以下之一认证：

- **`XAI_API_KEY`**：CI 最简单。见上面的 [环境变量](#environment-variables-for-headless)。
- **`grok login --device-auth`**（或 `--device-code`）：目标机器上不需要浏览器。
  见 [认证 > 设备码流程](02-authentication.md#device-code-flow)。
- **`grok login`**：有 GUI 的机器上基于浏览器的 OAuth2。

若你以前登录过，缓存的凭据会自动使用。

---

## 提示

- 无头模式默认启动**新会话**。用 `-r/--resume` 或 `-c/--continue` 在调用之间保持上下文。
- `--output-format json` 响应始终包含一个 `sessionId`，你可以把它与 `--resume` 一起用于后续调用。
- 把 `--yolo` 与 `--rules` 组合以设置护栏：`grok -p "..." --yolo --rules "Never delete files"`。
- 调试时提高日志级别并捕获 stderr：`RUST_LOG=debug grok -p "..." 2> debug.log`。

---

## 项目根发现

Grok 启动时，从 `--cwd`
（或当前目录）向上走，直到找到 `.git` 目录，以此发现项目根。

注意：若 `--cwd` 嵌在大型仓库（例如单体仓库）内部，
Grok 会把该仓库发现为项目根，并把发现（AGENTS.md、技能、git 历史）限定到它，这会让
启动变慢。把 `--cwd` 指向你想处理的具体子项目，以保持
作用域较小。

---

## 文件位置

Grok 把数据存在 `~/.grok`（用 `GROK_HOME` 覆盖；见 [无头环境变量](#environment-variables-for-headless)）：

| 路径                     | 内容                              |
| ------------------------ | ------------------------------------- |
| `config.toml`            | 用户配置                    |
| `auth.json`              | 缓存的 OAuth2/API 凭据         |
| `version.json`           | 用于更新检查的版本缓存       |
| `sessions/`              | 会话转录（SQLite）          |
| `memory/`                | 跨会话记忆存储            |
| `logs/`                  | 内部日志文件（例如 `unified.jsonl`） |
| `logs/mcp/`              | MCP 服务器日志                       |
| `skills/`                | 用户技能定义                |
| `personas/`              | 用户作用域的 agent Persona            |
| `crash/`                 | 崩溃报告                         |
| `trace-exports/`         | 会话追踪导出                 |
| `worktrees/`             | Git 工作树元数据                 |

### 只读 `~/.grok`

对容器或 CI，只读挂载 `~/.grok`：

- 预先填充 `auth.json` 或使用 `XAI_API_KEY`
- 会话持久化静默失败（临时）
- 更新检查记一条警告并跳过

```bash
export XAI_API_KEY="xai-..."
export GROK_DISABLE_AUTOUPDATER=1
grok -p "..." --no-auto-update
```

---

## 抑制更新检查

| 方法                          | 作用域     |
| ------------------------------- | --------- |
| `--no-auto-update`              | 会话   |
| `GROK_DISABLE_AUTOUPDATER=1`    | 进程   |
| 非 TTY 的 stderr（自动检测）  | 自动 |
| `[cli] auto_update = false`     | 持久|

`GROK_DISABLE_AUTOUPDATER` 设为假值（`0`、`false`、`off`、`no` 或空，任意
大小写）视为未设置。agent SDK
会为它们生成的非 leader agent 注入 `GROK_DISABLE_AUTOUPDATER=1`（SDK 隔离环境中的假值
会保持更新开启），stdio agent 跳过其后台更新，
除非它从托管安装运行（`$GROK_HOME/bin/grok`）。

更新消息发到 **stderr**。stdout 对 `--output-format json` 保持干净。另见 [无头环境变量](#environment-variables-for-headless)。

---

## 额外的无头标志

这些标志补充上面的 [命令行选项](#command-line-options) 表。那里已经列出的标志（`--prompt-json`、`--prompt-file`、`--verbatim`、`--sandbox`、`--no-auto-update`）此处不再重复。

| 标志                          | 说明                                       |
| ----------------------------- | ------------------------------------------------- |
| `--agent <NAME>`              | agent 名称或定义文件路径                |
| `--agents <JSON>`             | 以内联 JSON 作为子 agent 定义               |
| `--system-prompt-override`    | 覆盖 agent 的系统提示                |
| `--no-plan`                   | 禁用计划模式                                 |
| `--no-subagents`              | 禁用子 agent 生成                         |
| `GROK_MEMORY=0`                | 为本进程禁用跨会话记忆      |
| `--disable-web-search`        | 禁用网络搜索和抓取工具                |
| `--no-alt-screen`             | 内联运行（无备用屏幕）                  |
| `--worktree [NAME]`           | 从当前检出创建 git 工作树（包含脏变更）并在那里运行会话。从子目录启动会落在工作树的同一子目录。与 `-r` 一起时，会话恢复到新工作树。不能与 `--fork-session` 组合。 |
| `--ref <REF>` / `--worktree-ref <REF>` | 工作树基于的分支/标签/提交（与 `--worktree` 一起）；干净检出，无脏覆盖 |

---

## 被中断的无头运行

在 SIGINT/SIGTERM 时：

- 会话状态保存到最后一次完成的工具调用
- 工具所做的文件修改**不会回滚**
- SIGINT 的退出码是 **130**（`128 + 2`），SIGTERM 是 **143**（`128 + 15`）；CI 流水线可以把这些与普通错误（退出码 `1`）区分开
- 恢复：`grok -p "continue" --resume "<id>"` 或 `grok -p "continue" --continue`

具名会话以及 `-s`/`-r`/`-c` 标志的细节见 [无头模式中的会话管理](#session-management-in-headless-mode)。
