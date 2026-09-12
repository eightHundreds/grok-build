# 键盘快捷键

Grok Build TUI 的按键绑定参考。绑定是内置的，目前无法重映射。

---

## 输入模式

Grok 有两种输入模式，控制你如何在回看区导航：

- **简易模式**（默认）：方向键导航，`Shift+Arrow` 在回合间跳转，`Space` 聚焦提示框，任意字母键会自动聚焦提示框。
- **Vim 模式**（需开启）：`j`/`k` 导航，`H`/`L` 在选中回合同跳转，`J`/`K` 跳到视口顶部的回合（与时间线箭头相同），`h`/`l` 折叠，`e`/`E` 展开/收起，`i`/`Tab`/`Space` 聚焦提示框。

默认是简易模式。要切换到 Vim 模式，在 `~/.grok/config.toml` 的 `[ui]` 下设置 `vim_mode = true`，或在运行时用 `/vim-mode` 切换。详见 [配置](05-configuration.md)。

下表记录两种模式的绑定。「Key」列是 Vim 模式绑定，「Alt Key」列是简易模式的等价按键（方向键等）。

> **需要 Vim 模式**：回看区上下文中的单字母和 `Shift+letter` 绑定
> （`j/k`、`h/l`、`g/G`、`L/H`、`y/Y`、`o/O`、`r`、
> `x`、`e/E`，以及 `i` 插入模式替代）需要在
> `~/.grok/config.toml` 里设 `[ui].vim_mode = true`
> （或用 `/vim-mode` 切换）。方向键、`Tab`、
> `Esc`、`Space`、`PageUp/Down` 以及所有 `Ctrl+letter` 快捷键在
> 两种模式下都可用。

---

## 导航（回看区聚焦时）

在回看区窗格中移动对话条目。

| 按键 | 备用按键 | 动作 |
|-----|---------|--------|
| `j` | `Down` | 选中下一条 |
| `k` | `Up` | 选中上一条 |
| `⇧L` | `Shift+Right` | 跳到下一回合（用户提示） |
| `⇧H` | `Shift+Left` | 跳到上一回合（用户提示） |
| `⇧J` | | 跳到视口顶部的下一回合 |
| `⇧K` | | 跳到视口顶部的上一回合 |
| `g` | | 跳到回看区顶部 |
| `⇧G` | | 跳到回看区底部 |
| `Ctrl+K` | | 向上滚动一行（不改变选中） |
| `Ctrl+J` | | 向下滚动一行（不改变选中） |
| `PageUp` | | 向上滚动一页（选中移到视口顶部） |
| `PageDown` | | 向下滚动一页（选中移到视口底部） |
| `Ctrl+U` | | 向上滚动半页 |
| `Ctrl+D`（VSCode 中为 `Shift+D`） | | 向下滚动半页 |

`PageUp` 和 `PageDown` 在普通提示框聚焦时也会滚动对话，
不移动焦点、不改草稿。活动的提示历史、
`@` 文件搜索、斜杠菜单或补全下拉会自己占用这些键
做导航。

---

## 视图（回看区聚焦时）

控制回看区中条目的显示方式。

| 按键 | 备用按键 | 动作 |
|-----|---------|--------|
| `h` | `Left` | 折叠选中条目 |
| `l` | `Right` | 展开选中条目 |
| `e` | | 切换选中条目的折叠 |
| `⇧E` | | 展开全部 / 折叠全部条目 |
| `Ctrl+E` | | 展开/折叠所有思考块 |
| `r` | | 切换选中条目的原始 markdown |

在 `pager.toml` 的 `[scrollback.scroll]` 下设置
`respect_manual_folds = true`
（需开启，默认关闭——见
[配置](05-configuration.md)）会钉住你亲手折叠的块：
流式更新和结束事件（例如思考块结束）
不会重置它；在自动滚动跟随尾部时展开一块
会停止跟随，方便阅读；用 `⇧G`、在最后一条上按 `j`、
滚过底部，或发送新提示即可恢复。`⇧E` 清除所有钉住，`Ctrl+E` 清除思考块上的钉住。

### 块内容

| 按键 | 动作 |
|-----|--------|
| `y` | 把块内容复制到剪贴板 |
| `⇧Y` | 把块元数据（例如 shell 命令）复制到剪贴板 |
| `Enter` | 在全屏查看器中打开块内容 |
| `Ctrl+F` | 在全屏查看器中打开块内容（备用绑定） |

---

## 焦点

在提示输入和回看区窗格之间切换。

| 按键 | 备用按键 | 上下文 | 动作 |
|-----|---------|---------|--------|
| `Tab` | `Space`（Vim 模式下还有 `i`） | 回看区聚焦 | 聚焦提示输入 |
| `Tab` | | 提示框聚焦 | 聚焦回看区（简易和 Vim 回看模式都如此） |
| `Tab` | `Shift+Tab`（反向） | 阻塞卡片聚焦（问题、权限提示、取消回合面板） | 在该卡片的行间走动，两端环绕。焦点留在卡片内 |
| `Tab` | `Space`（Vim 模式下还有 `i`） | 回看区聚焦且有卡片停靠 | 把键盘交回该卡片（栏上的焦点提示会点名它） |
| `Enter` | | 提示框聚焦 | 发送当前提示 |

**Esc 不是焦点键。** 它遵循下面的清空 / rewind 语义，并且从不取消正在运行的回合（那是 `Ctrl+C` 的事）。Esc 的行为不依赖 `[ui].vim_mode`（回看导航）或 `[ui].simple_mode`（提示编辑器）。覆盖层、模态框、斜杠/文件下拉、语音、搜索和选区仍会先抢走 Esc。

## 阻塞卡片

有四种界面会阻塞 agent 等你的回答，并在打开时接管键盘：
**问题卡片**（`ask_user_question`）、**MCP
elicitation 卡片**（`x.ai/mcp/elicit`）、**权限提示**，以及
**取消回合面板**。同时打开多个时，权限提示先拿键盘，
然后是取消回合面板，再是问题，再是 elicitation——
快捷键栏始终显示当前接收按键的那一个的键。

它们共用一套约定：

- `Tab` / `Shift+Tab` 在该卡片的行间走动，两端环绕。它们从不
  把焦点移出卡片，因此光标始终在你看得见的地方。
- `Esc` 一级一级往回退：先清掉卡片上尚未处理的内容，
  只有再也没有可清的才会离开。
  离开到哪里是各卡片唯一的不同——问题卡片
  和权限提示会把键盘停靠在回看区，方便你
  往上滚、读背后的上下文（卡片仍留在屏幕上），而
  取消回合面板的「继续运行」会关闭面板并让
  回合（以及任何子 agent）继续跑。Enter 或 `1`–`4` 仍可选择
  取消与子 agent 的选项。
- 键盘停靠时，快捷键栏显示回看区自己的键，
  其焦点提示点名的是卡片而不是提示框：`Tab/Space:
  question`。该提示是钉住的，窄栏也不会裁掉
  唯一的返回路径。
- 在仪表盘的会话覆盖层里还有多一档：键盘停靠后，
  下一次 `Esc` 回到仪表盘，卡片保持待处理。
  （`Ctrl+\` 仍可从任意状态离开。）

### MCP elicitation 卡片（`x.ai/mcp/elicit`）

当 MCP 服务器请求用户输入（表单字段或 URL 同意）时显示。
标题始终包含 MCP 服务器名称。

| 按键 | 动作 |
|-----|--------|
| `↑` / `↓`、`j` / `k`、`Tab` | 在字段 / 操作之间移动 |
| 在字段上按 `Space` / `Enter` | 编辑文本，或切换布尔 / 枚举 |
| 在操作上按 `←` / `→` | Accept 与 Decline |
| 在 Accept 上按 `Enter` | 提交（表单先校验；URL 会打开浏览器） |
| `d` / Decline | 拒绝该请求 |
| `Esc` | 往回退：先离开文本编辑，再把焦点停靠到回看区（`Tab` 返回）。只有在等待已接受的 URL 时才会关掉卡片 |
| `Ctrl+C` | 取消该请求 |
| `o` | 等待完成时重新打开 URL |

### 问题卡片（`ask_user_question`）

| 按键 | 动作 |
|-----|--------|
| `↑` / `↓`、`j` / `k` | 在答案之间移动（两端夹住） |
| `Tab` / `Shift+Tab` | 在本题的答案间循环走动——从最后一个回到第一个。从不带你进入另一题 |
| `←` / `→`、`h` / `l`、`[` / `]` | 上一题 / 下一题 |
| `1`–`9`、`a`–`f` | 直接选中该答案 |
| `z` | 跳到自由文本行并开始输入 |
| `Space` | 切换聚焦的答案（多选），或在自由文本行开始输入 |
| `Enter` | 选中并前进，在最后一题提交，或编辑自由文本行 |
| `Esc` | 取消本题已选答案；没有选中时，把焦点停靠到回看区（`Tab` 返回）。在仪表盘会话覆盖层的*第一*题上，会改回仪表盘——从后面的题仍用 `←` 回去，因此先停靠，下一次 `Esc` 才离开。快捷键栏会点名当前所在档 |
| `y` | 复制聚焦的答案 |
| `Shift+X` | 关掉该问题（agent 在没有答案的情况下继续） |
| `Ctrl+F` | 全屏显示该卡片 |

`/feedback` 面板是本表的唯一例外：报告框没有
可走的答案，`Enter` 发送报告，`Esc` 关掉面板。当可以
提供 trace 上传时，在报告上按 `Enter` 会先显示上传
问题（`↑`/`↓` 选择，`Enter` 按你的选择发送，`Esc` 跳过
上传并仍发送报告）。

输入自由文本答案时，`Enter` 提交，`Esc` 回到
答案行；其他键都进文本字段。

### 权限提示

| 按键 | 动作 |
|-----|--------|
| `↑` / `↓`、`j` / `k` | 在选项之间移动（两端夹住） |
| `Tab` / `Shift+Tab` | 在选项间循环走动 |
| `1`–`9` | 直接选择该选项 |
| `Enter` | 选择聚焦的选项 |
| `←` / `→` | 加宽 / 收窄「始终」答案会记住的范围 |
| `e` | 手工编辑始终允许的模式（bash 提示） |
| `Ctrl+F` | 展开 / 折叠完整参数 |
| `Ctrl+O` | 打开始终批准模式 |
| `Esc` | 把焦点停靠到回看区（`Tab` 返回）。从不回答或关掉该请求 |
| `Ctrl+C` | 取消该请求 |

在「No」行上开始输入会改为给 agent 回一条消息；`Enter`
发送，`Esc` 回到选项。

### 取消回合面板

| 按键 | 动作 |
|-----|--------|
| `↑` / `↓`、`j` / `k`、`Tab` / `Shift+Tab` | 在选项之间移动 |
| `1`–`4`、`Enter` | 确认该选项 |
| `Esc` | 让一切继续跑。这会结束面板，因此从不是死胡同，也不需要停靠 |

## Escape

| 状态 | 手势 | 效果 |
|--------|---------|--------|
| 回合进行中（所有模式和窗格） | `Esc` | **不**取消。显示 “Press Ctrl+c to cancel the turn” toast；草稿不动。用 `Ctrl+C`（或面板 / 其他取消入口）。 |
| 回合正在取消 | `Esc` | 吞掉，无操作。此状态下 `Ctrl+C` 会升级为退出。 |
| 空闲 + 非空提示（文本或图片 chip），**提示框聚焦** | **800ms 内 2× `Esc`** | 清空提示框；被清的草稿会 stash（`Ctrl+S` 或 `Alt+S` 恢复，含图片），其文本在 `↑` 历史浏览中排第一。第一次按显示 “press again to clear”。 |
| 空闲 + 空提示 + 已有对话消息，**提示框或回看区聚焦** | **800ms 内 2× `Esc`** | 打开 rewind 选择器（与 `/rewind` 相同）。第一次按无声（无 toast）。 |
| 空闲 + 空 + 无消息，**或回看区聚焦且有草稿 / 带模式（`!` `#`）的输入框 / 待处理的 needs-input 覆盖层 / 打开的历史搜索** | `Esc` | 吞掉，无操作（不聚焦回看区）。清空只发生在提示窗格；rewind 需要空的 Normal 模式输入框、没有待处理覆盖层、也没有打开的历史搜索。阅读回看区从不改动你的草稿、输入框模式、等待回答的问题，或进行中的搜索。 |

**回合中 Esc 宽限：** 回合中按 Esc 后大约一秒内，空闲 rewind 臂会保持抑制——在即将结束的回合上连按 Esc，不会悄悄打开 rewind 选择器。只有 rewind 臂被按住；其他 Esc 行为不受影响。

**抢 Esc（在回合中提示和清空 / rewind 之前运行）：** 覆盖层、模态框、斜杠/文件/补全下拉、历史搜索、回看搜索、文本选区、链接高亮、语音，以及提示为空时的 **Bash / Remember 模式退出**（Esc 离开 `!` / `#` 模式并回到普通提示，即使回合正在进行）。单独的 `/feedback` 打开报告面板；Esc 关掉它。

**Ctrl+C 与 Esc：** 回合进行中且草稿非空时，Ctrl+C 清空草稿并保持回合；在空提示上再按一次 Ctrl+C 才取消。Esc 从不取消：回合中它只指向 Ctrl+C，并留下草稿。空闲且非空时，Ctrl+C 一次就清空；Esc 需要 800ms 内按两次。两种清空留下的东西不同：`Esc Esc` 会 stash 草稿，因此 `Ctrl+S` 能拿回来，而 `Ctrl+C` 会丢掉它（文本仍在 `↑` 历史里）。

---

## Agent 级

影响 agent 会话的操作，可从 agent 屏幕使用。

| 按键 | 上下文 | 动作 |
|-----|---------|--------|
| `Ctrl+P` | Agent 屏幕 | 打开命令面板 |
| `?`（Shift+/） | Agent 屏幕 | 打开命令面板（备用绑定） |
| `Ctrl+M` | Agent 屏幕 | 打开模型选择器 / 切换模型 |
| `Ctrl+M` | 提示框聚焦 | 切换多行输入模式 |
| `Ctrl+C` | Agent 屏幕 | 取消当前回合（或先清空非空草稿；见 Escape 表） |
| `Ctrl+O` | Agent 屏幕 | 切换始终批准（YOLO）模式 |
| `Ctrl+R` | Agent 屏幕 | 打开会话选择器（恢复先前会话，与 `/resume` 相同） |
| `Ctrl+;`（备用：`Ctrl+'`） | Agent 屏幕 | 切换提示队列窗格（非空时）。**仅本地 macOS** VS Code 家族：主绑定是 **`Ctrl+4`**（`;` / `'` 仍是备用）。SSH 和非 Mac 仍用 **`Ctrl+;`** / **`Ctrl+'`**。 |
| `Shift+Tab` | 提示框聚焦 | 循环模式（Normal → Plan → Auto（已启用时） → Always-approve） |
| `Ctrl+B` | Agent 屏幕 | 把正在运行的前台命令送到后台 |
| `Ctrl+T` | Agent 屏幕 | 切换 todos 窗格 |
| `Ctrl+G` | Agent 屏幕（完整 TUI） | 切换任务窗格 |
| `Ctrl+G` | 普通输入框（极简模式） | 在外部编辑器中编辑当前草稿，不发送。若终端占用了这个组合键，从命令面板选择 **Edit Prompt in External Editor**。 |
| `Ctrl+L` | Agent 屏幕 | 打开扩展模态框（**仅非 VS Code 家族**；在 VS Code / Cursor / Windsurf / Zed 上，`Ctrl+L` 是回合中 **插话**，扩展通过 `/plugins` / `/hooks` 打开） |
| `↑` | 提示框聚焦（空提示，普通输入模式） | 有排队提示时，把焦点移到队列窗格并高亮最后一行（`e` 编辑，`Enter` 立即发送）。否则打开历史面板并填入你上一条提示；`↑`/`↓` 逐步浏览条目（每条落到输入框），在最新一条上按 `↓` 关闭面板，输入则就地编辑召回的提示。召回的 `!` shell 命令会重新进入 shell 模式。`↓` 从不打开历史。 |
| `Ctrl+S`（备用：`Alt+S`） | 提示框聚焦 | 像 `git stash` 一样 stash / pop 草稿。输入框有文本或图片时：stash 并重新开始。空输入框时：恢复最新 stash（含图片和 `!` shell 模式）。用组合键 stash 的草稿还会在**你发送下一条提示后自动恢复**（双 Esc 清空的草稿保持 stash，因为那个手势是丢弃）。一次只存一份草稿：新 stash 替换旧的，旧草稿的文本仍可在 `↑` 历史里找到；被 stash 草稿的文本在那里排第一。 |
| `!` | 提示框聚焦 | 进入 shell 模式（在空提示上输入 `!`） |
| `Ctrl+.`（备用：`Ctrl+X`） | Agent 屏幕 | 打开键盘快捷键帮助 |
| `F2`（备用：`Ctrl+,` / `Cmd+,`） | Agent 屏幕 | 打开设置模态框 |

**注意：** `Ctrl+M` 依上下文而定。提示框聚焦时，它切换多行输入模式。否则打开模型选择器。

**注意：** 草稿被 stash 时，提示框顶边会显示 `Stashed`（若你设过 `/rename` 标题，会挨着它）。极简模式不画边框，因此每次 stash 或恢复都会在回看区打一行。stash 只存在于内存：退出即消失，也不会带到恢复的会话。新 stash 替换旧的，只有旧草稿的**文本**进入 `↑` 历史，因此被替换草稿上的图片会丢失。

**注意：** 外部编辑在每种渲染模式下都可用：极简模式绑定 `Ctrl+G`，完整 TUI 用 `/edit-prompt` 或命令面板。Grok 先解析 `$VISUAL`，再 `$EDITOR`，最后是 `vi`。值可以包含带引号的参数。保存只替换草稿（编辑器保存时追加的末尾换行会被去掉）；空文件会清空草稿。带粘贴/文件/图片 chip 的草稿必须在输入框里编辑，以免附件被压扁。

**注意：** `Ctrl+'` 是 `Ctrl+;` 的 Windows 备用——部分 Windows 控制台会丢掉标点键上的 `Ctrl` 修饰。

**注意：** `Ctrl+.` 需要 Kitty keyboard protocol（或 tmux `extended-keys on` 以便该协议能穿过）。在 VS Code / Cursor / Windsurf / Zed 集成终端、VTE、Apple Terminal、Windows Terminal、JetBrains、`extended-keys off` 的 tmux、screen 以及类似无 KKP 环境中，Grok 会把 **`Ctrl+X`** 宣传为快捷键速查的主键。即使 `Ctrl+.` 无效，**`Ctrl+X` 也始终可用**，因为它是经典控制字符。若修饰键在 tmux 里行为异常，运行 `/doctor`。

---

## 图片粘贴与拖放

| 动作 | macOS | Linux | Windows |
|---|---|---|---|
| 从文件管理器把图片拖进提示框 | Finder ✓ | Files / Dolphin ✓ | Explorer ✓ |
| 在文件管理器中复制文件，然后粘贴 | `Cmd+V` | `Ctrl+V` | `Ctrl+V` |
| 剪贴板里的截图或 “Copy Image”，然后粘贴 | `Cmd+V` | `Ctrl+V` | **`Alt+V`** |

非图片文件会把绝对路径作为文本插入，而不是 chip。

> **Windows 上的 `Alt+V`** 是 grok 特有的。Windows Terminal 默认的 `Ctrl+V` 只粘贴纯文本，并会悄悄丢掉图片剪贴板；`Alt+V` 绕过该拦截。若也要用 `Ctrl+V` 粘贴图片，在 Windows Terminal 的 `settings.json` 的 `actions` 里加入 `{ "command": null, "keys": "ctrl+v" }`。

### Linux PRIMARY 与 CLIPBOARD

Linux X11 有两套独立的文本选择：

- `Ctrl+V` 读取 **CLIPBOARD**，即显式复制/剪切的选择。它从不回退到 PRIMARY。要用 `xclip` 放进去，使用 `printf %s "text" | xclip -selection clipboard`。
- 在 Grok 里未修饰的中键点击读取 **PRIMARY**，即当前鼠标选择，仅当 `DISPLAY` 非空时。纯 X11 可用原生读取回退；XWayland 需要 `PATH` 上有 `xclip` 或 `xsel`，以便 Grok 读 X11 选择而不是 Wayland PRIMARY。按下只处理一次；松开不会再粘贴。
- `Shift+Insert` 是终端原生粘贴所选文本的方式。许多终端也用 `Shift+中键点击` 绕过应用的鼠标报告。

通过 SSH 时，远程 Grok 进程通常访问不到终端本地的 X11 选择。使用终端原生的 `Shift+Insert` 或 `Shift+中键点击`，让本地终端通过 PTY 发送所选文本。

---

## 活动回合期间（agent 正在运行）

agent 正在生成时：

- **普通 `Enter`**（输入框有文本）会把后续消息**排队**稍后处理。默认（`[ui].follow_up_behavior = "queue"`）这些后续会在当前回合结束后运行——并且在 agent 因后台任务或子 agent 阻塞等待时会故意**按住**（提示会解释按住的原因以及如何立即发送）。设为 `"steer"` 时，同一个 Enter 仍会在队列里显示该行，然后 shell 在下一个工具或模型安全间隙把它们注入回合中（见 [配置](05-configuration.md)）。
- **在已清空的输入框上再按 `Enter`**（双 Enter）会立即发送队列**顶部**的后续。
- **立即发送**组合键是 **取消并发送**：它停止当前回合（后台任务、子 agent 和队列其余部分继续跑），并把你的消息作为下一回合发送，因此它总是出现在记录底部：
  - **非空输入框** → 取消并立即发送该文本。
  - **空输入框** + 有排队后续 → 立即发送队列**顶部**的后续（不必聚焦队列窗格）。在队列窗格上，同一组合键（或 **[Send now]** 按钮）发送**选中**行。
  - **空闲**，或 **空输入框且没有排队** → 该键无操作。
- agent **阻塞等待**（任务输出或子 agent）时，带文本的普通 `Enter` 也会立即送达——shell 取消被阻塞的回合，并接着跑你的消息。

| 终端 | 主键 | 备用 | 动作 |
|----------|---------|------------|--------|
| 默认 | `Ctrl+Enter` | `Ctrl+I` | 立即发送（取消当前回合，接着跑你的消息） |
| Apple Terminal | `Ctrl+O` | `Ctrl+Enter`、`Ctrl+I` | 立即发送 |
| VS Code 家族（VS Code、Cursor、Windsurf、Zed） | **`Ctrl+L`** | *（无）* | 立即发送（不用 `Ctrl+I` —— Tab / 宿主聊天；插件用 `/plugins`） |

在 `/multiline` 模式下，`Shift+Enter`（或 `Alt+Enter`）发送，普通 `Enter` 插入换行——但回合中**空**输入框且有排队后续时例外，普通 `Enter` 仍会**立即发送**顶部那一行（与普通模式相同）。（`Ctrl+Enter` 在非 VS Code 家族上绑定时是回合中立即发送；它不会提交新的空闲回合。）

立即发送是有意打断的——读起来像「停下手里的事，先看这个」。若要把备注交给 agent **而不**停下它，用普通 `Enter` 排队；agent 会在下一个回合边界拾取。

> **WezTerm**：这些带修饰的 Enter 键需要在 WezTerm 配置里设 `enable_kitty_keyboard = true`。完整步骤和一行变通见 [终端支持指南](21-terminal-support.md#problem-ctrlenter-doesnt-interject-in-wezterm)。

> **Windows（非 VS Code 家族）**：部分控制台会丢掉 `Ctrl+Enter` 上的 `Ctrl` 修饰（可能塌成裸 `Enter` 或 `Ctrl+J`）。用 `Ctrl+I` 作为备用——字母键的 Ctrl 组合到处都稳定。在 VS Code 家族上，用 **`Ctrl+L`**。

> **VS Code 家族的 `Ctrl+L`**：Grok 用它插话，并让扩展快捷键保持未绑定（用 `/plugins` 或命令面板打开插件）。若你的终端配置仍把 **Clear**（或其他命令）映射到 `Ctrl+L`，宿主绑定可能抢走该组合——重新绑定或删掉它，让 PTY 收到 form feed（`\x0c`）。

---

## 全局

可从任意屏幕使用的操作。

| 按键 | 备用按键 | 动作 | 确认 |
|-----|---------|--------|-------------|
| `Ctrl+N` | | 创建新会话（可选在 git 工作树中） | 是（1000ms 内连按两次） |
| `Ctrl+\` | | 打开或切换 [Agent Dashboard](23-dashboard.md) | 否 |
| `Ctrl+Q` | `Ctrl+D` | 退出应用 | 是（1000ms 内连按两次） |

**VS Code 家族终端**（VS Code、Cursor、Windsurf、Zed 集成终端）：`Ctrl+Q` 被宿主捕获，因此 Grok 把 **`Ctrl+D` 作为唯一退出键**（`Ctrl+Q` 未绑定）。半页下滚改绑到裸 **`Shift+D`**。回合中插话用 **`Ctrl+L`**（无备用），因为 `Ctrl+Enter` / `Ctrl+I` 不能可靠到达 PTY；扩展通过 `/plugins` 打开，而不是 `Ctrl+L`。

> **回到欢迎屏没有按键绑定** —— 在会话内使用 `/home` 斜杠命令（别名 `/welcome`）。见 [斜杠命令](04-slash-commands.md)。

### 破坏性操作确认

确认列为「是」的操作需要在 1000ms 内连按两次。按一次会看到确认提示，再按一次才确认。这样可防止意外丢失会话。

---

## 欢迎屏

仅在欢迎屏上触发的绑定（尚未打开任何 agent 会话）。

| 按键 | 动作 |
|-----|--------|
| `Ctrl+R` | 恢复会话（打开会话选择器） |
| `Ctrl+W` | 打开 New Worktree 对话框（仅在 git 仓库内） |
| `Ctrl+I` | 导入 Claude 设置（可用时） |
| `Ctrl+Shift+I` | 关掉 Claude 导入行（可用时） |

`Ctrl+W`、`Ctrl+I` 和 `Ctrl+Shift+I` 仅在欢迎屏上有效。`Ctrl+R` 在欢迎屏和 agent 会话内都会打开会话选择器（会话内作为模态覆盖层打开，与 `/resume` 命令相同）。启用 `ui.mouse_reporting_toggle` 时，回看区窗格聚焦时 `Ctrl+R` 切换鼠标捕获；从提示框仍用 `Ctrl+R` 打开选择器。`Ctrl+S` 是会话内的提示 stash（欢迎屏上无操作，因为没有草稿可放下）。`Ctrl+Q` 就是上面记录的全局退出绑定，不是欢迎屏专用处理。

---

## Agent Dashboard

[Agent Dashboard](23-dashboard.md) 聚焦时的绑定（`Ctrl+\` 或 `/dashboard`）。

| 按键 | 动作 |
|-----|--------|
| `↑` / `↓`、`j` / `k` | 在 agent 行间导航（选中一行会打开 peek） |
| `Enter` | 打开选中的 agent，或发送已输入的 peek 回复 / 派发提示 |
| `Ctrl+S` | 回复或派发 **并** 附加到该 agent |
| `Ctrl+/` | 切换搜索 / 过滤模式 |
| `Ctrl+R` | 重命名选中的 agent |
| `Ctrl+T` | 固定 / 取消固定 |
| `Ctrl+G` | 切换分组（状态 ↔ 工作目录） |
| `Ctrl+X` | 取消正在运行的回合，或 2s 内连按两次永久删除 |
| `Ctrl+O` | 切换选中 agent 的始终批准 |
| `Tab` | 在列表与派发 / peek 输入之间切换焦点 |
| `Esc` | 往回退（取消搜索 → 关闭 peek → 清除过滤 → 取消聚焦 → 取消选中 → 退出） |
| `Ctrl+\` | 退出仪表盘（或从已附加的 agent 返回） |
| `Ctrl+.`（备用：`?`） | 快捷键速查 |

细节（peek 与派发、搜索前缀、持久化）：[Agent Dashboard](23-dashboard.md)。

---

## 命令面板

按 `Ctrl+P` 或 `?` 打开命令面板——可搜索的常用操作列表，每项显示其按键绑定或斜杠命令。它包括会话操作、扩展模态框各标签（Hooks、Plugins、Marketplace、Skills、Workflows、MCP Servers）等。

输入以过滤，然后按 `Enter` 执行选中操作。

---

## 快捷键栏

TUI 底部显示上下文快捷键栏，展示当前状态下最相关的按键绑定。提示会随以下情况变化：

- 哪个窗格聚焦（回看区 vs. 提示框）
- agent 当前是否在运行
- 选中了哪类条目

---

## 鼠标支持

TUI 支持鼠标交互：

- **点击**回看区条目以选中
- **滚轮**滚动回看区
- **点击**提示区以聚焦
- **悬停**提示区可看到高亮（可通过 `pager.toml` 配置）
- 在 Linux X11/XWayland 上**中键点击**粘贴 PRIMARY 选择

---

## 速查卡

### 回看区聚焦时（简易模式 —— 默认）

```
Navigation:       Up/Down (prev/next entry)  Shift+Left/Right (prev/next turn)
Scrolling:        Ctrl+J/K (line)  PgUp/PgDn (page)  Ctrl+U/D (half page)
Focus prompt:     Space or any letter key (auto-focuses and types)
```

### 回看区聚焦时（Vim 模式）

```
Navigation:       j/k (up/down)  H/L (prev/next turn)  K/J (viewport-top turn)  g/G (top/bottom)
Scrolling:        Ctrl+J/K (line)  Ctrl+U/D (half page; D=Shift+D in VSCode)  PgUp/PgDn (page)
Folding:          h/l (collapse/expand)  e (toggle)  E (all)
Content:          y (copy)  Y (copy cmd)  Enter (fullscreen)
View:             r (raw markdown)  Ctrl+E (thinking)
Focus prompt:     i, Tab, or Space
```

### 提示框聚焦时

```
Send:             Enter
Newline:          Shift+Enter or Alt+Enter
Multiline:        Ctrl+M (toggle)
Paste:            Ctrl+V (text, files, screenshots on macOS/Linux)
Selected text:    Middle click or Shift+Insert (Linux X11/XWayland PRIMARY)
Paste image:      Alt+V (Windows only — for screenshots / "Copy Image")
Select all:       Cmd+A (macOS, Ghostty only — see note below)
Select text:      Shift+←/→ (char) · Alt+Shift+←/→ (word) ·
                  Cmd+Shift+←/→ (visual row) · Shift+Home/End (logical line) ·
                  Shift+↑/↓ (row)
Copy / Cut:       Cmd+C / Cmd+X (with a selection; Kitty-protocol terminals)
Leave:            Tab (back to scrollback)
Cancel (running): Ctrl+C (empty prompt; non-empty draft clears first)
Clear (idle):     Esc Esc within 800ms (non-empty prompt)
Rewind (idle):    Esc Esc within 800ms (empty prompt + messages)
```

有选区时，输入 / `Enter` / 粘贴会替换它，删除和
按词删除组合只删除选区，方向键把选区收拢到
对应边缘（按词/按行移动从该边缘继续），`Esc` 或 `Tab`
会去掉高亮，同时仍执行它们的正常动作。注意
`Shift+←/→` 仅在**提示框**聚焦时选择；回看区
聚焦时，同一组合会在回合同跳转（见上面的导航）。

> **Cmd+A 仅对 Ghostty 开放。** Grok 应用内的 `Cmd+A` 处理
> 仅在检测到的终端是 Ghostty 时接线。其他终端
> 要么在终端层吞掉 `Cmd+A`（Apple Terminal、默认
> iTerm2），要么应用自己的终端内「全选」行为（Kitty、
> WezTerm）。在非 Ghostty 终端上，该绑定无操作，按键
> 落到终端的原生行为。
>
> 在 Ghostty 上，向 `~/.config/ghostty/config` 加入一行解除绑定，
> 让按键到达正在运行的 TUI：
>
> ```ini
> keybind = cmd+a=unbind
> ```
>
> Ghostty 重载配置后（它会监视配置文件），在
> 提示框里按 `Cmd+A` 会选中提示缓冲区里的每个字符，包括粘贴的
> 图片 chip。图片 chip 始终不含路径（`[Image #N]`）；
> 文件路径（已知时）只出现在悬停或光标位于
> chip 上/紧随其后时的图片预览覆盖层。

### 始终可用

```
Command palette:  Ctrl+P or ?
Model picker:     Ctrl+M (from scrollback)
Cancel:           Ctrl+C (see Escape table)
Always-approve:   Ctrl+O (toggle YOLO)
New session:      Ctrl+N (press again, then choose normal/worktree)
Quit:             Ctrl+Q (or Ctrl+D in VSCode)
```
