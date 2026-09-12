# Agent 仪表盘

Agent Dashboard 列出本 pager 进程里每一个顶层会话——本地会话和派生——并按状态分组。在同一屏幕上你可以 peek、回复、附着、钉住、重命名、停止，或派发新 agent。子 agent 不列出；它们跑在父会话下，父会话已经会显示工作进行中。

它不是 agents 模态框（`/config-agents` / `/agents`——定义和人设）、会话选择器（`/resume` / `Ctrl+R`，磁盘上的过往对话），也不是工作流运行 UI（`/workflow runs`）。

---

## 打开仪表盘

- **`grok dashboard`** — 把 TUI 启动到仪表盘。
- **`/dashboard`**（别名 **`/agents-dashboard`**、**`/sessions`**）— 从会话内打开。
- **`Ctrl+\`** — 与斜杠命令相同的视图。

在极简模式里隐藏。设 `GROK_AGENT_DASHBOARD=0` 或 `[dashboard].enabled = false` 可禁用。

---

## 你会看到什么

```
  main ~/xai [Choose Ctrl+l]                    ◆ 2 awaiting │ ⋮ 1 working │ ◇ 1 idle

  + New Agent                                Open Previous /resume │ Worktree Ctrl+w

▌● reviewer · audit token flow    Awaiting your input            2m
 ● implementer · fix login bug    Running: cargo test           12m
 ⋅ refactor · feat/login          Responding…                   24m
 ○ housekeeping                   idle                           1h
 ● implementer · add login tests  8 tools · 1.2k tok            14m
╭─────────────────────────────────────────────────────────────────╮
│ ❯ Dispatch a new agent                                          │
╰─ dispatch ──────────────────────────────────────────────────────╯
 ↑/↓ select (peek) · Enter open · Ctrl+R rename · Ctrl+T pin · Ctrl+X stop · ? help · Esc new
```

**页眉**显示新 agent 将在哪里运行——git 分支和工作目录——右侧是状态计数芯片（与各行相同的字形和颜色，外加标签）。点击位置（或按 `Ctrl+L`）可**选择**另一个目录；从派发框里 `/cd <path>` 做同样的事。

它下面的**操作行**放着 `+ New Agent`（没有选中行时的默认光标目标），右侧是 `Open Previous`（会话选择器；仅工作区仪表盘）和**工作树开关**。列表聚焦时（`Tab`），`→` / `←`（或 vim 模式的 `l` / `h`）按该顺序沿行移动光标，停在两端；`Enter` 像点击聚焦项一样——创建、打开选择器，或切换工作树模式——`Esc` 退回到 `+ New Agent`。这些操作始终也可以点击或用 `/resume` / `Ctrl+W`。在 git 仓库内打开工作树模式时，该行读作 `+ New Agent in Worktree` / `Disable Worktree`，下一次派发会在全新 git 工作树里创建 agent。

每一行都是一个顶层 agent。按状态排序（Needs input → Working → Idle → Inactive → Completed → Failed），让同状态的行坐在一起，或按工作目录排序（`Ctrl+G` 切换）。**Inactive** 是其他 pager 进程拥有、本进程尚未加载的仅花名册会话——后台噪音，因此该段**一开始是折叠的**（用 `→` / 点击展开）。

为了让 **Idle** 好扫，只有最近的空闲 agent 保持可见——最新的 8 个，外加过去一小时内活动过的。其余折进该组底部的 **"N more"** 行；选中它并按 `Enter` / `→`（或点击）展开，`←` 再折起。Idle 页眉始终显示真实总数。过滤或搜索活动时折叠会暂停。

状态图标与 Grok Build 里其他会话列表一致：

- `⋅`/`:`/`⸬`/`⁙` — **Working** 的动画转圈
- `●` — **Needs input**、**Completed**、**Failed**、**Blocked** 的实心圆（颜色：黄 / 绿 / 红 / 琥珀）
- `○` — **Idle** 和 **Inactive** 的空心圆

只要还有活着的后台工作，即使回合已经结束，一行仍保持 **Working**——后台任务、`monitor`，或活动的已调度 `/loop`。活动行会说明仍在跑什么（例如 `1 monitor · 2 loops still running`）。

没有行内分组页眉；排序让同状态的行相邻，每行的点 + 颜色显示所属组。

派发输入使用与 agent 视图相同的提示框外观。按 `Ctrl+/` 把它翻成**搜索模式**：`❯` 前缀变成黄色的 `Search:`，输入会实时过滤列表而不是派发。

---

## 快捷键

| 键 | 动作 |
| --- | --- |
| `↑` / `↓`、`j` / `k` | 在行和段标题间导航（选中一行会打开 peek） |
| `→` / `←`（在段标题上） | 展开 / 折叠该段（vim 模式用 `l` / `h`） |
| `Enter`（在段标题上） | 切换该段折叠 / 展开 |
| `Enter`（空回复） | 全屏打开选中的 agent（详情视图） |
| `Ctrl+S` | 发送 peek 回复并打开该 agent（或派发并附着新会话） |
| `Shift+Enter` / `Alt+Enter` | 在回复 / 派发输入里换行 |
| `1`–`9` | peek 显示选项时回答待处理的权限 / ask 问题 |
| `Enter`（已输入回复） | 向选中的 agent 发送 / 排队回复 |
| `/` | 把字面 `/` 送进提示框 |
| `Ctrl+/` | 切换搜索模式（实时过滤行） |
| `Ctrl+R` | 重命名选中行 |
| `Ctrl+T` | 钉住 / 取消钉住 |
| `Ctrl+G` | 切换分组（状态 ↔ 目录） |
| `Ctrl+X` | 取消正在运行的回合，或 2 秒内按两次以永久删除 |
| 悬停并点击 `[✗]` | 永久删除空闲/已完成的行（再点一次确认） |
| `Shift+↑` / `Shift+↓` | 重排已钉住的行 |
| `Esc` | 后退：取消搜索 → 关闭 peek → 清除过滤 → 取消聚焦派发 → 取消选中行 → 退出。从不清除已输入的派发草稿（那要用 `Ctrl+U` / `Ctrl+C`） |
| `Ctrl+\` | 从详情视图返回，或退出仪表盘 |
| `Ctrl+.`（备选：`?`） | 键盘快捷键速查。当 `Ctrl+.` 无法送达时页脚显示 `?`。列表聚焦或草稿为空时，光秃的 `?` 打开帮助 |

按状态分组时，每组有一个**段标题**（例如 `Working`、`Idle`），带 `▸`/`▾` 标记。选中标题并按 `→` / `←` 展开或折叠（vim 模式用 `l` / `h`）。点击切换；悬停会变亮。仪表盘保持打开期间会记住折叠状态。**Inactive** 每次 pager 启动时都从折叠开始；展开后会粘住直到你退出。

打开一行会在**详情视图**里显示该 agent 的对话：顶部页眉（agent 名；右侧是 `{i}/{n}` 循环芯片和 `[Dashboard]`）下方是全宽对话——没有带边框的模态框——因此内边距与列表视图一致。按键交给附着的 agent；`Esc` / `Ctrl+\`（或 `[Dashboard]`）返回仪表盘；`[‹]` / `[›]` 循环 agent。快捷键栏显示 `Ctrl+\: back to dashboard`。注意：`Esc` 只返回；在 agent 里 `/exit` 会关闭会话（仪表盘 toast："Session closed"）。

详情视图里的 `Ctrl+X` 取决于状态。**回合正在运行**时它取消该回合（与 `Ctrl+C` 相同，包括保留子 agent 的提示），从不关闭会话。否则——**空闲**、斜杠命令进行中，或取消仍待处理——`Ctrl+X` 武装确认：2 秒内再按一次以关闭会话并返回仪表盘。任何其他键取消确认；窗口内开始的回合会把已确认的按键变成取消。（若你的终端上 `Ctrl+X` 也是速查绑定，在详情视图里用 `Ctrl+.`。）

见 [键盘快捷键](03-keyboard-shortcuts.md#agent-dashboard)。

---

## 完成或关闭会话

**没有**「标记为已完成」命令。行状态从 agent 推导：

- 工作自行结束时为 **Completed** / **Failed**（回合已结束且没有后台任务 / monitor / `/loop` 仍在跑）。
- 回合运行时 **`Ctrl+X` 一次** 取消该回合。
- **`Ctrl+X` 两次**（2 秒内）**永久删除**会话（与 `/delete` 相同）。悬停空闲/已完成的行，把年龄换成 `[✗]`，点两次确认。
- 在详情视图里，`/exit` 也会关闭会话（Esc 只返回）。在附着的 agent 里 `/delete` 会抹掉该会话并返回仪表盘。

没有手动完成旗标。用 `/exit` 离开会话而不删除历史。

---

## 派发输入

底部文本区**始终派生新会话**。选中的行是导航光标，不是回复目标——打开 agent 才能和它说话。

- 自由文本 → 用该提示播种的新顶层会话。文本从不被当作过滤器（即使以 `/`、`s:`、`a:` 或 `#` 开头）；过滤是 `Ctrl+/` 搜索模式。前导 `/` 运行 pager 全局斜杠命令。
- 空输入 → 打开选中行，或在聚焦 `+ New Agent` 时创建新 agent。

`/usage` 在仪表盘上打开用量模态框。仪表盘没有会话，因此 **Usage limit** 标签显示你的账户额度，两个会话标签读作 "No active session"；打开一个 agent 可看它的上下文和 token 合计（`/context` 和 `/session-info` 只在会话内有效）。`Esc` 或 `[✗]` 关闭它。

输入后 `Ctrl+S` 会派发**并**附着；普通 `Enter` 留在仪表盘上，以便你派发多个会话。`Shift+Enter` / `Alt+Enter` 插入换行；框随草稿增高（到达上限后滚动）。

空或仅空白的提示会被忽略。超过 64 KiB 的提示会拒绝并弹出 toast。

### 焦点：输入栏 ↔ 总览列表（`Tab`）

两个焦点区：**派发输入**和**总览列表**。`Tab` 在它们之间切换；未激活的输入会变暗边框并隐藏插入符。

打开时，若至少有一个 agent，焦点默认在**总览列表**（以便 `↑`/`↓` / vim 的 `j`/`k` 立刻导航）。**没有** agent 时，焦点留在**派发输入**。无论哪种，光标都从 `+ New Agent` 开始（不预先选中 agent 行）。

- **输入聚焦**：输入新会话提示。空提示：`↑`/`↓` 在行间导航；非空：移动插入符。`Esc` 取消聚焦到列表（草稿保留）。
- **总览聚焦**：`↑`/`↓`（以及 vim 的 `j`/`k`）在行间移动。`Enter` 打开高亮的 agent（在 `+ New Agent` 上，发送已输入的草稿或创建新会话）。`Esc` 留在列表上并后退——清除过滤，然后取消选中（→ `+ New Agent`），然后退出。`Tab`、`i`（vim）或任何可打印键回到输入。

---

## Peek 面板

选中一行 agent 会用 **peek 面板** 取代派发框。没有选中行时（`+ New Agent`，或 `Esc` 之后），派发框回来。选中一行与已有 agent 说话；取消选中以开始新的。

从上到下：页眉（**上次响应类型**——`Thinking` / `Thought` / `Response` / `Edit` / `Read` / `Bash` / …——以及**时间**）、最近一次响应（按词换行，最多约 3 行；截断时为 `…`），以及实时的 `❯ reply` 输入。

选中 agent 的**模型**，以及始终批准（yolo）模式下的 **`always-approve`** 旗标，坐在面板底边上（与派发框同一徽章槽），包括在回答问题时。列表行不再重复模型或始终批准徽章。

**`Shift+Tab` 循环被 peek 的 agent 的模式**（普通 → 计划 → Auto（启用时）→ 始终批准 → 普通），作用在**活着的** agent 上。在派发框上，Shift+Tab 只为*下一个* agent 暂存模式。

与派发（只新建会话）不同，peek 回复**与选中的 agent 说话**：

- **在 `❯ reply` 里输入，然后按 `Enter`** 发送。空闲 agent 立刻开始；忙碌 agent **排队**该消息（与 agent 视图提示框相同）。`Ctrl+S` 回复并打开详情视图；`Shift+Enter` / `Alt+Enter` 插入换行（回复随草稿增高）。
- 空回复 + `Enter` 打开该 agent。
- 回复有内容后，**`↑`/`↓` 移动插入符**。为空时（或通过 `Tab` 取消聚焦时），`↑`/`↓` **切换选中的 agent**——面板跟着走，半写的草稿会被清掉，以免落到错误的 agent 上。（草稿还在回复里时，用 `Tab` 到列表来导航。）
- **`Esc` 取消选中**：先清除已输入的回复，然后取消选中并聚焦 `+ New Agent`。
- **`Tab`** 在回复和行列表之间切换焦点；可打印键重新聚焦回复。
- 完整提示编辑器（与派发 / agent 提示框相同）：多行粘贴芯片、鼠标选择、按词导航、`Ctrl+A`/`Ctrl+E`、`Alt+Backspace`、`Ctrl+W`/`Ctrl+U`/`Ctrl+K`、撤销、Shift+方向键选择、`Ctrl+Shift+V` 行内粘贴。**`@`** 打开根在**被 peek 的 agent** 工作目录的文件选择器；下拉浮在面板上方。仪表盘和弦（`Ctrl+X` 停止、`Ctrl+T` 钉住、`Shift+↑/↓` 重排，…）在面板打开时仍优先。
- 待处理的 **权限 / ask-tool** 问题：`❯ reply` 隐藏；改为选项列表。**`↑`/`↓` 高亮**，**`Enter` 回答**，**`1`–`9`** 直接回答。自由文本 **No / reject** 和 ask-tool **Other** 在自由文本行上接受输入的回答。多问题 Ask 表单一题一题走（`(i/N)`）；多选表单需要 agent 自己的视图。

在非常矮的终端上面板可能放不下；即使选中了行，派发框仍会留下。

---

## 搜索 / 过滤（`Ctrl+/`）

`Ctrl+/` 切换搜索模式，让普通输入始终派发。前缀从 `❯` 翻成黄色的 `Search:`；每个按键都会实时过滤列表。

- `Enter` — 确认：保留过滤并回到派发提示。
- `Esc` 或 `Ctrl+/` — 取消：清除过滤并退出搜索。
- `↑` / `↓` — 在过滤后的行间导航。

前缀（仅在搜索模式内）：

- `a:<name>` — agent 标签（不区分大小写的子串；人设 / 角色）。
- `s:<state>` — 行状态：`working`、`idle`、`completed`、`failed`、`needs-input`、`blocked` 以及同义词（`busy`/`running`/`done` 等）。
- `#<text>` — 对 `#<text>` 的子串匹配（标签里的字面 `#`）。
- 其他任何内容 — 对标签 + 工作目录的子串。

---

## 持久化

按用户的偏好在 `~/.grok/config.toml` 的 `[dashboard]` 下：

```toml
[dashboard]
enabled = true
grouping = "state"   # or "directory"
pinned   = ["top:<session_id>", "sub:<parent_session_id>:<child_session_id>"]
reorder  = ["top:<session_id>"]
```

钉住/重排条目使用**会话 id**（不是按进程的 agent 槽），因此能在重启后存活。
