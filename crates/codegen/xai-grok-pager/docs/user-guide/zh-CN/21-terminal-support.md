# 终端支持与排障

Grok Build 作为全屏 TUI 运行。它依赖终端对颜色、剪贴板、键盘输入、鼠标输入和全屏显示的支持。终端、多路复用器、容器和 SSH 会话对这些功能的处理可能不同。

## 诊断并修复终端问题

在 Grok 里运行 `/doctor` 可检查当前会话并查看可用修复。若 Grok 无法启动，在 shell 里运行 `grok doctor`。用 `grok doctor --json` 得到机器可读报告。

Doctor 会检查终端、多路复用器、颜色支持、键盘与换行行为、剪贴板路径，以及在包含音频捕获时的麦克风可用性。应用内命令还可以检查实时会话细节，例如通知焦点跟踪和沙箱配置冲突。

报告可以包含问题或建议，仍以成功退出。管道时 `grok doctor --json` 报告同样的颜色能力。麦克风检查不会开始录音，因此 Doctor 无法检测只在捕获期间表现为静音的 macOS 权限失败。

`/terminal-setup`、`/terminal-check` 和 `/terminal-info` 仍是 `/doctor` 的别名。

当 Doctor 发现明确不健康的 tmux 设置时，`/doctor fix` 会列出可用的自动修复。一次应用一个命名修复，例如 `/doctor fix tmux-clipboard` 或 `grok doctor fix dcs-passthrough --yes`。Doctor 可以持久化这四个 tmux 选项：

- `terminal.tmux-clipboard` — `set -g set-clipboard on`
- `terminal.dcs-passthrough` — `set -wg allow-passthrough on`
- `terminal.tmux-extended-keys` — `set -g extended-keys on`
- `terminal.tmux-truecolor` — `set -as terminal-features ",*:RGB"`

tmux 修复只编辑托管受影响 tmux 服务器的那台计算机上的持久配置，包括远程会话。普通 tmux 使用真正的 `$HOME/.tmux.conf`；Byobu-tmux 使用其有效的 `BYOBU_CONFIG_DIR`，若该目录不可用或不安全则拒绝猜测。Grok 会保留文件的换行和模式，更改已有文件时做备份，并拒绝冲突或含糊的直接赋值。

Grok 有意**不会**运行 `tmux source-file` 或更改活着的 tmux 服务器。用应用后显示的精确命令重新加载，或分离再重新附着，然后再次运行 `/doctor`。重新加载之前，活着的 finding 预期会保持。保守的配置扫描只检查直接的全局赋值； sourced 文件、条件、插件和生成的 tmux 设置请自行审阅。

---

## 已检测的终端

Grok 从环境变量检测这些终端模拟器：

- **Apple Terminal**
- **Ghostty**
- **iTerm2**
- **Warp**
- **WezTerm**
- **Kitty**
- **Alacritty**
- **Rio**
- **foot**（Wayland 原生，Linux）
- **VS Code**、**Cursor**、**Windsurf** 和 **Zed** 集成终端
- **JetBrains** IDE 终端
- **Grok Desktop**
- 基于 **VTE** 的终端，例如 GNOME Terminal、GNOME Console 和 Tilix
- **Windows Terminal**

检测有这些限制：

- 在 tmux 里，标识外层终端的变量可能到不了 Grok。
- 通过 SSH 时，许多终端变量不会被转发。
- tmux 的全局环境反映的是第一个附着到服务器的客户端，不一定是当前终端。

---

## 常见问题与修复

### 颜色看起来不对或缺少 truecolor

运行 `/doctor`。完全支持的设置显示 `color truecolor` 和 `themes all`。若不是，Doctor 会显示检测到的限制和相关修复。

在 tmux 里有两个分开的问题：Grok 发出什么颜色，以及什么颜色能穿过多路复用器。`color` 行回答第一个。对于第二个，当附着的客户端未标记 `RGB` 时，tmux 会把每一种 24 位颜色改写成外层终端 terminfo 所宣传的最近颜色，可能少到只有八种。主题于是看起来发灰，即使 `color` 读到 `truecolor`。Doctor 把这报告为 `terminal.tmux-truecolor`。重新加载你的 tmux 配置，然后分离再重新附着：服务器只在重新加载时读取新选项，客户端只在附着时修正色深，因此单独任一步都不会改变任何东西。

### 剪贴板问题

Grok 最多通过三条路径写入，在 `/doctor` 的 **Clipboard** 下显示：

- **native** — 本地操作系统剪贴板。
- **tmux** — Grok 在 tmux 里运行时的 tmux 粘贴缓冲区。
- **OSC 52** — 可以穿过 tmux、容器或 SSH 的转义序列。

#### Wayland

现代 Wayland 合成器可以在终端未保持焦点时更新剪贴板。较旧的合成器可能要求 Grok 保持焦点直到复制消息出现。适用时 Grok 会显示启动警告；运行 `/doctor` 查看检测状态和步骤。

`GROK_CLIPBOARD_NO_DATA_CONTROL=1` 是关闭 data-control 路径的高级回退。复制随后使用命令行剪贴板工具。

#### OSC 52 总开关

在 Linux 上，以及穿过 tmux、SSH 或无显示容器时，若该路径已启用，Grok 会发出 OSC 52。未实现 OSC 52 的终端可能把编码后的载荷显示为文本。启动 Grok 前设 `GROK_CLIPBOARD_NO_OSC52=1` 可关闭该路径。`/doctor` 随后显示 `osc 52 off`；native 和 tmux 路径不变。

#### Linux X11 选择

X11 的 **PRIMARY** 和 **CLIPBOARD** 是分开的：

- 未修饰的中键单击仅在 `DISPLAY` 已设置时读取 PRIMARY。在 XWayland 下，`xclip` 或 `xsel` 必须在 `PATH` 上。
- `Ctrl+V` 读取 CLIPBOARD，从不回退到 PRIMARY。
- `Shift+Insert` 仍是终端的已选文本粘贴。

#### SSH 与已选文本

远程 Grok 进程通常读不到本地终端的选择。使用终端原生的 `Shift+Insert`，或在终端用该手势绕过鼠标报告时按住 `Shift` 中键单击。

当 Grok 无法通过 SSH 识别外层终端时，它会预测将发送 OSC 52，但把该路径标为未验证。复制 toast 随后会点名备份文件，以便你取回文本。运行 `/doctor` 查看其他复制选项。

#### 通过 SSH 的 Apple Terminal

Apple Terminal 不支持 OSC 52，因此远程复制到不了本地剪贴板。每次复制仍会保存到备份文件（默认 `~/.grok/last-copy.txt`；用 `GROK_COPY_FILE` 覆盖）；当投递未验证或剪贴板不可达时，toast 会点名该路径。你也可以用 `/copy <file>` 或 `/minimal`。

若要直接转发剪贴板，从本地计算机通过 `grok wrap` 运行 SSH 命令，例如 `grok wrap ssh user@host`。同一命令可以包装容器和 pod shell。它还会在连接断开后恢复终端模式。

当 SSH 会话未使用 `grok wrap` 时，Grok 会显示一次性提示 “Run `/doctor` for details and fixes.”。会话通过 wrap 启动后，该提示不再出现。用 `/settings` → **Show contextual hints** → **SSH wrap** 关掉它，或在 `$GROK_HOME/config.toml` 的 `[ui.contextual_hints]` 下设 `ssh_wrap = false`。此设置不会隐藏 Doctor 建议。

对于反复的 SSH 使用，Doctor 提供 `grok doctor fix ssh-wrap`。它还会显示一次性命令、将要更改的文件，以及应绕过该别名的情况。ID `terminal.ssh-wrap` 仍被接受并出现在 JSON 里。

> **警告**：`grok wrap` 是实验性的，可能并非在每种设置下都有效。

#### iTerm2

iTerm2 可能需要 OSC 52 剪贴板访问权限。运行 `/doctor`；`terminal.iterm2-clipboard-permission` 建议会显示要检查的设置。

### 全屏或备用屏幕没有激活

Zellij 和 tmux 控制模式可能限制备用屏幕。Grok 在这些环境里通常使用 inline 模式。运行 `/doctor` 查看检测到的状况。你可以在 `~/.grok/pager.toml` 里配置 `[terminal] alt_screen`，或运行 `grok --no-alt-screen` 确认 inline 模式可用。

### Zellij 快捷键干扰 Grok

Zellij 可能在 Ctrl/Alt 键到达 Grok 之前拦截它们。在 Zellij 0.41 或更高版本上，使用 **Unlock-First (non-colliding)** 预设：

1. 按 `Ctrl+o`，然后 `c`。
2. 打开 **Change Mode Behavior**。
3. 选择 **Unlock-First (non-colliding)**。
4. 按 `Enter` 应用。

需要 Zellij 自己的窗格或会话控制时按 `Ctrl+g`。在极简模式里，若 `Ctrl+G` 仍到不了 Grok，打开命令面板并选择 **Edit Prompt in External Editor**。这会保留当前草稿；输入 `/edit-prompt` 会开始一份空的编辑器草稿，因为命令本身占着 composer。

### WezTerm 里 Ctrl+Enter 无法插入打断

WezTerm 默认关闭 Kitty 键盘协议。在 Grok 里运行 `/doctor`。`terminal.wezterm-kitty` finding 会显示设置和重启步骤。通过 SSH 时，Doctor 只显示能在当前会话里生效的变通办法。Apple Terminal 用 `Ctrl+O` 打断，因为它无法区分带修饰的 Enter 和弦。

### VS Code 里 Shift+Enter 不插入换行

VS Code、Cursor、Windsurf 和 Zed 终端使用 xterm.js，它只部分实现 Kitty 键盘协议，并错误编码某些带 Shift 的可打印键。因此 Grok 不在那里协商该协议，Shift+Enter 可能作为与 Enter 相同的 `CR` 到达。当 `TERM_PROGRAM` 未被转发时，通过 SSH 到达的 VS Code 也受影响。用 `Alt+Enter` 插入换行；`/doctor` 会报告 `terminal.newline-fallback`，并带上检测到的说明和变通办法。

### 鼠标滚动停止工作

若 Grok 不再接收鼠标输入，在终端里重新启用鼠标报告：

- **Apple Terminal**：**View → Allow Mouse Reporting**（`Cmd+R`）。
- **iTerm2**：**Settings → Profiles → Terminal → Enable mouse reporting**。

### 语音听写什么都没录到

大约 10 秒没有 transcript 后，Grok 会停止捕获并显示 **“No speech was detected. Voice stopped.”**，并带上麦克风修复步骤。在 macOS 上，被拒绝的麦克风授权看起来可能和静音一样，因为权限属于托管 Grok 的终端。打开 **System Settings → Privacy & Security → Microphone**，启用该终端并重启它。若访问已经打开，检查 **System Settings → Sound → Input** 下的输入设备和电平，然后再试。

运行 `grok doctor`，或在语音模式开启时运行 `/doctor`。**Voice** 段显示 Grok 将使用的麦克风。若没有可用的输入设备，Doctor 显示 `voice.no-input-device` 以及下一步。当 macOS 提供静音时，Doctor 无法被动检测被拒绝的 macOS 麦克风访问。

在 macOS 上，每次听写都使用短生命周期的捕获辅助进程，以便捕获结束时释放音频栈的内存。若问题可能出在辅助进程本身，设 `GROK_VOICE_CAPTURE=inprocess` 使用进程内回退以便比较。

### 搭配 GNU screen 的 Byobu

GNU screen 上的 Byobu 支持有限。`/doctor` 报告 `terminal.byobu-screen`，并说明如何切换到 Byobu 的 tmux 后端。

### 阿拉伯语和波斯语（RTL）文本

许多终端已经自己重排从右到左的文本（基于 VTE 的终端、Terminal.app、Konsole、mlterm 等）。因此 Grok Build **默认不**重排 RTL。

若**回看区**（或列表内容）里的阿拉伯语或波斯语读起来是反的，在 `~/.grok/pager.toml`（或项目配置）里启用应用侧重排：

```toml
[scrollback.display]
rtl_bidi = true
```

该设置随外观配置重新加载（无需完整重启）。若默认时文本看起来正确、启用后反而错了，请再关掉——你的终端已经在处理 bidi。

启用时：

- 重排回看区、列表内容和全屏块查看器里的完整内容行（以及镜像回看区的仪表盘 peek 预览和 hook 弹出层）。Chrome、下拉和模态框保持逻辑顺序，以便命中测试保持一致。
- 不改 markdown 表格列。
- 搜索高亮、选择/拖选复制、双击词/URL 选择和链接命中目标，都会在同一行的绘制（视觉）单元格与逻辑文本之间映射，因此屏幕上的高亮落在正确的字形上，而剪贴板粘贴保持逻辑顺序。
- 基方向按绘制行解析。以英语开头的软换行续行，其基方向可以与段落第一行不同。

这不是完整的镜像 RTL UI。

---

## 还是卡住了？

运行 `/feedback` 报告它。
