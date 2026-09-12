# 主题与外观自定义

Grok Build 从中心主题绘制所有 TUI 颜色。你可以在 Grok 运行时切换主题、跟随操作系统的浅色或深色外观，并通过配置文件调整回看区布局、动画和块样式。

---

## 可用主题

Grok 包含六个内置主题，外加跟随系统外观的 `auto` 选项：

| 主题 | 配置名 | 说明 | 需要 Truecolor |
|-------|-------------|-------------|--------------------|
| **GrokNight** | `groknight`、`grok-night`、`dark` | 中性深色底，品红强调。默认主题。在 256 色和 16 色终端上量化干净。 | 否 |
| **GrokDay** | `grokday`、`grok-day`、`light`、`day` | 适合明亮终端背景的浅色主题。 | 否 |
| **TokyoNight** | `tokyonight`、`tokyo-night`、`tokyo` | 来自 Tokyo Night 色板的深色、偏蓝背景。量化后会失去个性。 | 是 |
| **RosePineMoon** | `rosepine`、`rose-pine`、`rosepine-moon`、`rose-pine-moon` | 来自 Rosé Pine 家族的柔和深色色板，带淡紫强调。 | 是 |
| **OscuraMidnight** | `oscura`、`oscura-midnight` | 极深底色，紫色强调。 | 是 |
| **Terminal** | `terminal`、`terminal-default`、`transparent`、`native` | 使用终端自己的颜色：没有自己的背景，因此终端画布（半透明、背景图）会透出来。 | 否 |

主题名不区分大小写。`auto` 选项（别名 `system`）记录在 [自动主题（系统外观）](#auto-theme-system-appearance)。

### Terminal 主题

`terminal` 不绘制任何表面背景，也几乎不定义自己的颜色——一切都来自你的终端配置。回看区、输入框、模态框和状态行让终端画布可见（半透明或带背景图的窗口会透过 Grok，就像透过你的 shell 一样），正文使用终端的默认前景，强调（错误、diff、链接、语法）来自你配置的 16 色 ANSI 色板。因为它借用你配置的颜色，而不是假定浅色或深色背景，所以在任何配置上都保持可读，且无需外观检测，并在每种颜色深度下渲染一致。

该主题完全不画背景：你的消息用粗体而不是色带渲染，菜单和提示面板直接坐在终端画布上，任何菜单中选中或悬停的行使用反色（终端自己的前景/背景对调），因此每种组合在任何配置上都保持可读。装饰——空闲边框、分隔线、滚动条滑块——用 ANSI *bright black* 作为前景，也就是你配置里调成自己变暗色调的那个槽。聚焦的铬件，例如活动输入框边框，保持完整默认前景，让焦点仍然跳出。Grok 在此主题上也保持你的光标颜色不动（其他主题会把它改成自己的强调色）。

```toml
[ui]
theme = "terminal"
```

对比度只取决于你的终端配置：bright-black 槽非常暗的配置会渲染出淡淡的分隔线，因为 Grok 从你的色板派生一切，而不是硬编码颜色。

该主题正在逐步推出。在推出到达你的账户之前，它会从 `/theme` 和 `/settings` 隐藏，其名称无法解析，已配置的 `theme = "terminal"` 会回退到默认主题。设 `GROK_TERMINAL_THEME=1`（或在 `config.toml` 里设 `[features] terminal_theme = true`）可在推出前于本地启用。

### 极简模式没有主题

**极简模式**（`--minimal`）始终用单一固定的终端原生色板渲染，并完全忽略 `theme` 设置（它们仍应用于完整 TUI）。极简直接画在你终端自己的背景上，因此使用终端的默认前景/背景加上其 16 色 ANSI 色板——与 `git` 或 `ls` 相同——在任何浅色或深色终端配置上都保持可读，无需检测或配置。`/theme` 以及 `/settings` 中的主题行在极简模式下不可用。

极简模式的语法高亮**不会**在浅色和深色主题文件之间切换（有意避免极性检测）。近灰 token 继承终端默认前景；彩色 token 使用基础 ANSI 强调（红/绿/黄/蓝/品红/青），因此读文件输出和围栏代码在浅色和深色配置上都保持清晰。

---

## 切换主题

### 在 TUI 中

运行 `/theme` 斜杠命令（别名 `/t`）打开主题选择器。用方向键在列表中移动时，Grok 会实时预览每个主题。按 Enter 应用并保存你的选择，或按 Escape 还原。输入可按主题的任一配置名过滤列表，因此 `/theme transparent` 会收窄到 Terminal 那一行。

要不打开选择器就切换，直接传入名称：

```
/theme tokyonight
```

单独提交 `/theme` —— 不从选择器中选择 —— 会循环到下一个主题。

### 通过配置文件

在 `~/.grok/config.toml` 中设置主题：

```toml
[ui]
theme = "tokyonight"
```

---

## 自动主题（系统外观）

设 `theme = "auto"` 让 Grok 跟随操作系统的浅色/深色外观并自动切换主题：

```toml
[ui]
theme = "auto"
```

默认情况下，深色模式映射到 **GrokNight**，浅色模式映射到 **GrokDay**。用 `auto_dark_theme` 和 `auto_light_theme` 覆盖任一映射：

```toml
[ui]
theme = "auto"
auto_dark_theme = "tokyonight"
auto_light_theme = "grokday"
```

`theme = "system"` 是 `theme = "auto"` 的别名。

### 检测方式

| 平台 | 方法 |
|----------|--------|
| **macOS** | 读取 `AppleInterfaceStyle` 系统偏好 |
| **Linux** | 查询 XDG Desktop Portal（`org.freedesktop.appearance.color-scheme`） |
| **Windows** | 读取系统个性化注册表 |
| **SSH / tmux / 无界面** | `GROK_APPEARANCE` 或 `LC_GROK_APPEARANCE`（`dark`/`light`），然后 `COLORFGBG`，然后启动时的 OSC 11 背景查询。`grok wrap ssh …` 会从本地 OS 主题盖上 `LC_GROK_APPEARANCE`，以便它在 SSH 进登录 shell 后仍在。新 tmux 会话仅当创建该 tmux 服务器/会话时带有该环境（或 `update-environment` 包含它）才会继承。OSC 11 在 tmux 是直接终端（不是编辑器 `:terminal`）且 tmux ≥ 3.3 时用 DCS 包裹；到达外层模拟器还需要 `allow-passthrough`，回复是尽力而为。 |

运行后，Grok 每 5 秒轮询桌面 API 和环境提示。在本地桌面上切换 OS 的浅色和深色模式会在数秒内生效，无需重启。通过 SSH 时，wrap 盖上的环境对该跳固定。

你也可以设置 `GROK_THEME`（或 `LC_GROK_THEME`）来强制某个主题或 `auto`，而不编辑 `config.toml`。

### 通过设置窗格

运行 `/settings`（别名 `/config`）并打开 **Appearance** 类别，以交互方式设置 **Auto dark theme** 和 **Auto light theme**。在 `/theme` 选择器中选 `auto` 会用这些映射启用自动模式。

---

## 颜色支持检测

启动时，Grok 检测终端的颜色能力级别：

| 级别 | 说明 | 检测 |
|-------|-------------|-----------|
| **Truecolor**（24 位） | 完整 RGB 颜色。所有主题按设计渲染。 | `COLORTERM=truecolor` 或等效终端能力 |
| **256-color** | 索引色板。RGB 值映射到最近的色板条目。 | 标准 xterm-256color |
| **16-color** | 仅 ANSI 名。颜色映射到最接近的 ANSI 色。 | 基础终端支持 |

设置 `NO_COLOR` 时，Grok 不发出颜色，并以单色渲染。

运行 `/doctor` 可查看检测到的颜色级别以及此终端上可用的主题。若 truecolor 不可用，Doctor 会显示相关设置步骤或解释终端限制。

### 自动量化

每个主题都用完整 RGB 值定义。启动时，Grok 把所有颜色量化到检测到的能力级别。这意味着：

- 在 **truecolor** 终端上，颜色原样通过。
- 在 **256-color** 终端上，每个 RGB 值映射到最近的索引色板条目。
- 在 **16-color** 终端上，颜色映射到 ANSI 名。

GrokNight 和 GrokDay 使用量化干净的中性灰。TokyoNight、RosePineMoon 和 OscuraMidnight 使用量化后会失去个性的独特着色背景，因此主题选择器在非 truecolor 终端上隐藏它们。

### 运行时生成的颜色

运行时生成的颜色（语法高亮、背景混合）也通过同一管道量化，确保在所有终端类型上外观一致。

---

## 光标颜色

Grok 用 OSC 12 转义序列把终端光标设为当前主题的 `accent_user` 颜色，以表示活动的 Grok 会话。光标颜色会：

- 在启动和切换主题时应用。
- 退出时通过 OSC 112 重置为终端默认。

这在支持 OSC 12 的终端中有效（大多数现代终端）。

---

## 紧凑模式

用 `/compact-mode` 斜杠命令切换紧凑模式。紧凑模式会：

- 去掉外侧垂直内边距（上下边距变为 0）。
- 把水平内边距减到最小（1 列）。
- 减少提示区和信息块的顶部内边距。

该设置持久化在 `~/.grok/config.toml` 的 `[ui].compact_mode` 下，重启后仍在。

在小屏幕上使用紧凑模式以最大化内容区。

---

## 语法高亮

Grok 捆绑三个 `.tmTheme` 文件用于代码块语法高亮，并根据活动主题选择一个：

- `grok-night.tmTheme` —— GrokNight、RosePineMoon 和 OscuraMidnight
- `grok-day.tmTheme` —— GrokDay
- `tokyo-night.tmTheme` —— TokyoNight

切换主题时，Grok 自动选择匹配的文件。`.tmTheme` 文件内建在二进制中，因此你不能用自己的替换它们。

---

## 用 pager.toml 深度自定义

要对 TUI 外观做细粒度控制，创建 `~/.grok/pager.toml`。此文件控制回看区布局、块样式、动画等。所有设置都有默认值；只指定你覆盖的值。（开发构建会把此文件生成为模板，每个默认都注释掉——取消注释一行即可覆盖；注释掉的值会继续跟踪未来的默认。）

### 布局

控制视口内边距和块间距：

```toml
[scrollback.layout]
outer_vpad = 1          # 视口的垂直内边距（上/下）
outer_hpad_left = 2     # 左边距（最小：1）
outer_hpad_right = 2    # 右边距（最小：1）
block_pad_left = 2      # 强调线与内容之间的内边距
block_pad_right = 2     # 内容之后到右边缘的内边距
```

### 滚动条

```toml
[scrollback.scrollbar]
enabled = true          # 显示/隐藏滚动条
gap_left = 0            # 内容与滚动条之间的间隙（0 = 相邻）
gap_right = 0           # 滚动条与屏幕边缘之间的间隙（0 = 贴边）
# scrollbar_bg = "none" # 覆盖背景色（或 "none" 使用主题默认）
# scrollbar_fg = "none" # 覆盖滑块颜色（或 "none" 使用主题默认）
```

### 滚动行为

```toml
[scrollback.scroll]
margin = 0                  # 选中条目上下的上下文行（0 = 贴边）
min_page_fraction = 0       # 最小滚动占视口的百分比（0-100）
follow_indicator = "center" # "center" = 显示 ▼/▲ 滚动箭头，"none" = 隐藏
follow_auto_select = true   # 跟随时自动选中最新条目
follow_by_overscroll = true # 滚过底部进入跟随模式
anchor_on_fold = true       # 折叠时让块标题保持同一屏幕位置
```

### 显示选项

```toml
[scrollback.display]
sticky_headers = true              # 滚过用户提示时把它们钉为标题
tab_width = 4                      # 每个制表符的空格数（0 = 原样通过）
expandable_indicator = true        # 在可折叠的已折叠条目上显示 "›"
expandable_indicator_char = "›"    # 使用的字符（默认："›"）
collapsed_accent_char = "❙"        # 可分组折叠块的强调（在旧版 Windows 控制台上回退到 "|"）
dim_accent = 0.5                   # 变暗强调的混合系数（0.0-1.0）
line_under_last_entry = false      # 最后一条下方的水平线
selection_buttons = false          # 在选中框上显示复制/查看按钮
```

### 动画

```toml
[animation]
fps = 30           # 帧率（1-60）。更高 = 更平滑，更多 CPU
wave_rows = 32     # 强调动画每个波浪周期的行数
```

### 块样式：编辑 Diff

```toml
[scrollback.blocks.edit]
indent = true                   # 缩进 diff 内容
vpad = false                    # diffs 周围的垂直内边距
# expanded_by_default = true    # 未设置：跟随 config.toml 的 [ui] collapsed_edit_blocks
                                # （旗标开 = 折叠单行）；取消注释以钉住任一种形态
hunk_separator = "…"            # hunk 之间的分隔符（"…"、"───"、"⋯"，或 "" 表示无）
dual_line_numbers = false       # 双列行号（旧 + 新，像 GitHub）
# line_summary = false          # 在折叠标题显示 +N/-M；未设置跟随同一旗标
# bg = "none"                   # 块背景（"none"、"light"、"dark"）
```

### 块样式：思考/推理

```toml
[scrollback.blocks.thinking]
accent_enabled = true       # 为思考块显示强调线
animate = true              # 思考时动画强调线
truncated_lines = 3         # 截断模式显示的行数
bg_blend = 70               # markdown 颜色与背景的混合（0-100）
header = true               # 显示 "Thinking..." 标题
header_bright = false       # 明亮标题样式（相对变暗/柔和）
```

### 块样式：工具调用

```toml
[scrollback.blocks.tool]
muted_collapsed = true     # 把折叠的工具调用变灰
dim_details = true          # 变暗括号细节（行数、匹配数）
bullet = "diamond"          # 工具标题前的项目符号样式
```

可用的项目符号样式：

| 配置值 | 字符 | 说明 |
|-------------|-----------|-------------|
| `none` | （无） | 无项目符号 |
| `dot` | `·` | 间隔号（最小） |
| `small-circle` | `•` | 圆点 |
| `circle` | `●` | 实心圆 |
| `small-triangle` | `▸` | 右指小三角 |
| `triangle` | `▶` | 右指三角 |
| `diamond` | `◆` | 实心菱形（默认） |

### 块样式：执行（Shell 命令）

```toml
[scrollback.blocks.execute]
first_lines = 2                   # 截断模式开头显示的输出行
last_lines = 3                    # 截断模式结尾显示的输出行
accent_enabled = true             # 显示强调线（运行时带动画）
header_style = "label"            # "shell"（$ 前缀）或 "label"（Run 前缀）
muted_command_collapsed = true    # 折叠时弱化命令文本
```

### 块样式：用户提示（回看区）

```toml
[scrollback.blocks.prompt]
vpad = true            # 垂直内边距
bg = "light"           # 背景（"none"、"light"、"dark"）
show_prefix = true     # 显示提示前缀字符
min_lines = 2          # 截断/粘性模式下最少内容行
```

### 提示输入控件

```toml
[prompt]
collapse_unfocused = true    # 回看区聚焦时收起
mouse_hover = true           # 鼠标悬停时显示高亮
show_prefix = true           # 显示提示前缀字符
```

### 终端行为

```toml
[terminal]
alt_screen = "auto"    # "auto"、"always" 或 "never"
```

备用屏幕策略：
- `auto` —— 在普通终端和普通 tmux 中全屏；在 tmux 控制模式和 Zellij 中行内。
- `always` —— 始终进入全屏。
- `never` —— 从不进入全屏；在主回看区中行内运行。

### 插件 UI

```toml
disable_plugins = false   # 设为 true 以隐藏 /hooks、/plugins 命令和标注
```

---

## 主题颜色槽

每个主题定义以下颜色槽，供整个 TUI 使用：

**背景：** `bg_base`、`bg_light`、`bg_dark`、`bg_highlight`、`bg_hover`、`bg_terminal`、`bg_visual`

**强调：** `accent_user`、`accent_assistant`、`accent_thinking`、`accent_tool`、`accent_system`、`accent_error`、`accent_success`、`accent_running`、`accent_skill`、`accent_plan`、`accent_verify`、`accent_remember`、`accent_model`

**文本：** `text_primary`、`text_secondary`

**灰色：** `gray_dim`、`gray`、`gray_bright`

**语义：** `command`、`path`、`running`、`warning`、`fuzzy_accent`

**边框与滚动条：** `selection_border`、`hover_border`、`prompt_border`、`prompt_border_active`、`scrollbar_bg`、`scrollbar_fg`

**粘贴：** `paste_bg`、`paste_fg`、`paste_dim`

**Diff：** `diff_delete_bg`、`diff_delete_fg`、`diff_insert_bg`、`diff_insert_fg`、`diff_equal_fg`、`diff_gutter_fg`

**Markdown：** 标题颜色（`md_heading_h1`–`md_heading_h6`）、`md_code`、`md_code_bg`、`md_text`、`md_muted`、`md_task_checked`、`md_task_unchecked`、`link_fg`

主题系统在内部管理这些槽，并自动为你的终端量化它们。
