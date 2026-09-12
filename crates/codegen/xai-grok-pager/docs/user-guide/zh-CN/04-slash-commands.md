# 斜杠命令

在提示框里输入 `/` 打开命令菜单。它会随你输入做模糊匹配，选中一条命令会立即执行。

命令来自两处：**shell 内置**，由 agent 后端（xai-grok-shell）处理；以及 **pager 内置**，由 pager 前端（xai-grok-pager）处理。两者出现在同一菜单里，任何启用且带 `user-invocable: true` 的技能也会出现。若技能复用了内置名（例如 `login`），内置仍占用 `/login`，技能以 `/plugin-name:login` 可用——菜单会给两者打标，让冲突可见。

下面每条命令在有别名时都会列出。少数命令仅在某功能或会话状态启用时出现；这些情况会在文中标明。菜单也会按渲染模式过滤——见 [`/minimal` 与 `/fullscreen`](#minimal-and-fullscreen)。

---

## 会话管理

### `/new`

开始全新会话并清空当前对话。别名：`/clear`。

### `/resume`

打开会话选择器，从磁盘重新加载先前会话。

### `/dashboard`

打开 [Agent Dashboard](23-dashboard.md)：本 pager 中顶级会话的实时名册（peek、回复、派发、固定、重命名、停止、附加）。别名：`/agents-dashboard`、`/sessions`。

不是 `/config-agents`（别名 `/agents`），后者管理 agent *定义*和 persona。在极简模式下隐藏；用 `GROK_AGENT_DASHBOARD=0` 或 `[dashboard].enabled = false` 关闭。

### `/compact [context]`

压缩对话历史以回收上下文窗口空间。传入备注可告诉 Grok 要保留什么：

```
/compact
/compact keep the auth implementation details
```

上下文窗口达到 85% 时，Grok 也会自动压缩（用 `[session] auto_compact_threshold_percent` 调整）。

### `/context`

显示上下文窗口的使用情况：按类别分解（系统提示、消息、推理与开销、剩余空间），外加工具定义、技能列表和 MCP 服务器公告的信息行及其估计 token 成本。

### `/session-info`

显示会话详情——认证方法、模型、回合数和上下文用量。别名：`/status`、`/info`。点击值或拖选即可复制；`c` 复制会话 ID，`y` 复制整块。

### `/fork`

把当前会话分支出一个新 agent，保留到此刻为止的历史。

### `/rewind`（别名：`/undo`）

把对话回滚到更早的回合，并丢弃之后的一切。`/undo` 是同一条命令。

### `/copy`

把最近一条回复的源 markdown 复制到剪贴板。传入数字可复制倒数第 N 条回复，或传入文件路径把文本写到文件而不是剪贴板（在 SSH 上很有用，本地剪贴板常常到不了）。

```
/copy
/copy 2
/copy out.txt
/copy 2 ~/exports/last-reply.md
```

每次复制也会写入备份文件——默认是 `~/.grok/last-copy.txt`，若设置了 `GROK_COPY_FILE` 则用它。已确认的复制会短暂 toast（例如 `Copied!`）。未验证的 OSC 52 送达以及剪贴板不可达的回退会点名备份路径，方便你找回文本。

### `/export`

把对话导出到文件或剪贴板。

### `/quit`

退出应用。别名：`/exit`。

### `/home`

离开当前会话并回到欢迎屏。别名：`/welcome`。

### `/delete`

删除当前会话的历史。会先确认。擦除历史前会停止任何正在运行的回合、后台任务和子 agent。回到欢迎屏；若你是从仪表盘打开该会话的，则回到仪表盘。

要删除你不在其中的会话，打开 `/resume` 或欢迎屏会话列表，按 `d` 再按 `y`。在仪表盘上，连按两次 `Ctrl+X` 或点击 `[✗]`。

### `/rename`

重命名当前会话。别名：`/title`。

```
/rename new session title
/rename --auto
```

`--auto` 取消钉住的手工标题，并让自动标题恢复。它只适用于 Build 会话——聊天对话没有本地自动标题器。它必须是唯一参数（`/rename --auto Something` 是错误）。不能通过本命令把会话命名为 `--auto`；那种极端情况请用仪表盘重命名编辑器（`Ctrl+R`）。

---

## 模型与模式

### `/model <name>`

切换模型。接受模型 ID 或显示名（不区分大小写）；对推理模型，可把力度作为第二个参数。别名：`/m`。

```
/model grok-4.6
/model Grok 4.6
/model Reasoning X high
```

### `/effort <level>`

在**当前**模型上设置推理力度，不必重新选择。级别是 `low`、`medium`、`high` 和 `xhigh`，仅在活动模型支持推理力度时生效。

```
/effort high
```

### `/always-approve` 与 `/auto`

两者都是权限模式的真实开关：它们留在菜单里，再运行你已经处于的模式会把它关掉。

| 命令 | 关闭时 | 已经开启时 |
|---|---|---|
| `/always-approve` | 跳过所有权限提示 | 回到询问 |
| `/auto` | 分类器批准安全工具（危险的仍可能提示） | 回到询问 |

在另一个已开启时运行其中一个会切换模式——例如始终批准开启时运行 `/auto` 会切到 auto。`/auto` 仅在自动权限模式功能启用时出现。你也可以用 `Shift+Tab`（循环 Normal / Plan / Auto（已启用时） / Always-approve）、`Ctrl+O` 或 `/settings` 改模式。

### `/multiline`

切换多行输入。开启时，`Enter` 插入换行，`Shift+Enter`（或 `Alt+Enter`）发送消息。回合中，空输入框上的裸 `Enter` 仍会强制发送队列顶部的后续。别名：`/ml`。

### `/history`

打开提示历史搜索：按从新到旧模糊搜索本会话的提示，然后按 `Enter` 或 `Tab` 把匹配项放回提示框。

要快速召回，改为在空提示上按 `↑`。有排队提示时，那会把焦点移到队列窗格并高亮最后一行；否则面板打开并已填入你最近一条提示，`↑`/`↓` 逐步浏览条目（每条落到输入框），越过最新条目按 `↓` 关闭面板，输入则就地编辑召回的提示。

### `/compact-mode`

切换紧凑显示——更少内边距、更紧间距，输出更密。

### `/vim-mode`

切换 vim 风格回看键（`j`/`k`、`h`/`l`、`g`/`G`、`y`/`Y` 等）。关闭时（默认），回看区里的裸字母或 `Shift+letter` 只会聚焦提示框并打出该字符。该设置会持久化到 `[ui] vim_mode`。

### `/edit-prompt`

用外部编辑器打开提示，两种渲染模式都可用。Grok 先解析 `$VISUAL`，再 `$EDITOR`，最后是 `vi`；命令值可以包含带引号的参数。保存会替换草稿但不发送；保存空文件会清空草稿。输入 `/edit-prompt` 必然会替换输入框内容，因此编辑器从空草稿开始；要编辑**已有**草稿，从命令面板选择 **Edit Prompt in External Editor**（或在极简模式下按 `Ctrl+G`），这会保留文本，并拒绝不压扁地处理粘贴、文件引用或图片 chip。

```
/edit-prompt
```

### `/minimal` 与 `/fullscreen`

就地把当前会话切到另一种渲染模式。`/minimal`（你在全屏时提供）切到实验性的回看区原生模式；`/fullscreen`（你在极简时提供；别名 `/full`）切回标准全屏模式。切换发生在正在运行的进程内——什么都不重启，因此进行中的回合继续流式输出，你的输入框草稿、排队提示和权限模式都会带过去；标记（极简里是已提交行，全屏里是 toast）会提醒你如何切回去。两者都是会话范围——不改 `config.toml`——`--minimal` / `--fullscreen` CLI 标志也是同样的会话范围。要让直接运行 `grok` 默认打开某种模式，用 `/settings` → **Default screen mode** 或设置 `[ui] screen_mode`。（若就地切换在奇特终端上行为异常，`GROK_SCREEN_MODE_SWITCH=exec` 会恢复旧行为：把 pager 重新启动到同一会话。）

少数命令只在两种模式之一可用，因为它们驱动的界面在另一种模式里不存在：`/find`、`/jump`、`/timeline`、`/theme`、`/tutorial` 和 `/dashboard` 仅全屏，而 `/expand` 仅极简。（`/workflow runs` 不同：它在全屏打开运行窗格，在极简降级为文本概览，而不是拒绝。）这些命令在无法运行的模式里会从命令菜单和面板隐藏。若你仍把它们打出来，Grok 会说明原因——并指向实际有用的那条。当另一种模式是唯一办法时，那就是模式切换：`/theme isn't available in minimal mode (minimal renders with your terminal's own palette). Run /fullscreen to switch this session.` 当本模式已经用另一种方式完成同一工作时，它会点名那种方式：`/expand isn't available in fullscreen mode: press Tab to focus the scrollback, then → on the block.` 其余命令两种模式都可用。注意 `--no-alt-screen` 在这里仍算全屏，因此它保留仅全屏命令。

### `/plan`

进入计划模式。

```
/plan [description]
```

### `/view-plan`

打开当前已保存计划的预览。别名：`/show-plan`、`/plan-view`。

---

## 记忆

`/flush`、`/dream` 和 `/memory` 需要通过 `GROK_MEMORY=1`、`[memory] enabled = true` 或托管远程设置启用记忆；`/memory` 还需要已配置的记忆后端。`/remember` 始终可用。

### `/memory`

浏览、查看并管理已保存的记忆。传入 `on` 或 `off` 以启用或关闭记忆。别名：`/mem`。

```
/memory
/memory off
```

### `/flush`

立刻把当前会话的知识写入记忆，触发一次对最重要内容的 LLM 摘要。在压缩前用它，或任何你想锁定上下文的时候。

### `/dream`

运行记忆整理——把会话日志合并成有组织的主题。

### `/remember`

立刻把一条备注写入记忆，不必等自动摘要。

```
/remember the staging deploy uses the eu-west cluster
```

---

## Hooks 与插件

`/hooks`、`/plugins`、`/marketplace`、`/skills` 和 `/workflows` 都打开同一个扩展模态框，各自落在自己的标签上。

### `/hooks`

在 Hooks 标签打开扩展模态框，可查看已加载的 hook、添加或移除自定义 hook，并单独开关。该模态框不授予项目信任——信任模型见 [10-hooks.md](10-hooks.md)。

shell 还会公布单独的 `/hooks-list`、`/hooks-trust`、`/hooks-add`、`/hooks-remove` 和 `/hooks-untrust` 命令；在 pager 里这些都折进 `/hooks` 模态框。

### `/plugins`

在 Plugins 标签打开扩展模态框，查看已安装插件、从市场安装新插件，并管理信任。

shell 另外支持子命令（`/plugins list`、`/plugins install <source>`、`/plugins uninstall <name>`、`/plugins update`、`/plugins reload`）。在 pager 里，模态框用可视化方式做同样的事。

### `/marketplace`

在 Marketplace 标签打开扩展模态框，浏览并安装插件。

### `/skills`

在 Skills 标签打开扩展模态框，查看已安装技能。

---

## 媒体生成

### `/imagine <description>`

根据文本描述生成图片。

```
/imagine a golden sunset over a calm ocean with silhouetted palm trees
```

### `/imagine-video <description>`

根据文本（或图片）描述生成视频。它规划镜头、生成源图，并用 `image_to_video` 做成动画。

```
/imagine-video a cat playing piano in a jazz club
```

---

## 调度

### `/loop [interval] <prompt>`

按重复间隔运行一条提示。把间隔写成 `30m`、`1 hour` 或 `every 2 days`；省略则 Grok 会询问。

```
/loop 30m check deploy status
/loop check deploy status every hour
```

间隔是 `Ns`（秒，最少 60）、`Nm`（分钟）、`Nh`（小时）或 `Nd`（天）；低于 60 秒的会抬到最小值。循环任务 7 天后过期，可用创建循环时报告的任务 ID 通过 `scheduler_delete` 取消。

---

## 工作流与目标

### `/goal`

设置、管理或检查自主目标。Grok 跨多轮工作，只有在独立证据审查确认该声明后才把目标标为完成；若审查无法复现结果或没有可用证据，目标保持活动或带着具体缺口暂停。

```
/goal Migrate the auth module to the new API
/goal status
/goal pause
/goal resume
/goal clear
```

参数是 `<objective> [--budget <tokens>]`，或 `status`、`pause`、`resume`、`clear` 之一。这里的 `--budget` 是该目标运行的 **token** 预算，与工作流使用的 agent 数量预算分开。`/goal` 在会话启用目标模式时出现。由谁驱动取决于后台工作流：开启时，宿主评估每一轮模型并在完成候选项上做对抗验证；关闭时，旧的面向模型的 `update_goal` 路径报告进度并触发验证。

### `/deep-research <query>`

启动后台研究工作流。它规划一组有界问题，收集带来源证据的结构化声明，在独立验证分片上交叉核对每条声明，并只渲染存活下来的声明及其已验证来源定位。失败的分片、丢弃的声明和研究者的不确定会作为覆盖限制报告；只要还有剩余，报告就会标为 **Partial**。

```
/deep-research Compare the migration risks of PostgreSQL 17 and MySQL 9
```

该命令立即返回——在 `/workflow runs` 里跟进进度，最终报告会自己出现在对话中。

工作流对逻辑子 agent 调用使用绝对累计的 `agent_budget` 上限：每次 `agent()` 调用以及 `parallel()` 面板中的每一项占用一个名额，架构纠正重试不占用。默认是 128，显式值是 1–1,024，会越过剩余预算的面板会在任何子项启动前被拒绝。模型启动的工作流在 `workflow` 工具上设置 `agent_budget`；具名斜杠启动接受 `--agent-budget N` 或 JSON 参数里的 `agent_budget` 字段。具名启动也可以用 `--effort LEVEL` 或 JSON `effort` 设置子项推理力度，而不改当前会话的 `/effort`；子脚本自己的 `effort` 选项优先。另外，宿主配置的上限（默认 32）限制每次运行同时跑多少子项；更大的面板会排队，并仍作为屏障。`budget()` 把上限报告为 `total`，已接纳调用为 `spent`，`reserved`（始终为零）和 `remaining`。

### `/workflow`

启动已保存的工作流，或按会话唯一显示名管理正在运行的工作流。同一工作流启动两次时，显示名会编号（`review-changes`、`review-changes-2`）；你从不需要内部运行 ID。单独的 `/workflow` 打印本会话运行的文本概览。

输入 `/workflow` 加空格可自动补全已保存的工作流名（内置、项目和用户）以及管理动词 `runs`、`pause`、`resume`、`stop` 和 `save`。选中一个名字会填入并在你加参数前提供启动标志；按 Enter 之前不会启动。`pause` / `resume` / `stop` / `save` 然后列出本会话的运行句柄——单独的 `/workflow stop` 不会选中一次运行。

```
/workflow review-changes --agent-budget 256 --effort high {"target":"origin/main...HEAD"}
/workflow review-changes {"target":"origin/main...HEAD","agent_budget":256,"effort":"high"}
/workflow runs
/workflow pause review-changes
/workflow resume review-changes
/workflow stop review-changes-2
/workflow save review-changes
```

`/workflow runs` 在全屏 TUI 中打开实时 **Workflow Runs** 仪表盘——活动与保留的运行，不是已保存定义的目录。每一行显示运行的显示名、阶段、agent 名册、进度和结果。在运行详情里，`p` 暂停，`r` 恢复普通暂停，`x` 停止。受预算限制的运行不能裸恢复：`r` 返回 shell 的拒绝（用传入更高 `agent_budget` 的模型/工具恢复来提高上限），而 `x` 仍会停止。`s` 保存该运行的脚本，但对已知内置和编号重复句柄是隐藏的——对这些，选一个新的唯一 `meta.name` 并显式保存编辑后的脚本。在极简模式和非 TUI 客户端中，`/workflow runs` 打印与单独 `/workflow` 相同的文本概览。

项目工作流位于 `.grok/workflows/*.rhai`；用户工作流位于 `~/.grok/workflows/*.rhai`。同进程暂停/恢复会从已提交的宿主调用结果继续原始不可变脚本、参数和 `agent_budget` 上限——要迭代，编辑返回的脚本副本并作为新运行启动。

受预算限制的运行不同：它只通过提供高于已接纳 agent 数的 `agent_budget` 的模型/工具恢复请求来恢复。单独的 `/workflow resume <name>` 无法提高上限，因此会拒绝受预算限制的运行。被进程重启打断的运行完全不会恢复，因为外部效果没有稳定的跨进程身份。而且恢复不是恰好一次：同进程暂停前结果未提交的外部效果可能再跑一次。

### `/workflows`

在 **Workflows** 标签打开扩展模态框——Grok 发现的已保存工作流的只读目录（内置、项目 `.grok/workflows/` 和用户 `~/.grok/workflows/`），带每条的来源、描述和路径。同一目录也会在会话前导的技能列表下提供给模型。用 `/workflow <name>`（或它自己的斜杠命令）启动一个，然后在 `/workflow runs` 里观看。

---

## 其他

### `/theme`

切换颜色主题。别名：`/t`。

### `/feedback [message]`

报告问题或发送反馈。打开报告面板：`Enter` 发送，`Esc` 丢弃。消息会预填面板，便于发送前编辑。在 `--minimal` 中，消息仍会立即发送。

```
/feedback
/feedback Something isn't working correctly
```

### `/btw`

向 agent 发送旁白，不打断当前任务。在极简模式（`--minimal`）下，答案出现在提示框上方可关掉的面板：`Esc` 关掉它，完成的答案会存入原生回看区，已关掉面板的迟到回复会被丢弃。旁问及其答案不是主回合的一部分。

```
/btw also check the error handling
```

### `/mcps`

打开 MCP 服务器管理模态框。

### `/doctor`

检查当前会话的终端、剪贴板、颜色、输入、通知和沙箱问题。Doctor 显示它发现了什么以及如何解决每个问题。运行 `/doctor fix` 列出可用的自动修复；其他发现包含手工步骤。`/terminal-setup`、`/terminal-check` 和 `/terminal-info` 仍是别名。

### `/release-notes`

查看当前版本的发行说明。别名：`/changelog`。

### `/docs`

浏览内置 How-to Guides、打开在线 Build 文档，或按标题直接跳到某篇指南。别名：`/howto`、`/guides`。

```
/docs
/docs web
/docs Getting Started
```

- 单独的 `/docs`（或 `/docs how-to`）打开 How-to Guides 选择器。
- `/docs web` 在浏览器中打开 https://docs.x.ai/build/overview。
- `/docs <title>` 按不区分大小写的标题匹配打开指定指南。

### `/tutorial`

打开入门教程：一组短主题（你的第一条提示、附加上下文、导航、斜杠命令、工作树、计划模式、自定义、从其他 agent 工具切换）——每篇大约 30 秒可读完，`→` 直接流到下一主题。什么都不会自动弹出——本命令（或命令面板）才是入口。

```
/tutorial
```

别名：`/tour`、`/onboarding`

### `/import-claude`

打开 Claude 导入模态框，带入 `~/.claude` 设置：权限、环境变量、MCP 服务器、hooks 和路径。

---

## Agent 与 Persona

### `/config-agents`

打开 agents 模态框，查看并管理 agent 定义、设置默认项，并切换活动项。别名：`/agents`。

不是实时多会话 [Agent Dashboard](23-dashboard.md)（`/dashboard` / `Ctrl+\`）。

### `/personas`

创建、编辑和删除 persona。子 agent 可以应用 persona 来塑造其行为。

---

## 账户与账单

### `/login`

在不离开会话的情况下登录或重新认证。

### `/logout`

登出并回到登录屏。

### `/usage`

查看额度用量或管理账单。别名：`/cost`。

```
/usage
/usage manage
```

在会话内这会打开用量模态框，含账户额度以及该会话的上下文和 token 总计。从 [Agent Dashboard](23-dashboard.md#dispatch-input) 打开时，同一模态框叠在仪表盘上；那里没有会话，因此只有 **Usage limit** 标签有数据。

要查看任意本地会话已持久化的每回合 token 和费用总计，从 shell 使用 `grok usage <session-id> [turn]`。见 [会话管理](17-sessions.md#the-grok-usage-subcommand)。

### `/privacy`

在 **Coding data, retention, and training** 打开设置，在那里选择
**Opt in** 或 **Opt out**。不接受参数。

```
/privacy
```

此设置不触及 `[features] telemetry`、`trace_upload` 或你的外部 OTEL 设置——见 [用量监控](24-monitoring-usage.md#related-settings)。在团队账户上只有团队管理员能改，管理员也可以为团队启用或关闭 Zero Data Retention（[how to enable ZDR](https://docs.x.ai/developers/faq/security#how-to-enable-zdr)）。当选择权不在你时，该行会这样说——`ZDR` 或 `· Admin Managed`——而不是打开选择器。ZDR 锁定编码数据共享；它不会静音外部 OTEL 或 `user.email`——见 [ZDR and this stream](24-monitoring-usage.md#zdr-and-this-stream)。

---

## 配置与 UI

### `/settings`

打开设置模态框，以交互方式查看和更改配置。别名：`/config`、`/preferences`、`/prefs`。

### `/timestamps`

打开或关闭消息时间戳。

---

## 作为斜杠命令的技能

任何在 SKILL.md frontmatter 中带 `user-invocable: true` 的已启用技能都会显示为斜杠命令。（通过 `/skills` 关掉技能后，它就不再被公布。）因此位于 `~/.grok/skills/commit/SKILL.md` 的技能会这样运行：

```
/commit fix typo in README
```

来自插件的技能同样如此。当两个技能在不同范围同名时，加上限定：

```
/local:commit      # 项目范围技能
/user:commit       # 用户范围技能
```

内置命令始终赢得裸名。把技能命名为 "compact" 时，`/compact` 仍运行内置——技能以 `/local:compact`（或插件的 `/acme:compact`）可用。两者都出现在斜杠菜单：内置标为 `built-in`，技能标为 `skill · local` / `skill · acme`。

---

## 自动补全

菜单支持模糊搜索：在 `/` 后开始输入即可过滤。每条显示命令名、描述、需要参数时的参数提示，以及来源（builtin、技能范围或插件名）。按 `Tab` 或 `Enter` 接受高亮的命令。
