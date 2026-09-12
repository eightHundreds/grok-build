# 状态行

可选的一行，画在 pager 底部——全屏时在快捷键栏上方，极简模式时在提示信息行下方——默认关闭。它显示实时会话上下文，例如模型、上下文窗口用量、费用、目录和 git 工作树，或你配置的脚本输出。在 `~/.grok/config.toml` 里用 `[ui.status_line]` 开启。

## 设置

### 内置

```toml
[ui.status_line]
type = "builtin"
items = ["cwd", "model", "context"]   # 省略时的默认
```

例如会画成 `grok-shell-status-line │ Grok 4.5 │ 12% ctx`。条目按你列出的顺序出现，过长的会用 `…` 省略：目录和会话名在 40 列，模型在 30 列。

| 条目 | 显示 |
| --- | --- |
| `cwd` | 当前目录（basename） |
| `model` | 模型显示名 |
| `context` | 上下文窗口百分比，到达自动压缩阈值时为琥珀色；agent 未报告阈值时用 80% |
| `cost` | 会话费用，低于 $0.005 时隐藏，以免误显示 `$0.00` |
| `turn-timer` | 当前回合已用时间，从满一秒开始 |
| `session-name` | 会话名（若已设置） |

### 命令

把 `command` 指向脚本。Grok 把 [JSON](#available-data) 通过 stdin 喂给它，并显示 stdout。`~/` 前缀会展开成家目录。

```toml
[ui.status_line]
type = "command"
command = "~/.grok/statusline.sh"
```

字段名和嵌套遵循常见状态行约定，移植脚本通常只需小改而不是重写。下表未列出的内容不会发送。

### 关闭

`type = "disabled"` 是默认，什么也不显示；`off`、`none`、`hidden` 也当作 `disabled`。

### 选项

| 键 | 类型 | 默认 | 说明 |
| --- | --- | --- | --- |
| `type` | string | `disabled` | `builtin`、`command` 或 `disabled`。 |
| `items` | array | `["cwd", "model", "context"]` | 内置段，按顺序。 |
| `command` | string | 无 | `type = "command"` 时的脚本。 |
| `padding` | integer | `0` | 每侧水平间距（字符），上限 16。若 padding 宽到不剩列，会占住这一行但不画内容。 |
| `refresh_interval` | integer | 未设置 | 仅 `command` 行，单位秒，1 到 86,400。即使没有状态变化也按此间隔重跑脚本，让空闲会话仍能显示事故页、CI 状态等。未设置则完全事件驱动。它调度的那次运行带 `"trigger": "refresh_interval"`，失败时保留上次输出而不是画错误（见 [刷新运行](#refresh-runs)）。会打网络的脚本应偏好更长间隔，并在 `state` 运行时读缓存。 |

## 工作方式

- **刷新。** 会话状态变化时更新（会话开始、回合结束、切换模型或力度、HEAD 移动、压缩、客户端接入），回合进行中会持续更新，而不是靠定时器。空闲会话不会重跑你的脚本，所以脚本里的时钟不会自己走——除非设置 `refresh_interval`，它会在上述事件之上再加定时器。这些更新以固定 300 ms 防抖，忙碌回合不会每帧都跑脚本；必须立刻显示的变化（缩放、新快照、切换 agent）只等 100 ms。已经在跑的那次不会被取消：下一次变化等它结束。Grok 在启动时读取 `[ui.status_line]`，改动下次启动才生效。
- **输出。** 你打印的每一行成为状态行的一行，最多五行，每行按 1024 字符截断（含 ANSI 转义本身），所以颜色很重的行留给文字的空间更少。矮终端会更少，从底部丢掉多余行。ANSI 颜色保留；其他转义（光标移动、清行、回车覆盖）丢掉。`http`/`https`/`mailto` 的 OSC 8 超链接保留，其他目标按纯文本。stdout 超过 64 KiB 会截断并停掉脚本。脚本成功但什么都不打印会拿走这一行，而不是回退到内置段，因此只偶尔打印的脚本会让对话上下挪一行。
- **尺寸。** Grok 把 `COLUMNS` 和 `LINES` 设成你的输出所填的那一行，而不是整个窗口：面板 padding 和你的 `padding` 已经扣掉。`tput` 也会报这些，因为它在 stdout 不是终端时读这些变量。`LINES` 是当前已填的行数而不是可能长到的高度，所以在你打印更多之前读到 `1`；上限始终是五。行还没画过一次、或这一帧没空间时，尺寸用上次画过的，若从未画过则是 80x1。
- **Shell。** `command` 是一条 shell 命令行，因此 `jq -r '…'` 和管道可按原样写；路径若指向可执行文件会直接跑，否则走 `sh -c`，这也是 `#!` 缺失或错误时的路径。路径含空格时像在提示符里一样加引号。每次运行都是新进程，所以改脚本文件会在下次运行生效。
- **后台工作不会幸存。** 脚本留下的任何进程在运行结束时都会被杀掉：干净退出、超时、或输出过多。运行在你的脚本退出时结束，之后后台任务再打印的内容会丢失。
- **环境。** 脚本在会话工作目录、然后仓库根、然后 pager 自己的目录中运行（取第一个本地路径），超时 10 秒，超时后行显示 `[status line: timed out]`。`COLUMNS` 和 `LINES` 描述脚本填充的行，不是窗口。不跑 shell rc（清空 `BASH_ENV` 和 `ENV`），并设 `GIT_OPTIONAL_LOCKS=0`。分页器和编辑器被中和，方式与 Grok 其余部分相同，因此脚本里的 `git` 或 `gh` 不会卡在等分页器。
- **输入。** JSON 负载写入 stdin 并带尾随换行，因此 `read -r line` 和 `input=$(cat)` 都能用。

## 刷新运行

给 `command` 行设置 `refresh_interval` 后，脚本也会按定时器重跑，让事故页或 CI 状态在会话空闲时也能上到这一行：

```toml
[ui.status_line]
type = "command"
command = "~/.grok/statusline.sh"
refresh_interval = 300   # 秒
```

- **负载会说明脚本为何运行。** 回答定时器的运行带 `"trigger": "refresh_interval"`——定时器到期时若正好有状态变化，会搭那一次运行——没有到期的定时器则带 `"trigger": "state"`。在 `refresh_interval` 上打网络、在 `state` 上读缓存；否则忙碌回合会连续重跑脚本，变成对脚本所调服务的请求风暴。
- **负载是 Grok 上次发送的那份。** 定时器运行用上次状态变化的负载重跑脚本，因此会话数字（费用、上下文、token）是那次变化时的，不是定时器触发时的。只有脚本自己拉取的才是新的。
- **刷新失败保留上次输出。** 一旦脚本已经回答过——画出一行，或故意什么都不打印——定时器运行失败或超时会原样留下这一行，无论那是上次输出还是状态运行已经画过的失败，并把失败写到 `~/.grok/logs/unified.jsonl`，以免脆弱端点在安静夜里刷错误。连续三次刷新失败说明脚本本身坏了，这时才显示错误；脚本还从未回答过（新会话，或刚切换 agent）的刷新失败也会立刻画出，因为没有可保留的内容。由会话状态触发的运行仍会立刻报告失败。
- **错过的触发会合并。** 行被隐藏（全屏子 agent、欢迎屏）或已有运行占着槽位时，触发会等待，行恢复后只欠一次运行——不会为挂起或长回合跳过的那些触发来一串。定时器保持自己的节奏，无论脚本跑多久：运行还在进行时到期的触发会带到下一次，而不是堆在后面。
- **定时器属于跑脚本的那种模式。** `builtin` 下的 `refresh_interval` 什么也不调度，并通过 `grok inspect` 报告；`disabled` 下它和其他东西一起关掉。

## 可用数据

移植脚本时请仔细读这些。`workspace.repo_root` 是仓库根，没有 `project_dir`（别处用来表示启动目录）。`context_window.session_usage` 和 `session_*` token 计数是整段会话累计，不是单次调用，而当前窗口是 `context_window.context_tokens`。没有额外会话目录列表，因为 Grok 没有。`transcript_path` 指向 Grok 自己的更新流，而不是另一种工具格式的 transcript，`prompt_id` 只在回合进行中出现。这些情况下移植脚本读不到值，而不是读到错值，所以请保护你用到的字段。

下表之外的内容不会发送。移植脚本来读 agent 改了多少行、限速摘要、编辑器模式、思考/快速模式旗标、输出风格、PR、额外会话目录、或工作树创建自哪个目录，都会发现它们不在：要么是 Grok 没有的功能，要么是它无法诚实给出的数字。

| 字段 | 说明 |
| --- | --- |
| `cwd`, `session_id` | 工作目录和唯一会话 id |
| `session_name` | 会话的标签名，由客户端填写。出现在 `command` stdin，不出现在 `SessionStatus` 通知 |
| `prompt_id` | 正在处理的提示的 UUID。仅在回合中出现 |
| `transcript_path` | 会话 `updates.jsonl` 的路径。该文件是 Grok 自己的更新流，解析其他工具 transcript 格式的脚本读不懂 |
| `model.id`, `model.display_name` | 模型标识和显示名。agent 读不到会话模型时省略 |
| `workspace.current_dir` | 当前目录 |
| `workspace.repo_root` | 仓库根，不在仓库里时省略。不是 `project_dir` |
| `workspace.branch` | 已检出分支，任意仓库。detached HEAD 时省略 |
| `workspace.git_worktree` | 链接工作树内的工作树名 |
| `workspace.repo.{host,owner,name}` | 从 `origin` 远程解析，git 仓库内。没有 owner 段的远程会省略 `owner` |
| `schema_version` | 负载形状修订。加字段不 bump；删除或改类型才 bump。用 `>=` 测试，按它分支而不是按 `version` |
| `version` | Grok 发布版本，用于显示 |
| `cost.total_duration_ms` | 本进程接入会话以来的毫秒。恢复的会话从恢复时起算，费用也一样 |
| `cost.total_cost_usd`, `cost.total_api_duration_ms` | 会话费用和 API 等待毫秒。会话里还没有任何带价格的东西时费用缺省，用量账本读不到时也缺省，因此把缺省费用当成未知而不是零 |
| `context_window.context_window_size` | 最大上下文，token。模型窗口未知时省略 |
| `context_window.context_tokens` | 对话此刻占用的 token，只计输入，因此压缩后会下降。agent 读不到计数时省略，所以 `0` 永远表示空上下文 |
| `context_window.session_input_tokens`, `.session_output_tokens` | 整段会话计费，因此只会增长。用 session 命名是因为它们计的是会话：别处的 `total_*` 表示当前窗口里有什么，这里对应 `context_tokens`。用它们除以 `context_window_size` 会超过 100% 并继续涨。用量账本读不到时省略 |
| `context_window.used_percentage`, `.remaining_percentage` | 当前窗口有多满，0 到 100 的整数。与 `context_window_size` 或 `context_tokens` 一起省略，因为未知窗口的百分比不是数字 |
| `context_window.session_usage.{input_tokens,output_tokens,cache_creation_input_tokens,cache_read_input_tokens}` | `input_tokens`、`cache_creation_input_tokens` 和 `cache_read_input_tokens` 加回 `session_input_tokens`，外加 `output_tokens`。整段会话累计，不是一个回合。第一次调用前缺省 |
| `context_window.auto_compact_threshold_percent` | 会话自动压缩的位置。agent 未报告时省略 |
| `effort.level` | 推理力度（模型支持时） |
| `turn.started_at_ms` | 进行中回合开始的 Unix 毫秒，回合之间缺省。用你自己的时钟减去它得到已用时间 |
| `worktree.{name,path,branch,main_worktree_root}` | 链接工作树内的活动工作树。文件系统根上的工作树省略 `name`，`main_worktree_root` 是工作树分出的位置 |
| `trigger` | 这次运行为何被调用：定时器要的是 `refresh_interval`，否则是 `state`。出现在 command 行的 stdin，不出现在描述会话而不是某次运行的 `SessionStatus` 通知 |

Grok 无法来源的字段会省略而不是填占位，因此这一行从不显示编造值。务必保护它们：`jq -r` 对缺失键打印字面 `null`，所以在 jq 里写 `// 0` 或 `// "?"`，在 JavaScript 里用 `?.`。

## 示例

保存脚本（例如 `~/.grok/statusline.sh`），`chmod +x` 后设为 `command`。这个例子用 [`jq`](https://jqlang.org/)；Python 和 Node.js 原生解析 JSON。负载没有脏文件计数，所以脚本自己调 `git`。

```bash
#!/bin/bash
input=$(cat)
DIR=$(echo "$input" | jq -r '.workspace.current_dir')
MODEL=$(echo "$input" | jq -r '.model.display_name // "?"')
PCT=$(echo "$input" | jq -r '.context_window.used_percentage // 0')
BRANCH=$(echo "$input" | jq -r '.workspace.branch // "detached"')
DIRTY=$(git diff --numstat 2>/dev/null | wc -l | tr -d ' ')
printf '%b\n' "${DIR##*/} │ $MODEL │ ${PCT}% ctx │ \033[32m$BRANCH\033[0m ~$DIRTY"
```

## 提示

- 用模拟输入测试：`echo '{"session_id":"t","workspace":{"current_dir":"/tmp/demo"},"model":{"display_name":"Grok 4.5"},"context_window":{"used_percentage":25}}' | ./statusline.sh`
- 把 `git status` 这类慢命令缓存到按 `session_id` 分键的临时文件，每隔几秒刷新。`session_id` 对每个会话稳定，跨会话唯一。
- 用 `printf '%b'` 而不是 `echo -e` 来获得可靠转义。

## 排障

- **什么都不显示。** Grok 启动时读 `[ui.status_line]`，改 `config.toml` 后要重启。重启就够：新客户端接入时，agent 会为仍在跑的会话打开这一行。只有 agent 视图激活时才画，欢迎屏或全屏子 agent 视图不会。检查 `type` 不是 `disabled`，以及 command 脚本可执行并写 stdout。
- **行里有一条消息。** 以 `[ui.status_line]` 开头的行表示 Grok 无法按所写使用该段：它会点名读不了的键，或你选的模式还缺什么。`grok inspect` 列出同样的问题，包括此版本不认识的键，行被关掉时先看这里。它能读到的部分仍然生效，Grok 不会改写读不了的段。设 `type = "disabled"` 会去掉这一行和消息。
- **空白行永远不填。** agent 没在发状态更新，通常意味着 `grok` 或 leader 进程比这个客户端旧。重启 leader 或更新 Grok。
- **只有你自己的配置能设这个。** `command` 行会跑程序，因此只从你的 `~/.grok/config.toml` 和管理员托管配置读取。仓库不能设：仓库本地的 `.grok/config.toml` 只读 MCP 服务器，`[ui.status_line]` 不在任何项目层能提供的键里，所以克隆仓库不能让 Grok 跑它的脚本。
- **推送的配置没有效果。** `[ui.status_line]` 会从 campaign 和版本覆盖补丁里剥掉，因为状态行可以点名你机器上会跑的命令。写在你自己的 `config.toml`。
- **错误。** 脚本打印的任何内容都会显示，即使非零退出，因此 `printf …; [[ -n $dirty ]]` 行为如你所料。脚本什么都不打印且失败时显示 `[status line: exit N]`，直到下次成功——对会话状态触发的运行立刻报告失败；定时器运行的失败则保留上次输出（见 [刷新运行](#refresh-runs)）。脚本的 stderr 从不画上去，所以调试 `echo` 不会打扰这一行；用 `--debug` 跑 Grok 才能读到。Grok 根本启动不了的脚本显示 `[status line: could not start the script: …]`，没执行位的文件就是这样，被系统杀掉则显示 `[status line: killed by signal]`。`#!` 指向缺失解释器时会改在 `sh` 下重试，因此显示退出码。
