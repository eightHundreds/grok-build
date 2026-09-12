# 会话管理

Grok 会自动把每段对话保存到磁盘。无论你在 TUI、无头模式，还是通过 agent stdio 工作，Grok 都会把交流记录为会话。你可以恢复、回退或压缩它。本文说明如何管理会话。

---

## 什么是会话

会话是带完整历史的持久对话。它包括：

- 所有用户提示和 agent 回复
- 工具调用及其结果
- TODO/任务列表状态
- 用于之后撤销回合的回退点
- Token 用量和回合计数
- 子 agent 会话（启用时）

会话由唯一的会话 ID 标识（Grok 生成时为 UUIDv7；客户端可用 `-s` 提供自己的 ID），并存放在 `~/.grok/sessions/` 下。设置 `GROK_HOME` 可覆盖基目录；未设置时 Grok 使用 `~/.grok`。

---

## 存储布局

Grok 把每个会话存在自己的目录里，并按工作目录分组。它会对工作目录做 URL 编码来命名分组。编码后的名字超过 255 字节时，改用 slug 加哈希，并把原始路径记在分组内的 `.cwd` 文件里。

```
~/.grok/sessions/<encoded-cwd>/<session-id>/
  summary.json            # 元数据：摘要/标题、时间戳、模型 ID、消息计数
  updates.jsonl           # ACP 会话更新流（对话 + 工具调用）
  chat_history.jsonl      # 发给模型的原始聊天消息
  plan.json               # TODO/任务列表状态
  rewind_points.jsonl     # /rewind 撤销用的回退点
  signals.json            # 会话信号（token 用量、工具/回合计数）
  feedback.jsonl          # 用户反馈与评分
  compaction_checkpoints/ # 压缩保存的状态（手动或自动）
  subagents/              # 每个子 agent 的元数据（meta.json）；子会话仍在常规 sessions 树里
```

`summary.json` 是索引条目。它记录会话摘要和生成的标题、模型 ID、创建与更新时间戳、消息计数，以及派生或恢复会话的父会话引用。它还记录最近一次的 last-turn 摘要和会话 recap，方便列表界面展示。`updates.jsonl` 是权威对话日志，驱动 `/resume` 和会话恢复。每回合的 token 与费用合计可通过 `grok usage` 查看。

### 会话标题

仪表盘和 `/resume` 里显示的会话标题，会根据对话自动生成。提示框边框只有在你手动 `/rename` 之后才显示标题；若草稿已暂存，旁边还会有 `Stashed` 说明。标题生成从你的第一条提示之后立刻开始，因此会话总会有标题；随后会在前几个回合根据整段对话再生成几次，然后冻结。这样标题可以越过含糊的第一条提示，反映会话真正在做什么，之后又保持稳定，方便你辨认会话。手动 `/rename` 始终优先：一旦你重命名会话，自动生成就再也不会覆盖它。用 `/rename --auto` 可以把标题交回自动生成。

---

## 开始与结束会话

### 新会话

每次启动 TUI 都会创建新会话。要在会话中途明确重新开始：

```
/new
```

这会清空当前上下文并开始新对话。别名：`/clear`。

### 退出

结束会话并退出 Grok：

```
/quit
```

别名：`/exit`。若要离开当前会话但留在 Grok 里，用 `/home` 回到欢迎屏。

### 删除当前会话

```
/delete
```

确认后永久删除会话历史。回到欢迎屏；若你是从仪表盘打开该会话的，则回到仪表盘。在 `/resume` 或欢迎屏的会话列表里，按 `d` 再按 `y`。在 [Agent Dashboard](23-dashboard.md) 上，连续按两次 `Ctrl+X`（或悬停 `[✗]`）会永久删除。

---

## 恢复会话

### 从 TUI

用 `/resume` 浏览并恢复以前的会话：

```
/resume
```

这会打开会话选择器，列出当前工作区的近期会话。选一个即可恢复。该命令不接受参数。

在选择器里输入会按标题过滤列表，并同时搜索对话内容；内容匹配出现在 "Extended search results" 标题下。按 `Ctrl+/` 可立刻搜索，不必等短暂停顿。

对于本 pager 里活着的顶层会话（父会话和派生）——切换、重命名、peek、派发或关闭——请用 [Agent Dashboard](23-dashboard.md)：`/dashboard`（别名 `/sessions`、`/agents-dashboard`）或 `Ctrl+\`。

### 从命令行

按 ID 或标题恢复指定会话：

```bash
grok --resume <session-id-or-title>
```

不是会话 ID 的值会按当前目录的会话标题匹配，忽略字母大小写（简单的小写比较）——`/rename` 之后很方便。若多个会话同名，单独一个手动重命名过的会话优先于自动生成的重复项；否则命令报错并列出匹配的 ID。形如 UUID 的值一律当作会话 ID，不当标题。脚本应优先用 ID。

不带值运行 `grok --resume` 会恢复当前目录最近的会话。

### 从欢迎屏

启动 `grok` 时，欢迎屏会列出当前目录的近期会话。选一个即可恢复。

---

## 派生与重命名会话

### 派生

把当前会话分支成一个对等 agent，从对话副本开始：

```
/fork [--worktree|--no-worktree] [directive]
```

可传可选的 `directive` 作为新会话的第一条提示。用 `--worktree` 或 `--no-worktree` 选择派生是否在新的 git 工作树中运行；两者都省略则会每次询问。此版本不支持 `--at <turn>` 旗标。

### 重命名

重命名当前会话的标题：

```
/rename <title>
/rename --auto
```

别名：`/title`。`/rename --auto` 会清除手动标题并重新启用自动命名。

---

## /rewind 命令

`/rewind`（别名 `/undo`）把对话回退到更早的回合，丢掉之后的回合。该回合之后的文件改动会原样留在磁盘上。

```
/rewind
/undo
```

当你运行 `/rewind` 或 `/undo`（或在空闲、提示框为空且已有对话消息时，800ms 内按 **Esc Esc**），Grok 会：

1. 显示回退点列表（每个用户提示一个）
2. 让你选择回退到哪一点
3. 把对话历史截断到该点

当 **Confirm before rewind** 开启时（`/settings` 中的默认），每次选择都会要求确认（Yes / Yes, and don't ask again / No）。**Yes, and don't ask again** 会关掉该设置。设置关闭后，选择会立刻执行。

**重要：** `/rewind` 不会恢复磁盘上的文件。只有对话历史会被截断。

---

## /compact 命令

`/compact` 压缩对话历史，以节省上下文窗口空间。用在早期消息已不再相关的长会话里。

```
/compact
/compact [context]
```

可选的 `context` 参数让你额外说明压缩时要保留什么。

### 自动压缩

当上下文窗口接近上限时，Grok 会自动压缩对话。自动压缩触发时你会看到通知。模型配置上的 `context_window` 设置控制何时到达该阈值。

---

## /session-info 命令

查看当前会话的详情：

```
/session-info
```

会显示：

- 会话标题（已设置时）
- Shell 版本
- 认证方式（OAuth 对比 API key；API-key 会话还会建议用 `grok login` 获取 SuperGrok）
- 会话 ID
- 工作目录
- 模型（编码模型会带模型哈希）
- API 后端和沙箱配置（已设置时）
- 上下文窗口用量（已用与总 token，以及已用百分比）

在 Session info 标签页上，点击一个值可复制，或拖选一段范围（高亮方式与工具查看器相同）。`c` 复制会话 ID，`y` 复制整块。复制走的是 Grok 其余部分同一套剪贴板路径，包括 `grok wrap`。

---

## 无头会话管理

在无头模式里，通过命令行旗标管理会话：

```bash
# New session each time (default)
grok -p "Hello"

# Resume an existing session by ID or title (errors if it does not exist)
grok -p "Continue where we left off" -r <session-id-or-title>

# Continue the most recent session in the current directory
grok -p "What were we doing?" -c
```

在无头模式中，用 `-r`/`--resume` 恢复已有会话（会话不存在则报错），或用 `-c`/`--continue` 继续当前目录最近的会话。非 ID 值会按当前目录的会话标题匹配，忽略字母大小写（重复项中单独一个手动重命名的匹配优先；其余重复项报错并列出 ID；形如 UUID 的值一律走 ID 路径）——脚本应把 JSON 输出（见下）里的会话 ID 传给 `-r`。

只用 `-s`/`--session-id` **创建** 带 **UUID** 的新会话（值不是 UUID，或该 ID 在目标会话目录下已有会话，都会报错）。它**不会**恢复已有会话——那是以前隐藏的 upsert 行为；请改用 `-r`/`-c`。只有同时传 `--fork-session` 时才把 `-s` 与 `-r`/`-c` 组合使用（把历史派生到新 ID；可选的 `-s` 指定子会话 UUID）。这与 Claude Code 的防覆盖模型一致（在写入 cwd 下做客户端预检；顺序使用可靠，同一 ID 并发则尽力而为）。

要读回会话 ID，请求 JSON 输出：

```bash
grok -p "Hello" --output-format json | jq -r '.sessionId'
```

---

## Agent stdio 会话管理

用 ACP 构建时，通过协议方法管理会话：

```typescript
// Create new session
const { sessionId } = await connection.request("session/new", {
  cwd: "/path/to/project",
  mcpServers: [],
});

// Load existing session
await connection.request("session/load", {
  sessionId: "existing-session-id",
  cwd: "/path/to/project",
  mcpServers: [],
});

// Change a live option (model or reasoning_effort).
// session/new and session/load already return the typed configOptions list.
await connection.request("session/set_config_option", {
  sessionId,
  configId: "model",
  value: { value: "grok-4.6" },
});
```

agent 会自动持久化所有会话更新。客户端可以重连并按 ID 加载以前的会话。选项 ID、值形状和 leader-mode snoop 见 [Agent 模式](15-agent-mode.md#session-config-options)。

---

## grok sessions 子命令

从命令行列出或搜索会话。`grok sessions` 需要子命令：

```bash
# List recent sessions for the current directory
grok sessions list

# Limit the number of results (default 20)
grok sessions list --limit 50

# Search sessions by keyword (matches titles and prompts)
grok sessions search "rate limit"
```

`grok sessions list` 显示当前工作目录的会话，按工作树标签分组。每行列出会话 ID、创建与更新日期、来源状态和摘要。`grok sessions search` 会把本地 SQLite 索引与远程结果合在一起。

---

## grok usage 子命令

打印会话已持久化的 token 与费用用量。请用这个，而不是直接读会话文件：

```bash
# Session totals plus every recorded turn
grok usage <session-id>

# One turn
grok usage <session-id> 3
```

输出是带 `sessionId`、`updatedAt`、`session` 和 `turns` 的 JSON。指定回合时用同一信封，`turns` 里只有一项。会话合计覆盖整段对话，包括恢复或派生继承的历史。`costUsdTicks` 是每美元 10¹⁰ ticks（除以 `1e10` 得到美元）。缺少的回合号是错误。交互式额度与账单仍在 TUI 的 `/usage` 上。

---

## 工作树会话

使用子 agent 或会话派生时，Grok 可以为每个会话创建隔离的 git 工作树。每个工作树有自己的工作目录副本，因此一个会话里的文件改动不会影响另一个。

工作树会话通过 `x.ai/git/worktree/*` 扩展方法在内部管理。关键操作：

- **Create**：为隔离会话创建新工作树
- **Apply**：把工作树改动合并回主工作目录
- **Remove**：会话结束后清理工作树

用 `grok -w -r <session-id>` 在全新工作树中恢复会话。

### 检查磁盘用量

`grok du`（别名：`grok disk-usage`）报告 grok 家目录（`~/.grok`）占用了多少磁盘。它先按从大到小列出每个顶层目录，再列出每个工作树及其大小、类型、年龄、标签和路径。注册表未跟踪的工作树显示为 `untracked`。传 `--json` 可得到同一份机器可读报告。

```text
Disk usage for ~/.grok
    412.3 GB  worktrees
      1.2 GB  sessions
    412.0 MB  (top-level files)
    413.9 GB  total
  Worktree clones share storage with their source, so the total can exceed real disk use.

Worktrees
        SIZE  TYPE                AGE        LABEL  PATH
    380.0 GB  session             12d ago    my-fix ~/.grok/worktrees/xai/worktree-abc
     32.3 GB  untracked (session) 40d ago           ~/.grok/worktrees/xai/worktree-old

To reclaim space, run `grok worktree gc --max-age 7d --dry-run`, then the same command without `--dry-run`. Without `--max-age`, gc expires nothing, and it keeps a worktree whose work it cannot find elsewhere, naming each one.
Untracked rows are not in the registry, so gc never visits them. Remove one with `grok worktree rm --dry-run <path>`, then without `--dry-run`.
```

`AGE` 是 `grok worktree gc` 衡量的值：自工作树上次访问以来的时间，若创建时间更新则用创建时间。会话和 agent 活动会更新它；在该目录里开着的 shell 或编辑器不会。未跟踪的工作树没有注册表条目，因此年龄来自其下最新的文件。

大小在 Unix 上是物理块计数，在其他平台是逻辑文件大小，与 `grok worktree show` 报告的一致。工作树克隆与源共享存储，但每份副本都按全量计入，因此合计可能超过 `du -sh` 以及实际占用的空间。当合计超过卷上已用空间时，报告会说明。`--json` 用 `volume_capacity_bytes` 和 `volume_available_bytes` 带上同样的数字。

报告只衡量一块文件系统，即存放 grok 家目录的那一块。位于任何其他文件系统上的目录不计入合计，而是记入 `other_filesystem_dirs`，其工作树行的大小显示为 `-`（`--json` 里为 `null`）。指向目录的顶层符号链接（例如被挪走的 `worktrees`）记入 `unfollowed_dir_symlinks`；其目标不计入合计，不过其下各行仍会计算大小。报告读不到的目录和条目分别记入 `unreadable_dirs` 和 `unstatable_entries`。运行 `RUST_LOG=debug grok du` 可点名每一个。

`--json` 里每一行工作树还带有以 unix 秒表示的 `created_at`、`last_accessed_at` 和 `last_modified_at`，以及 `repo_name` 和 `git_ref`。未跟踪行的注册表字段为 `null`。`git_ref` 是注册工作树时记录的分支，不是现在检出的分支。

注册表不可用时，每一行都显示为 `untracked`，报告会点名原因。`--json` 的 `registry` 字段带同样的值：`read`、`absent`、`busy`、`unopenable` 或 `corrupt`。`busy` 表示注册表被另一进程占用，请重试。`unopenable` 表示权限或 I/O 问题，请检查该文件。只有 `corrupt` 需要删除：删掉报告点名的文件，然后运行 `grok worktree db rebuild`。

要回收空间，运行 `grok worktree gc --max-age 7d`，它会删除超过你给定年龄的已跟踪工作树。不带 `--max-age` 时，gc 不会过期任何东西，并且只访问注册表跟踪的工作树。用 `grok worktree rm <path>` 删除未跟踪的工作树。两条命令都接受 `--dry-run` 并报告将要做什么：gc 统计会删除的工作树，`rm` 点名路径。

每次运行大约在一分钟内尽可能多地判断工作树，因为同一趟检查也会在会话旁按定时器跑，而读完整棵工作树并不免费。没轮到的记为 `Not judged this pass`，等下次运行，所以在有大量空间要回收的机器上，请反复跑 gc 直到该数字为零。

删除过期工作树之前，gc 会检查删除是否会毁掉工作：未提交、未跟踪或 ignored 文件，没有任何幸存引用持有的提交，或只存在于该工作树 git 目录里的状态。无法检查的工作树也会保留。报告会统计保留的工作树并点名原因，与被活进程挡住的那些分开。`--force` 不会跳过这项检查，而 `grok worktree rm` 不应用它：它会删除你点名的路径。

Ignored 文件也算工作，只有一个例外：被仓库自身 ignore 规则排除、并且要么带有工具的 cache tag、要么名称像其输出目录之一（`target`、`node_modules`、`.venv` 等）的目录。仅有名称永远不够，因此没人排除的手写 `build/` 仍会保住工作树。

只被工作树自己的 reflog 点名的提交——`reset --hard` 或 amend 留下的那种——会在工作树来源仓库里得到一个持久名字 `refs/grok/reclaimed/<worktree>/<commit>`。Git 在 prune 时把 reflog 算作可达性，所以没有这个名字，删除工作树就会让该提交不可达。用 `git log refs/grok/reclaimed/` 和 `git branch <name> <commit>` 恢复。

这些名字不会无限堆积。每次 gc 会丢掉不再持有任何东西的名字：提交现在已从真实引用可达，或已超过 30 天。报告把它们计为 `names_collected`。

---

## 会话存储细节

### 持久化格式

Grok 把对话存成换行分隔的 JSON（JSONL）。`updates.jsonl` 里每一行都是一条自包含的 ACP 会话更新事件。这种格式支持：

- 增量写入（会话期间只追加）
- 高效流式读取（用于会话恢复）
- 便于调试（每一行都是合法 JSON）

较小的状态文件——`summary.json`、`plan.json` 和 `signals.json`——是普通 JSON 而不是 JSONL。JSONL 是会话内容的权威来源；`grok sessions search` 还会在会话标题和提示上维护本地 SQLite FTS5 索引，以便快速关键词搜索。

### 会话元数据

`summary.json` 记录的字段包括：

- `info` —— 会话 ID 和工作目录
- `session_summary` 和 `generated_title` —— 会话摘要及其模型生成的标题
- `title_is_manual` —— 标题由手动 `/rename` 设置时为 true（因此自动生成会放过它）
- `created_at` 和 `updated_at` —— 创建与最后更新时间戳
- `num_messages` 和 `num_chat_messages` —— 更新与聊天消息计数
- `current_model_id` —— 正在使用的模型
- `parent_session_id` —— 派生或恢复的源会话
- `agent_name` —— 会话上次保存时活动的 agent 定义
- `last_turn_summary` —— 最近一回合的极短摘要
- `last_recap` —— 最新会话 recap 的有界预览

### 磁盘用量

长会话里，会话历史（`updates.jsonl`、`chat_history.jsonl`）主导磁盘占用。用 `/compact` 缩小历史。

---

## 提示

- 当前上下文不再相关时，用 `/new` 重新开始。
- 在长会话里主动用 `/compact`，让上下文窗口保持有效。
- 用 `/rewind` 撤销错误；它把对话回退到更早的回合（被去掉的回合里的文件改动会原样留下）。
- 在无头模式中，从 JSON 输出捕获 `sessionId` 并传给 `-r`，可构建保持上下文的多步自动化。
- 用 `/session-info` 查看上下文窗口已用多少。
