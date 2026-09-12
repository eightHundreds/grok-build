# 沙箱模式

沙箱模式用操作系统内核原语（Linux 上的 Landlock，macOS 上的 Seatbelt）限制 agent 进程及其派生命令能访问的文件系统和网络。内核在进程生命周期内强制执行这些限制。

沙箱模式默认关闭。

---

## 快速开始

```bash
# Run with workspace sandbox (read everywhere, write to CWD + temp dirs + ~/.grok/)
grok --sandbox workspace

# Read-only mode (read everywhere, write only to ~/.grok/ + temp dirs)
grok --sandbox read-only

# Most restrictive profile (read CWD + system paths + ~/.grok, write CWD + ~/.grok/sessions + temp dirs, no child network)
grok --sandbox strict
```

---

## 内置配置

| 配置                  | 文件系统读                     | 文件系统写                                     | 子进程网络    | 适用场景                          |
| --------------------- | ------------------------------ | ---------------------------------------------- | ------------- | --------------------------------- |
| `off`（默认）         | 不受限                         | 不受限                                         | 不受限        | 无沙箱                            |
| `workspace`           | 任意位置                       | CWD + `~/.grok/` + `/tmp` + `/var/tmp`         | 允许          | 日常开发                          |
| `devbox`              | 任意位置                       | 除 `/data` 外的所有顶层目录                    | 允许          | 一次性开发虚拟机                  |
| `read-only`           | 任意位置                       | `~/.grok/` + `/tmp` + `/var/tmp`               | 拦截¹         | 探索、代码审阅                    |
| `strict`              | CWD + 系统路径 + `~/.grok`     | CWD + `~/.grok/sessions` + `/tmp` + `/var/tmp` | 拦截¹         | 不受信任的代码                    |

¹ 子进程网络拦截仅在 **Linux** 上强制执行（通过 seccomp）。在 macOS 上是空操作——这些配置在那里不限制子进程网络。

要在配置之上再拦截特定文件（例如 `.env` 或凭据路径），用带 `deny` 列表的[自定义配置](#custom-profiles)——由内核强制（读 + 写/重命名），并支持 `**/*.pem` 这类 glob 模式。

### 配置详情

**workspace** —— 日常开发的推荐配置。agent 可以读取系统上的任何文件（用于理解依赖、系统库等），但只能写入当前工作目录、`~/.grok/` 和临时目录（`/tmp`、`/var/tmp`，以及 macOS 临时目录）。网络访问对 `web_search` 和 MCP 服务器等工具是允许的。

**devbox** —— 为一次性开发虚拟机预留的内置配置。agent 可以到处读，并写入除 `/data` 和虚拟文件系统（`/proc`、`/sys`、`/dev`）以外的每个顶层目录，包括家目录。网络访问允许。`--sandbox devbox` 跑的是内置配置，会盖过你在 `sandbox.toml` 里定义的任何 `[profiles.devbox]`。

**read-only** —— 希望 agent 分析代码但不改你的项目文件时使用。agent 可以读一切，但只能写入 `~/.grok/`（会话持久化需要）和临时目录。子进程网络访问在 Linux 上被拦截（macOS 上空操作）。

**strict** —— 最严格的配置，用于审阅不受信任的代码。agent 可以读当前工作目录、必要的系统路径和 `~/.grok`。写入限于 CWD、`~/.grok/sessions` 和临时目录——不是整个 `~/.grok` 树。子进程网络访问在 Linux 上被拦截（macOS 上空操作）。

### 直接全局 hook 写保护

在 `workspace`、`read-only` 和 `strict` 下（以及扩展这些基座的自定义配置），内核对用作用户全局 hook 源的、由 Grok 拥有的直接磁盘路径实施**写拒绝**（在已授予读权限时它们仍可读）。内置 `strict` 可以读 `~/.grok`（hook 仍可读）；写入是 CWD + `~/.grok/sessions` + 临时目录，不是整棵树。在配置授予写权限的地方，写拒绝仍然生效：

- `~/.grok/hooks/`（hook 目录）
- `~/.grok/hooks-paths`（注册表文件；不会当作 hook JSON 加载——只有其中的绝对目标会被加载）
- `hooks-paths` 里列出的绝对目标（相对行会被忽略；缺失的目标会拒绝启动沙箱）

在这些配置下首次启动时，若它们缺失，Grok 会创建真正的空 `hooks/` 目录和空 `hooks-paths` 文件（从不用符号链接或错误类型）。Claude/Cursor 全局设置**不**受此写拒绝覆盖；发现那些厂商仍由兼容性设置单独门闩控制。

带符号链接的 `$GROK_HOME`，或带符号链接分量的 `hooks-paths` 条目，会在沙箱启动时被拒绝（防止改指向）。受保护路径的已有父目录会被钉住，使它们不能从拒绝之下被重命名走（兄弟项仍可写）。在 Linux 上，bubblewrap 内部会禁用嵌套用户命名空间，因此不能重排挂载绑定。项目 hook 仍由文件夹信任门闩控制。`devbox` 配置不应用此保护（一次性虚拟机）。需要该保护的配置若无法应用内核策略（包括 Linux 上没有经验证的只读挂载），会拒绝启动。

---

## 自定义配置

在 `~/.grok/sandbox.toml`（全局）或 `.grok/sandbox.toml`（按项目）创建自定义沙箱配置：

```toml
[profiles.project]
# Start from a built-in profile, then add overrides
extends = "workspace"
restrict_network = true

# Paths the agent can read but NOT write/delete
read_only = ["/data"]

# Additional writable paths
read_write = ["/tmp/scratch"]

# Paths or globs to kernel-deny (read + write/rename, enforced; see notes below)
deny = ["/data/shared-secrets", "**/.env", "**/*.pem"]
```

使用自定义配置：

```bash
grok --sandbox project
```

自定义配置不能复用内置名称。`--sandbox devbox` 始终跑内置 `devbox` 配置，盖过你定义的任何 `[profiles.devbox]`。

若用户文件和项目文件对同一自定义配置的定义不同，Grok 使用用户配置并显示启动警告。运行 `/doctor` 可查看两个文件位置以及如何解决冲突。定义完全相同则不产生警告。

### 自定义配置字段

| 字段               | 类型     | 说明                                             |
| ------------------ | -------- | ------------------------------------------------ |
| `extends`          | String   | 要继承的内置基座配置（`workspace`、`devbox`、`read-only`、`strict`）。省略时默认为 `workspace` |
| `restrict_network` | Boolean  | 拦截子进程的网络访问                             |
| `read_only`        | String[] | 额外的只读路径                                   |
| `read_write`       | String[] | 额外的读写路径                                   |
| `deny`             | String[] | 要内核拒绝的路径或 glob（读 + 写/重命名；见说明）。含 `*`、`?` 或 `[` 的条目是 glob |

> **关于 `read_only` / `read_write`：** 这些是**字面目录授予**，
> 不是 glob。末尾的 `/**`（或 `/*`）当作父目录，因此
> `…/cache/**` 授予 `…/cache`（光秃的 `/**` 授予 `/`）。去掉这些之后
> 仍含 `*`、`?` 或 `[` 的条目（例如 `/home/**/cache`，或字面名为
> `dir[1]` 的目录）会被跳过并给出警告——请列出你需要的
> 具体目录，或把 glob 放到 `deny` 下。带前导或尾随空白的条目
> 也会被跳过并给出警告：空白在字面路径里有意义，所以应修好
> 条目，而不是依赖修剪。

> **关于 `deny`：** 非空的 `deny` 列表由**内核强制执行**。被拒绝的路径
> 通过 macOS 上的 Seatbelt 和 Linux 上的 bwrap bind-over
> **拒绝读且拒绝写/重命名**，因此被拒绝的路径既不能被读（通过
> `bash`、`grep` 或子 agent），也不能被挪出拒绝集合再到别处读
> （`mv secret x && cat x` 这条绕过已被堵住）。在 **Linux** 上，读拒绝需要
> `bubblewrap`：若缺失（或任何一个 deny 路径无法绑定），Grok
> 会拒绝启动，而不是带着暴露的被拒绝路径运行（`devbox` 只对
> `/data` 写拒绝，仍会回退到 Landlock）。对**不在** `deny` 里的路径的写入
> 由你在 `read_write` 里授予的内容控制。

> **`deny` 里的 glob：** 条目若含 `*`、`?` 或 `[`，就是 **glob**。
> 这些字符**始终**表示 glob——要拒绝名字里含它们的字面文件，
> 请改为点名父目录。支持的、gitignore 风格子集是：
>
> - `*` —— 一个路径段内任意一串字符（在 `/` 处停下）
> - `?` —— 一段内恰好一个字符
> - `**` —— 跨越目录（作为整个路径段，例如 `**/`、`a/**`）；`**/`
>   也匹配零个目录，因此 `**/.env` 匹配 `.env` 和 `sub/.env`
> - `[abc]` / `[a-z]` —— 字符类；前导 `!` **或** `^` 取反
>   （`[!a]` 和 `[^a]` 都表示「不是 `a`」）
>
> 花括号交替（`{a,b}`）、反斜杠转义、空路径段（双写的
> `//` 或尾随 `/`）、`.` 或 `..` 段，以及不常见的类形式
> `[]…]`（字面 `]` 在前）和 POSIX `[[:…:]]` **不受支持**，
> 因此两个平台永远不会把同一个 glob 解释成不同样子。使用不受支持
> 元字符的 glob，或格式错误的 glob，会让 Grok 在**两个**平台上都
> **拒绝启动**（失败关闭）——把 `*.pem` 和 `*.key` 写成两条
> 而不是 `*.{pem,key}`。
>
> 相对 glob 锚定在工作区；绝对 glob（例如
> `/home/**/.ssh`）锚定在其字面前缀。非 glob 条目保持精确路径
> 匹配。相对 glob **只在工作区内**匹配。要拒绝
> 别处的文件，把条目写成绝对路径。除此之外，强制方式
> 因平台而异：
>
> - **macOS 是密闭的：** 每个 glob 变成运行时应用的 Seatbelt 正则，
>   因此匹配的文件会被拒绝，**即使是在 Grok 启动之后创建的**。
> - **Linux 是尽力而为：** 挂载命名空间无法在运行时做 glob，因此每个
>   glob 会展开成**启动时已存在**的文件并 bind-over。
>   **之后**创建且匹配 glob 的文件**不会**被覆盖——在 Linux 上
>   必须密闭的东西请写精确路径。匹配到的符号链接
>   会与其解析目标一起被遮住。匹配文件过多、或树太深太宽
>   无法扫描的 glob，会让 Grok **拒绝启动**而不是执行不足；
>   错误会点名这些 glob 以及扫描停下的目录。启动扫描从每个
>   glob 的字面前缀开始，并包含 gitignored 和隐藏文件，因此在
>   非常大的工作区上，优先用锚定 glob（`certs/**/*.pem` 只扫描
>   `certs/`）而不是光秃的 `**` 模式。

---

## 工作原理

沙箱在启动时用内核原语应用到**整个 grok 进程**——不是按命令包装。这意味着所有工具操作都被覆盖：

- `read_file`、`search_replace`、`list_dir` —— 由进程内的 Landlock/Seatbelt 限制
- `bash` 命令、`grep`（rg）—— 子进程自动继承文件系统限制
- 网络 —— 在 Linux 上可通过 seccomp 拦截子进程；在 macOS 上是空操作

当**请求**了非 `off` 的沙箱配置时（CLI、`GROK_SANDBOX`、配置，或托管要求）：

- agent **在本进程内**运行，不经过共享 leader，因此配置强制执行时工具调用留在本进程。若本来会开 leader 模式，启动时会有一行说明
- 若内置配置应用失败，Grok 会警告并无强制地继续（见 [平台支持](#platform-support)），但仍会拒绝 leader，以免工具被委派到别处
- `grok workspace start`、`restart` 和 `resume` 不可用；`pause`、`stop` 和 `status` 仍可用

要使用被拒绝的命令，请在选中该配置的源头关掉它。

沙箱一旦应用就**不可逆**。agent 不能在运行时放宽限制。

---

## 恢复会话

会话启动时使用的配置会随会话保存，并在**会话生命周期内固定**。恢复时（`grok --resume <id>`、
`grok --continue` 或 `grok -r`），Grok 会自动恢复同一配置——
因此用 `--sandbox workspace` 启动的会话不会默默回到更严的
默认值，并弄坏以前能跑的命令。

恢复**不会**改变会话的沙箱：

- 恢复时省略 `--sandbox` 会使用会话保存的配置。
- 传入与保存配置**相同**的 `--sandbox <profile>` 是允许的。
- 传入与保存配置**不同**的 `--sandbox <profile>` 会被
  **拒绝并报错**——更改已恢复会话的沙箱是安全隐患
  （可能放宽本应受限的访问，或弄坏依赖更宽访问的会话）。要使用
  不同配置，请开新会话。

**新**会话的配置解析顺序：

1. 显式的 `--sandbox <profile>` 旗标或 `GROK_SANDBOX` 环境变量
2. 配置里的 `[sandbox] profile`
3. `off`（无沙箱）

---

## 平台支持

| 平台  | 机制     | 最低版本               |
| ----- | -------- | ---------------------- |
| Linux | Landlock | Kernel 5.13 或更高     |
| macOS | Seatbelt | macOS（所有版本）      |

若沙箱无法应用（例如不支持的内核、缺少 entitlements），Grok 会记录警告并无强制地继续。例外是显式请求的**自定义配置**：在 **macOS 和 Linux** 上，若无法应用（未知配置、格式错误的 `sandbox.toml`，或——在 Linux 上——非空 `deny` 缺少 `bubblewrap`），Grok 会拒绝启动，而不是带着暴露的被拒绝路径运行。

---

## 网络限制

在 Linux 上，带 `restrict_network` 的配置通过 seccomp 拦截**子进程**（bash 命令、脚本）的网络访问。在 macOS 上，网络拦截是空操作。在进程内发 HTTP 请求的内置工具（网页搜索、LLM API 调用）从不受影响——agent 需要网络才能工作。

在 Linux 上实际意味着：

- `web_search`、`web_fetch` 和 LLM API 始终有网络访问
- 启用 `restrict_network` 时，`curl`、`wget` 和 `npm install` 这类 `bash` 命令会被拦截

---

## Shell 环境策略

沙箱控制子进程能到达哪些文件和网络。顶层 `[shell_environment_policy]` 表控制它继承哪些环境变量，这样模型跑的工具命令就不能读到恰好坐在你 shell 环境里的秘密。

```toml
[shell_environment_policy]
inherit = "core"                 # all (default) | core | none
ignore_default_excludes = false  # also drop *KEY* / *SECRET* / *TOKEN*
exclude = ["ACME_*", "CI_*"]     # drop these names
include_only = ["PATH", "HOME"]  # if set, keep only these names
set = { MY_FLAG = "1" }          # force these values
```

Grok 按顺序构建子环境：从 `inherit` 开始（`all` 保留一切，`core` 保留一小套平台变量如 `PATH` 和 `HOME`，`none` 从空开始）；除非 `ignore_default_excludes = true`，否则丢掉内置秘密模式 `*KEY*`、`*SECRET*` 和 `*TOKEN*`；丢掉任何 `exclude` 匹配；应用 `set`；当 `include_only` 非空时，只保留匹配的名字。模式是不区分大小写的 glob（`*`、`?`）。

默认（`inherit = "all"`、`ignore_default_excludes = true`）不改环境，因此在你配置策略之前什么都不变。在非持久后端上，策略也会过滤从你的登录 shell 捕获的变量，因此 `.rc` 文件的 export 不能绕过 `exclude` 或 `include_only` 塞进秘密。持久 shell 是一个例外：它把策略应用到基环境，但登录时 `.rc` 文件 export 的变量会从快照回放且不再过滤，所以请把秘密从那里的 shell 启动文件里拿掉。强制覆盖 macOS、Linux 和 Windows 上的 bash 工具和终端。

---

## 事件日志

沙箱事件会记到 `~/.grok/sessions` 供调试。事件包括：

- 已应用的配置（哪个配置、时间戳）
- 违规（试图访问被拒绝的路径）

---

## 何时使用沙箱模式

**在这些情况下用 `workspace`：**

- 在自己的项目上工作，想要基本的写保护
- 在共享环境中运行，想限制改动范围

**在这些情况下用带 `deny` 列表的自定义配置：**

- 需要在基座配置之上拦截特定文件（例如 `.env` 或凭据路径）
- 需要覆盖 `bash`、`grep` 和子 agent 的内核强制——不只是 `read_file` 工具

**在这些情况下用 `read-only`：**

- 审阅你不信任的代码
- 探索代码库且没有误改风险
- 跑代码分析或审计

**在这些情况下用 `strict`：**

- 分析不受信任或第三方代码
- 在对安全敏感的环境中运行
- 你想要最大隔离

**在这些情况下跳过沙箱：**

- agent 需要安装依赖（`npm install`、`pip install`）
- agent 需要修改工作目录外的文件
- 你在受信任的环境中工作，想要最大灵活性

---

## 权衡

| 方面     | 无沙箱                     | 有沙箱                          |
| -------- | -------------------------- | ------------------------------- |
| 安全     | agent 拥有完整系统访问     | agent 受配置规则限制            |
| 能力     | 可以做任何事               | 受配置限制                      |
| 性能     | 无额外开销                 | 开销可忽略                      |
| 恢复     | 必须信任 agent             | 内核强制边界                    |

沙箱在操作系统层强制限制——Linux 上通过 Landlock 或挂载命名空间，macOS 上通过 Seatbelt——不是单独的虚拟机。
