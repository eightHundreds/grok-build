# 权限与安全

控制 Grok 能访问和做什么：权限模式、allow/ask/deny 规则、hook，以及可选的操作系统级沙箱。

- **模式** 设定 Grok 请求批准的频率（始终批准、auto、ask，以及相关模式）。
- **规则** 在该基线上设定哪些工具被允许、询问或拦截。

---

## 权限模式

当 Grok 编辑文件、运行命令或调用外部工具时，它可能会暂停等待批准。权限模式控制这发生的频率。

模式设定基线。Allow、ask 和 deny [规则](#configuring-permissions) 仍叠加在任何模式之上。

### 起点

| 情形 | 模式 |
| --------- | ---- |
| 交互式 TUI | 默认（ask），或用 auto 减少提示并做后台检查 |
| 脚本、SDK、CI、agent 服务器 | 始终批准；用 [deny 规则](#configuring-permissions) 或 hook 加硬限制 |

```bash
grok -p "Run the tests" --always-approve
grok agent --always-approve stdio
grok agent --always-approve serve --bind 127.0.0.1:2419 --secret <token>
```

ACP 客户端可以在 `session/new` 上设 `"_meta": { "yoloMode": true }`。见 [Agent 模式](15-agent-mode.md#automation-and-sdks)。

### 可用模式

| 模式 | 不询问即可运行的内容 | 最适合 |
| ---- | ------------------------ | -------- |
| `default`（**ask**） | 只读工具和内置只读 shell 命令 | 日常交互使用 |
| `acceptEdits` | 不提示即可编辑文件 | 本地编码，稍后审阅 diff |
| `plan` | 为兼容而接受；用[计划模式](19-plan-mode.md)做有门闩的规划 | 兼容 Claude 的设置 |
| `auto` | 安全检查允许的工作；其他调用被拦截或升级 | 想少些提示的交互会话 |
| `dontAsk` | 仅预先批准的工具和内置只读处理 | 严格的 CI 允许列表 |
| `bypassPermissions`（**始终批准**） | 工具调用大体上（`deny` 规则、hook 和某些 shell `ask` 规则仍生效） | 受信任的自动化和 agent 服务器 |

**始终批准** 是产品名称；配置和兼容 Claude 的设置可能用 `bypassPermissions` 表示同一模式。始终批准和 auto 互斥（两者都被请求时始终批准优先）。

### 如何设置模式

**交互式 TUI：** `Shift+Tab` / `Ctrl+O`、`/always-approve` 或 `/auto`，或 `/settings`（[快捷键](03-keyboard-shortcuts.md)、[命令](04-slash-commands.md)）。

**CLI：**

```bash
grok --always-approve -p "Run the test suite"
grok --permission-mode auto
grok agent --always-approve serve --bind 127.0.0.1:2419 --secret <token>
```

**配置：**

```toml
[ui]
permission_mode = "always-approve"   # or "auto", "ask", …
```

也支持 `.claude/settings.json` 里兼容 Claude 的 `defaultMode`（见 [Claude 兼容设置](#3-claude-code-compatibility-claudesettingsjson)）。对该进程，CLI 覆盖配置。

### 始终批准

跳过普通权限提示，让工具不等点击就跑。`deny` 规则、hook 和某些 shell `ask` 规则仍生效。管理员可以锁定关掉该模式（见下）。

| 机制 | 示例 |
| --------- | ------- |
| CLI | `--always-approve`（别名 `--yolo`），或 `--permission-mode bypassPermissions` |
| 配置 | `[ui] permission_mode = "always-approve"` |
| 交互 | `/always-approve`、`Ctrl+O` |
| ACP | `session/new` 上的 `_meta.yoloMode: true` |

#### 带硬限制的始终批准

为自动化保留始终批准，并为你永远不想跑的路径或命令加上 deny 规则：

```toml
# project .grok/config.toml
[ui]
permission_mode = "always-approve"

[permission]
deny = [
  "Bash(rm -rf *)",
  "MCPTool(sales__delete_*)",
]
```

```bash
grok -p "Deploy the service" --always-approve --deny 'Bash(rm -rf *)'
```

Deny 始终胜过 allow，也胜过始终批准的正常放行。见 [配置权限](#configuring-permissions)。

### Auto 模式

通过在许多工具调用运行前检查它们来减少交互提示。例行本地工作通常会继续。分类器不会自动允许的调用会弹出权限提示，让你可以允许或拒绝。在非交互会话（`grok -p`、未识别的 stdio）里，同一次调用会失败并报告给模型（例如 `Auto mode blocked this action …`）。

对于必须在没有交互批准的情况下跑工具的自动化，请用始终批准（若需要硬拦截再加上 deny 规则），而不是单独用 auto。

### 禁用始终批准（管理员）

组织可以通过 CLI、TUI 或 `/always-approve` 阻止启用始终批准。在 `requirements.toml` 里设置（用户级在 `~/.grok/` 下，或对用户无法移除的强制执行用系统级 `/etc/grok/`）：

```toml
[ui]
disable_bypass_permissions_mode = true
```

不要用 `permission_mode` 做这把锁；那个键是可切换的默认值。`requirements.toml` 里遗留的 `[ui] yolo = false` 键也会为兼容而禁用始终批准。

Grok 仍可以从托管设置加载 Claude 风格的权限**规则**；始终批准用上面所示的 `requirements.toml` 锁定。

---

## 工具调用如何被授权

当模型请求工具时，按以下顺序检查：

1. **`PreToolUse` hook**。Hook 可以在任何其他检查之前拒绝工具调用。允许调用的 hook 不会跳过下面的检查；它只是选择不拒绝。见 [10-hooks.md](10-hooks.md)。

2. **权限规则**（来自配置文件或 `--allow`/`--deny` 旗标）
   - 匹配的 `deny` 规则拒绝该调用。`deny` 胜过其他每一条规则。
   - 匹配的 `ask` 规则会提示你，包括对本来会自动批准的文件读取、搜索和 shell 命令。
   - 匹配的 `allow` 规则批准该调用。

3. **记住的授权**。你从较早提示保存的按命令批准在这里生效，范围是当前项目。已有授权可以满足 `ask` 规则而不再提示。[危险命令](#dangerous-commands) 列表上的命令会再次提示，而不是使用记住的前缀。见 [交互式批准](#interactive-approvals-and-where-they-persist)。

4. **内置自动批准**。只读工具和一组固定的只读 shell 命令不提示即可运行（见下）。

5. **提示策略**（由[权限模式](#permission-modes)设定）：提示你、自动批准，或自动拒绝该调用。

[始终批准](#always-approve) 在第 2 步之后短路这条流水线：`deny` 规则、hook，以及匹配 shell 命令段的 `ask` 规则仍生效，但不咨询记住的授权（包括记住的 "never allow" 条目），非 shell 工具上的 `ask` 规则也不提示。

---

## 默认从不提示的操作

下列操作被视为只读，在每一种模式（包括 `dontAsk`）下都不提示即可运行，除非匹配的 `deny` 规则或 hook 拦截它们。`ask` 规则会强制对文件读取、搜索和 shell 命令提示（见 [工具调用如何被授权](#how-a-tool-call-is-authorized)）。

### 只读工具

- `read_file`
- `list_dir`
- `grep`（内容搜索）
- `web_search`
- `todo_write`
- `get_command_or_subagent_output` / `kill_command_or_subagent`（子 agent 控制）
- 调用 skills

### 只读 Shell 命令

拆分链式命令（按 `&&`、`||`、`;` 和管道）之后，下列命令作为主命令出现时被识别为只读。此列表按词边界匹配，因此 `ls` 不会匹配 `lsof` 或 `less`。（你自己的 `Bash(...)` 规则匹配方式不同；见 [规则匹配参考](#rule-matching-reference)。）

**文件系统（只读查看）：**
- `ls`、`cat`、`pwd`、`date`、`whoami`、`hostname`、`uptime`、`ps`
- `head`、`tail`、`wc`、`sort`、`uniq`、`tr`、`cut`

**Git（只读）：**
- `git status`、`git branch`、`git log`、`git diff`、`git ls-files`、`git show`、`git rev-parse`
- `git blame`、`git describe`、`git merge-base`、`git shortlog`
- `git check-ignore`、`git check-attr`、`git cat-file`、`git ls-tree`、`git show-ref`、`git for-each-ref`、`git rev-list`、`git name-rev`、`git count-objects`

**搜索与检查：**
- `grep`、`rg`（不是会为每个文件派生预处理器的 `rg --pre` / `rg --pre=…`）

**Kubernetes（只读）：**
- `kubectl get`、`kubectl logs`、`kubectl describe`

> **注意：** `tee` 不在此列表上，因为它可以把输入写到任意文件。`cargo check` 不在此列表上，因为它会编译并运行仓库里的 `build.rs`、过程宏，以及任何 `build.rustc-wrapper`（因此在 Ask 模式下会提示；Auto 模式仍可能按启发式把 `cargo` 当作项目代码运行器允许）。`sort --compress-program=…`（包括唯一的长选项缩写）、`git -c` / `--config-env` 覆盖，以及本地/工作树配置安装了可执行 hook 的 git 命令（`core.fsmonitor`、`diff.*.command`/`textconv`/`external` 驱动，或 shell `alias.<safe-subcommand> = !…`）会抬高请求级下限并提示而不是自动批准，除非用户授予了那条精确的完整脚本或启用了始终批准。

这些检查按段应用。在 `ls && rm -rf /` 这样的命令里，`ls` 段被识别为只读，但 `rm` 段不在列表上。在 `default` 模式下 `rm` 段会提示；在 `dontAsk` 下它被拒绝。

---

---

## 配置权限

Grok 从三个兼容来源读取权限规则。所有来源的规则合并成一套；规则的效果取决于其动作（`deny` > `ask` > `allow`），而不是来自哪个文件。

### 权限规则存在哪里（范围）

权限规则可以是全局的（所有项目）、项目范围的（一个仓库），或项目内属于你个人的：

| 范围 | 文件 | 与队友共享 |
|-------|------|-----------------------|
| 全局（所有项目） | `~/.grok/config.toml` | 否 |
| 项目（提交） | `<project>/.grok/config.toml` | 是（提交它） |
| 项目（个人） | `<project>/.claude/settings.local.json` | 否（gitignore 它） |
| 交互式授权 | 由 Grok 按项目在内部存储 | 否 |

关于范围的说明：

- Grok 会从仓库根到你的工作目录发现每一层目录上的 `.grok/config.toml`，因此子目录可以在仓库根之上追加规则。
- 所有范围的规则合并成一套规则；`deny` > `ask` > `allow` 跨范围生效，因此全局 `deny` 不能被项目 `allow` 覆盖。
- Grok 没有原生的 `config.local.toml`。对于项目里个人的、不提交的规则，使用 `.claude/settings.local.json`；Grok 直接读取它（见 [Claude Code 兼容](#3-claude-code-compatibility-claudesettingsjson)）。
- 交互式 "Always allow" 决定存储在仓库外，范围是该项目（见 [交互式批准](#interactive-approvals-and-where-they-persist)）。

要在一个项目里停止对特定命令的提示，把一条窄的 allow 规则加到该项目的 `.grok/config.toml`（或 `.claude/settings.json`）：

```toml
[permission]
allow = ["Bash(cargo test *)", "Bash(npm run build)"]
```

这只批准列出的命令。相比之下，始终批准模式批准所有工具调用。

### 1. CLI 旗标

```bash
grok -p "Review the API changes" \
  --allow 'Bash(git *)' \
  --allow 'Bash(gh *)' \
  --allow 'Read' \
  --allow 'Grep' \
  --deny 'Bash(rm -rf *)'
```

`--allow RULE` 和 `--deny RULE` 可以重复，并且始终被强制执行。

规则语法示例：
- `Bash(git *)` — 任何以 `git ` 开头的命令
- `Bash(npm run build)` — 精确命令（或前缀）
- `Bash(git commit:*)` — `cmd:*` 后缀形式，等价于对 `git commit` 的前缀匹配
- `Read(src/**)` — `src/` 下的读访问
- `Edit(**/*.rs)` — 编辑任何 Rust 文件
- `Grep` — 所有 grep 操作
- `MCPTool(my-server__*)` — 来自特定服务器的 MCP 工具

精确匹配语义（包括链式命令和通配符如何求值）见 [规则匹配参考](#rule-matching-reference)。

### 2. 原生配置（`~/.grok/config.toml` 和 `.grok/config.toml`）

```toml
[permission]
rules = [
  { action = "allow", tool = "bash", pattern = "git *" },
  { action = "allow", tool = "bash", pattern = "gh *" },
  { action = "allow", tool = "read" },
  { action = "allow", tool = "grep" },
  { action = "deny",  tool = "bash", pattern = "rm -rf *" },  # block a dangerous pattern
  { action = "ask",   tool = "edit" },
]
```

结构化的 `tool` 字段接受小写名称 `bash`、`read`、`edit`、`grep`、`mcp`、`webfetch` 和 `websearch`，对应 [工具名称](#tool-names) 里的工具类。

因为 `deny` 始终胜出，你不能把这些 `allow` 规则与对 `bash` 的兜底 `deny` 组合成「只允许 git/gh」；一条 `deny tool = "bash"` 规则也会拦截 `git` 和 `gh`。要默认拒绝，在 `.claude/settings.json` 里用 `defaultMode: "dontAsk"`，或用 `PreToolUse` hook（见下）。

来自全局 `~/.grok/config.toml` 和每一个项目 `.grok/config.toml`（从仓库根到你的工作目录）的规则，会与任何 `.claude/settings.json` 规则一起合并成一套规则。

组织部署的托管配置也会贡献 `[permission]` 规则：系统的 `/etc/grok/managed_config.toml`，以及 Grok 自动维护在 `~/.grok/managed_config.toml` 的用户级副本。托管规则像任何其他来源的规则一样合并，托管 `allow` 规则有两个特有属性：你自己的 `deny` 和 `ask` 规则胜过托管 `allow`（严重性排序），以及在始终批准被锁定关闭时，兜底的托管 `allow` 会被忽略。对于用户无法改掉的规则，使用 root 拥有的系统 `/etc/grok/requirements.toml`。

每个来源的权限规则在会话启动时读一次。改动在下一会话生效。

原生 `[permission]` 段也接受紧凑的 `allow` / `deny` / `ask` 字符串数组形式，使用与 `--allow` / `--deny` 旗标和 `.claude/settings.json` 相同的规则字符串：

```toml
[permission]
deny = [
  "Read(/Users/you/private/**)",
  "Edit(/Users/you/private/**)",
  "Bash(rm -rf *)",
]
allow = [
  "Bash(git *)",
  "Bash(gh *)",
]
```

`deny` 始终胜过 `allow`（求值是 `deny` > `ask` > `allow`），无论顺序或来源。要在操作系统层也拦截项目外路径的读取，把 deny 规则与 `strict` 沙箱配置组合（见 [18-sandbox.md](18-sandbox.md)）。

### 3. Claude Code 兼容（`.claude/settings.json`）

Grok 读取 `~/.claude/settings.json` 和 `~/.claude/settings.local.json`，以及项目级 `<project>/.claude/settings.json` 和 `settings.local.json`（向上走到仓库根）。权限规则的原生 `.grok` 来源是上一节描述的 `config.toml`。

示例：

```json
{
  "permissions": {
    "defaultMode": "dontAsk",
    "allow": [
      "Read",
      "Grep",
      "Bash(git *)",
      "Bash(gh *)"
    ],
    "deny": [
      "Bash(rm -rf *)"
    ]
  }
}
```

支持的 `defaultMode` 值包括 `default`、`auto`、`acceptEdits`、`bypassPermissions`、`dontAsk` 和 `plan`。Grok 从 `permissions` 下的规范位置读取 `defaultMode`；嵌套键缺失时也接受顶层 `defaultMode`。

`permissions.allow`、`permissions.deny` 和 `permissions.ask` 条目会被翻译成原生规则，然后按 [规则匹配参考](#rule-matching-reference) 里的语义匹配。翻译说明：

- MCP 工具的规则既可以用 `.claude/settings.json` 文件里的 `mcp__server__tool` 形式，也可以用原生 `MCPTool(server__tool)` 形式（见 [MCP 规则](#mcp-rules)）。
- 点名无法识别工具的规则，以及 `Agent(model:opus)` 这类参数规则，会带着警告跳过，而不是让加载失败。
- `permissions.additionalDirectories` 会被解析但不支持。

你可以用 **Ctrl+I**（"Import Claude settings"）交互式导入已有的 Claude 设置。

---

## 规则匹配参考

本节定义规则究竟如何匹配。

### Bash 规则

`Bash(...)` 模式以两种方式之一匹配命令（对 `allow` 规则是每个链式段——见下面的「链式命令」）：

- **前缀**：命令以模式文本开头，逐字符比较。没有词边界要求，因此 `Bash(git)` 既匹配 `gitleaks` 也匹配 `git status`。加上尾随空格和通配符（`Bash(git *)`）可要求前缀是完整的词。
- **Glob**：模式作为 glob 匹配整条命令（或整个段）。`*` 可以出现在任何位置，并匹配任意字符，包括空格和斜杠，因此 `Bash(git * main)` 匹配 `git checkout main`。也支持 `?` 和 `[...]`。

匹配区分大小写。匹配前会修剪命令的前导空白。对 `deny` 和 `ask` 规则，原始命令字符串除此之外不规范化；段级检查还会匹配规范化形式（见下）。

Bash 规则上的尾随 `:*` 后缀会被剥成普通前缀：`Bash(git commit:*)` 变成前缀 `git commit`。因为前缀没有词边界，写成 `Bash(sed:*)` 的 `deny` 也会拦截 `sed-custom` 这类命令。

**链式命令。** Grok 像 shell 一样解析每条命令，并按 `&&`、`||`、`;`、`|` 和换行拆分。规则动作对段的处理不同：

- `deny` 和 `ask` 规则对每一段以及整串检查。一个被拒绝的段会拒绝整条命令。
- `allow` 规则是合取的：只有当**每一**段都独立匹配一条 allow 规则时，命令才由规则自动批准。`Bash(git *)` 批准 `git status && git diff`，但不批准 `git status && rm -rf /`——`rm` 段不匹配任何 allow 规则，因此命令落到该模式的正常处理（`default` 模式下提示；`auto` 模式下由分类器处理，仍可能批准或拦截；`dontAsk` 下拒绝）。因此单条 allow 规则永远不能批准夹带无关命令的链。

> **Allow 规则不是封闭的允许列表。** 不匹配任何 allow 规则的命令并不会因此被拒绝——它落到该模式。在 `auto` 模式下，分类器可以批准你的规则从未提及的命令。对于默认拒绝策略，使用 `dontAsk`（或始终批准加上作为硬拦截的 `deny` 规则），如 [配置权限](#configuring-permissions) 所述。

无法拆成简单段的命令（子 shell、命令替换 `$(...)`、反引号、后台 `&`、控制流）在配置了 Bash 限制时作为单个单元提示。

每段在匹配规则前会规范化。前导环境赋值如 `RUST_LOG=debug` 会被剥掉，一组固定的包装器（`timeout`、`nice`、`ionice`、`chrt`、`stdbuf`、`env`）会被揭开，因此规则匹配内层命令：`Bash(npm test *)` 批准 `RUST_LOG=debug timeout 30 npm test --workers=4`。这适用于 `deny`、`ask` 和 `allow` 规则、记住的授权，以及只读命令列表。

更多匹配细节：

- 规则也适用于传给 `bash -c` 的字面脚本。对 `allow`，该脚本里的每条命令本身都必须被允许。
- 不在列表上的包装器（`sudo`、`xargs`、`nohup`，…）不会被揭开。请写明确点名它们的规则。
- 当解析器无法安全揭开某种形式时（例如 `env -S`），命令会提示而不是匹配 `allow` 规则。
- 匹配看到的是用单空格连接的已解析词，没有 shell 引号。请按未加引号的命令写模式。

### 危险命令

内置列表（`rm`、`chmod`、`chown`、`chgrp`、`chattr`、`pkill`、`kill`、`killall`、`git push`）即使某段被记住的命令前缀或只读命令列表覆盖也会提示。配置里的显式 `allow` 规则会批准它们，始终批准模式会像对待其他命令一样自动批准它们；用 `deny` 规则无条件拦截它们。在把 `Bash(rm *)` 这类规则加为 allow 规则之前请仔细审阅。

### Read、Edit 和 Grep 规则

路径模式是 glob，在词法规范化后对照工具路径匹配（`.`/`..` 折叠；相对路径与会话工作目录拼接）。以 `~` 为前缀的工具路径按字面匹配——从不与工作目录拼接——因为工具只在权限检查之后才把 `~` 展开成家目录：

- `*` 和 `?` 不跨越 `/`；`**` 会。`Read(src/*)` 匹配 `src/main.rs` 但不匹配 `src/nested/mod.rs`；整棵树用 `Read(src/**)`。
- 光秃的文件名只匹配那条精确字符串。用 `**/.env` 匹配任意深度的 `.env`。
- 没有锚点前缀：模式里前导的 `//` 或 `~/` 当作字面 glob 文本。请改写绝对路径模式或 `**/` 模式。
- 因为匹配前会折叠 `.`/`..`，有根的模式不能被遍历逃逸：`Read(./**)` 范围是工作目录（`src/main.rs` 这类光秃相对路径会匹配；`./../../etc/passwd` 不会），`Read(src/**)` 留在 `src/` 下。无根模式（`*`，或像 `**/*.rs` 这样前导 `**`）有意在任意深度、任意位置匹配。
- `Read` 规则也管辖 `grep` 搜索；`Grep(...)` 规则只匹配 grep。
- 原生 Read/Edit/Grep 检查对 deny 和 ask 会跟随路径内符号链接到解析后的目标。只匹配解析目标的 allow 不会为工具参数授予 allow。
- 无法解析的路径内符号链接，在任何 deny 或 ask 文件规则适用于该工具时会提示。

`Read` 和 `Edit` deny 规则还适用于 shell 命令触及的文件路径（例如对拒绝路径的 `cat` 或 `sed`），包括以 `-c` 传给 `bash`、`sh`、`dash`、`zsh` 或 `ksh` 的字面内联脚本。shell 级检查使用与上面直接 Read/Edit/Grep 工具相同的、感知工作目录的规范化和 deny/ask 符号链接跟随（工作目录下的绝对操作数也会匹配 `Read(src/**)` 这类有根规则）。对于覆盖每个进程的操作系统级强制，把 deny 规则与沙箱组合（[18-sandbox.md](18-sandbox.md)）。

### MCP 规则

`MCPTool(...)` 模式匹配 `server__tool` 形式的完整 Grok 工具名，并支持 glob：`MCPTool(linear__*)` 匹配来自 `linear` 服务器的每个工具。Grok 工具名不带 `mcp__` 前缀。

`.claude/settings.json` 文件里使用的 `mcp__` 规则拼写也被接受并改写到同一匹配器上：`mcp__linear`（`linear` 服务器上的每个工具）、`mcp__linear__get_issue`（一个工具）、`mcp__linear__*`（该服务器上的每个工具），以及 `mcp__*`（每个 MCP 工具）。

### WebFetch 规则

- `WebFetch(domain:example.com)` 不区分大小写地匹配该主机和每个子域（`api.example.com`），忽略前导 `www.`。`domain:` 模式内不支持通配符。
- 没有 `domain:` 前缀的模式对整个 URL 做 glob：`WebFetch(https://api.example.com/*)`。

### 工具名称

已识别的工具名称：`Bash`、`Read`、`Edit`（以及 `Write`）、`Grep`（以及 `Glob`）、`MCPTool`、`WebFetch`、`WebSearch`。光秃的 `*` 规则匹配每个工具。工具名位置不支持 glob。

点名无法识别工具的规则（例如 `Agent(model:opus)`）会带着警告跳过，而不是让加载失败。

### 求值顺序

每个来源的规则合并成一套，按严重性而不是顺序求值：任何匹配的 `deny` 拒绝，否则任何匹配的 `ask` 提示，否则任何匹配的 `allow` 批准。没有规则匹配时，请求落到内置自动批准，然后是提示策略，如 [工具调用如何被授权](#how-a-tool-call-is-authorized) 所述。

---

## 交互式批准及其持久化位置

当工具调用需要批准时，权限提示提供这些选择：

- **Allow once**：批准这一次调用。
- **Reject once**：拒绝它，可选地带一条消息回给模型。
- **Enable always-approve mode**：批准之后所有工具调用，不只是正在提示的这一次。
- **Allow all edits this session**：对文件编辑显示。此授权只保存在内存中，重启后不存活。

### 按命令的 "Always Allow"

一组更窄的选项只记住正在提示的那条特定命令、MCP 工具或 web-fetch 域，例如 "Always allow `cargo test`"。这些行默认开启。用以下方式禁用它们：

```toml
# ~/.grok/config.toml
[ui]
remember_tool_approvals = false
```

组织可以通过 `requirements.toml` 或托管配置里的同一键禁用它们。门闩开启时（默认），提示会增加：

- **`Always allow: <command>`**，为该命令前缀持久化一条 allow。
- 对应的 "never allow" 行，以同样方式持久化一条 deny。
- MCP 工具和 web-fetch 域的等价 "always allow" 和 "never allow" 行。"never allow" 行始终记住正在提示的精确工具（从不是整个服务器）或精确域；记住的 deny 胜过任何授权，被拒绝的域也覆盖其子域。

记住的前缀限于命令的短形式：只读命令只持久化其列出的前缀（例如 `git status`，不是完整参数列表），其他命令持久化一段短的前导前缀。确认前，提示会准确显示将记住什么。

[危险命令](#dangerous-commands) 列表上的命令（例如 `git push` 和 `rm`）从不尊重记住的*前缀*：只有对整条命令的精确授权才算数，因此它们的 "Always allow" 行默认是完整命令。批准它只会停止对那次精确调用的提示；任何不同的参数会再次提示。当没有任何可记住的授权能阻止脚本再次提示时——危险命令藏在 `env` 前缀后面，或链里其他步骤仍需要批准——"Always allow" 行根本不会提供，而不是保存一条不会生效的规则。

### 持久化按项目

交互式授权存储在你家目录下 Grok 自己的状态目录里，范围是你启动 Grok 时所在的 git 仓库（其仓库根），因此在仓库根接受的授权也适用于从同一仓库子目录启动的会话。在 git 仓库外，授权范围是启动目录，每个 git 工作树保留自己的授权。在一个项目里做出的授权从不适用于另一个，授权不会写入仓库，也不打算手改。

要检查或重置项目的授权，打开 Grok 家目录（你家目录下的 `.grok` 目录，或 `$GROK_HOME`）的 `sessions` 子目录：那里的每个项目目录（URL 编码的范围根）都有一份 `permission.toml`（外加按客户端的 `permission_<client>.toml` 变体），列出记住的命令前缀、glob、MCP 工具/服务器、web-fetch 域，以及 "never allow" 条目。删除该文件会重置该项目的授权；下一次匹配的工具调用会再次提示。把它当作只读状态——要*添加*规则，请改用声明式的 `[permission]` 配置。

交互式授权是个人的、按机器的状态。对于可以在代码审阅里检查并与队友共享的允许列表，请改用项目 `.grok/config.toml` 里的声明式规则。

---

## 用 Hook 把 Bash 限制到特定命令

`PreToolUse` hook 可以对 `Bash` 工具强制允许列表，并在每一种权限模式下生效。Hook 在权限系统之前求值；hook deny 会停掉调用，hook allow 落到正常权限检查（因此你的 `deny` 规则仍生效）。

> **注意：** Hook 失败开放。若 hook 脚本崩溃、超时或缺失，工具调用会像 hook 已允许一样继续，失败会在 UI 里报告。用作安全边界的 hook 必须处理自己的错误，并且必须考虑链式命令，如下面的例子那样。见 [10-hooks.md](10-hooks.md)。

### 示例：只允许 `git` 和 `gh`

**`~/.grok/hooks/git-gh-only.json`**

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "git-gh-only.sh",
            "timeout": 5
          }
        ]
      }
    ]
  }
}
```

**`~/.grok/hooks/git-gh-only.sh`**

```bash
#!/bin/sh
# Allow only git and gh commands, including within chained commands.

set -eu

deny() {
  echo '{"decision": "deny", "reason": "'"$1"'"}'
  exit 2
}

INPUT=$(cat)
CMD=$(echo "$INPUT" | jq -r '.toolInput.command // empty')

[ -n "$CMD" ] || deny "Empty command is not allowed"

# Normalize '&&' and '||' to ';' so chains can be checked segment by
# segment, then reject constructs this script cannot inspect.
CMD=$(echo "$CMD" | sed 's/&&/;/g; s/||/;/g')
case "$CMD" in
  *'$('*|*'`'*|*'&'*|*'>'*|*'<'*) deny "Substitution, background, and redirection are not permitted" ;;
esac

# Split on the separators and require every segment to start with git or gh.
echo "$CMD" | tr ';|' '\n\n' | while IFS= read -r SEGMENT; do
  SEGMENT=$(echo "$SEGMENT" | sed 's/^[[:space:]]*//')
  [ -n "$SEGMENT" ] || continue
  case "$SEGMENT" in
    git\ *|git|gh\ *|gh) ;;
    *) deny "Only git and gh commands are permitted. Blocked segment: $SEGMENT" ;;
  esac
done
```

```bash
chmod +x ~/.grok/hooks/git-gh-only.sh
```

此 hook 拒绝每一条 `Bash` 命令，除非每个链式段都以 `git` 或 `gh` 开头，并直接拒绝命令替换、后台和重定向，因为它无法验证它们执行什么。它在每一种权限模式下都有效。

关于 hook 安装、JSON 格式、项目 hook 的信任模型，以及其他事件，见 [10-hooks.md](10-hooks.md)，其中还包含一个互补的「拦截危险模式」示例。

---

## 配置示例

### 无头且仅 git 和 gh（CI 与自动化）

```bash
grok -p "Implement the feature using only git and GitHub CLI" \
  --allow 'Read' \
  --allow 'Grep' \
  --allow 'Bash(git *)' \
  --allow 'Bash(gh *)'
```

安装上面的 `git-gh-only` hook 以拒绝其他每一条 `Bash` 命令。要对所有工具默认拒绝，还在 `.claude/settings.json` 里设 `{"permissions": {"defaultMode": "dontAsk"}}`。

### 只读代码审阅者

```toml
# .grok/config.toml
[permission]
rules = [
  { action = "allow", tool = "read" },
  { action = "allow", tool = "grep" },
  { action = "deny",  tool = "edit" },
  { action = "deny",  tool = "bash" },
]
```

### 交互式开发

使用 `default` 模式，再加上你最常跑的命令（`git`、`cargo test`、`rg` 等）的窄 `Bash(...)` allow 规则。

---

## 与沙箱组合

权限控制模型被允许请求什么。操作系统级沙箱（见 [18-sandbox.md](18-sandbox.md)）控制即使命令获批后进程仍能做什么。

对不受信任代码的推荐组合：

1. `dontAsk` 加上窄 allow 规则，或限制性 hook
2. `--sandbox strict` 或自定义配置
3. 项目信任，外加审阅任何 `SessionStart` hook

---

## 在 TUI 里管理权限

- 权限决定出现在 transcript 里。
- `/always-approve` 命令切换始终批准模式；其他模式通过 `defaultMode` 设置（见 [如何设置模式](#how-to-set-the-mode)）。
- 权限提示包含按命令的 "Always allow" 选项，只对当前项目持久化（默认开启；用 `[ui] remember_tool_approvals = false` 禁用）。见 [交互式批准](#interactive-approvals-and-where-they-persist)。
- 要管理 hook 和插件，运行 `/hooks` 或 `/plugins`（在大多数终端上，**Ctrl+L** 也会打开 Extensions 模态框；在 VS Code、Cursor、Windsurf 和 Zed 上，`Ctrl+L` 反而是回合中插入打断）。见 [10-hooks.md](10-hooks.md)。

---

## 最佳实践

1. **优先用窄模式。** `Bash(git *)` 授予的访问少于光秃的 `Bash` allow 规则。
2. **组合多层。** `dontAsk`、窄 allow 规则、限制性 hook 和沙箱各自独立限制。
3. **审阅来源不熟悉的项目配置。** 文件夹信任会门闩 `.grok/config.toml` 和 `.claude/settings.json` 里的项目权限规则，以及项目说明和 skills 的启动加载。带着这些来源的无头启动需要 `--trust` 或先前的授权。在信任不熟悉的检出之前审阅它们以及任何项目 hook（见 [10-hooks.md](10-hooks.md)）。
4. **测试你的策略。** 在设好 `defaultMode: "dontAsk"`（或安装了你的 `PreToolUse` hook）的情况下，跑有代表性的命令并确认什么被拦截。
5. **把只读命令列表当作便利，而不是安全边界。**

---

## 另见

- [Hooks](10-hooks.md) — PreToolUse 和其他生命周期脚本
- [无头模式](14-headless-mode.md) — 一次性 CLI 和自动化旗标
- [Agent 模式](15-agent-mode.md) — ACP、stdio 和 agent 服务器
- [沙箱](18-sandbox.md) — 操作系统级隔离配置
- [配置](05-configuration.md) — 原生 `config.toml` 结构
