# 配置

Grok 从配置文件、环境变量和 CLI 标志读取设置。本页覆盖常用选项。`config.toml`、`managed_config.toml` 和 `requirements.toml` 的字段列表见 [26-config-reference.md](26-config-reference.md)（启动时提取到 `~/.grok/docs/user-guide/`）。

---

## 优先级

设置按最高优先级优先解析：

1. **CLI 标志**（例如 `--yolo`、`--model`、`--sandbox`）
2. **环境变量**（例如 `XAI_API_KEY`、`GROK_MEMORY`）
3. **`requirements.toml` / MDM**（组织强制；夹紧其下每一层配置，包括 overlay）
4. **`GROK_CONFIG` / `GROK_CONFIG_PATH` overlay**（高于 `config.toml` 和托管配置，低于 `requirements.toml` / MDM）
5. **config.toml**（`~/.grok/config.toml`）
6. **`managed_config.toml`**（组织部署的默认；低于 `config.toml`）
7. **内置默认**

在配置文件这一档内，各层从低到高合并：`managed_config.toml` → `config.toml` → `GROK_CONFIG` overlay → `requirements.toml` / MDM。因此 `requirements.toml` 和 MDM 会夹紧**你的 `config.toml` 和 overlay**。

`GROK_CONFIG` / `GROK_CONFIG_PATH`（第 4 档）是配置 **overlay**：一层合并后的配置，不是像 `XAI_API_KEY`（第 2 档）那样的直接设置环境变量。它们设置配置键（受下面的允许名单约束），因此应把它们读成配置文件档的一部分，而不是环境变量档。

### 用 `GROK_CONFIG` 注入配置

启动 `grok agent stdio` 的 harness 或 ACP 客户端可以注入设置，而不必写 `config.toml` 或搬迁 `$GROK_HOME`：

- **`GROK_CONFIG`**：内联 JSON 对象 overlay。
- **`GROK_CONFIG_PATH`**：*额外*的文件 overlay（不是替换 `config.toml`），按扩展名读取的 JSON 或 TOML 文件（`.json` → JSON，否则 TOML）。两者都设置时 `GROK_CONFIG` 优先。空的 `GROK_CONFIG` 当作未设置；格式错误的会记一条警告并落到 `GROK_CONFIG_PATH`。

overlay 在你的 `config.toml` 上做 **deep-merge**（只覆盖它设置的键），放在用户/托管层之上，但**低于** `requirements.toml` / MDM，因此企业钉住仍然赢。格式错误的 blob 会被忽略并记警告。这镜像了 `codex-acp` 适配器的 `CODEX_CONFIG`（合并进会话配置的 JSON 对象）；Grok 是 ACP 原生的，因此 overlay 住在 agent 自身。它只影响从合并配置读取的设置，并且**不是**权限升级路径。overlay 被限制、失败即关，只允许一份软设置的 **允许名单**（`models`、`features`、收窄的 `toolset`，以及仅限其过滤字段的 `shell_environment_policy`，这些字段在启动器已经控制的环境名中选择，不能把环境值注入工具子进程）；其他表都会在卡口丢掉，因此 overlay 不能派生命令、设置认证策略、重定向网络流量、提升信任或添加发现源。即使在允许名单的设置上，一组特定的安全门闩也读取原始磁盘层而不是 overlay。`ConfigLayers::env_overlay` rustdoc 是 overlay 能到、不能到以及哪些门闩无 overlay 读取的权威清单；另见 [内部环境变量参考](../internal/22-environment-variables.md)。无界面权限控制请用 `GROK_DEFAULT_SELECTED_PERMISSION`。例如，设置默认推理力度：

```bash
GROK_CONFIG='{"models": {"default_reasoning_effort": "high"}}' grok agent stdio
```

---

## config.toml（主配置）

位置：`~/.grok/config.toml`。若文件缺失，Grok 使用内置默认，因此你只需设置想覆盖的值。

### 常规设置

```toml
[cli]
auto_update = true                     # 启动时检查更新

[models]
default = "grok-4.5"                   # 新会话使用的模型
web_search = "grok-4.5"                # web_search 工具使用的模型
# 可选的选择器允许名单（对目录键或模型 id 的 glob）。空 = 不限制。
# 已签名的策略钉住会替换此列表（仅模型 id），且不能从这里加宽。
# allowed_models = ["grok-4.5", "grok-4*"]

# 应用到每个模型的默认；按模型的 [model.<id>] 值始终优先。
# 按模型覆盖和完整细节见「自定义模型」。
extra_headers = { "X-Request-Tags" = "team=example,env=prod" }
extra_body = { provider_tag = "global-default" }
temperature = 0.7
top_p = 0.95
max_completion_tokens = 8192
max_retries = 8
inference_idle_timeout_secs = 600
subagent_rate_limit_max_attempts = 8
stream_tool_calls = true

[ui]
simple_mode = true                     # readline 风格提示编辑（默认）；false = 提示框里的 vim 编辑
vim_mode = false                       # vim 风格回看导航键（默认：false）
max_thoughts_width = 120               # 推理显示的最大列宽
default_selected_permission = "always_allow_all_sessions" # 第一次批准提示上预选的行
remember_tool_approvals = true         # 在权限提示上显示按命令的「Always allow」选项；
                                       # 授权按项目记住（默认：true）；见 22-permissions-and-safety.md
show_thinking_blocks = true            # 在 TUI 中显示 agent 思考块（默认：true）
group_tool_verbs = true                # 把连续的读/搜索/列出工具调用和子 agent 行
                                       # ——以及其中已完成的思考——折成一行（默认：true）
collapsed_edit_blocks = false          # 把编辑显示为一行 +N/-M diffstat 摘要，并合并
                                       # 同一文件连续的编辑为一行，展开看
                                       # diffs（默认：false；pager.toml [scrollback.blocks.edit]
                                       # 的 expanded_by_default/line_summary 覆盖其折叠形态）
page_flip_on_send = true               # 把刚发送的提示钉在视口顶部，让
                                       # 回复从新的一页开始（默认：true）；设为 false
                                       # 则发送从不移动滚动位置
follow_up_behavior = "queue"           # 回合中后续："queue"（等回合结束；默认）或
                                       # "steer"（普通 Enter 仍可见地排队，然后在
                                       # 下一个工具/模型安全间隙注入）。见键盘快捷键 → 回合中。
screen_mode = "fullscreen"             # 默认渲染模式："fullscreen" | "minimal"
                                       # （未设置 → fullscreen）；通过 /settings → Default screen mode 设置

[features]
telemetry = false                      # 匿名用量遥测
feedback = true                        # 反馈系统（默认：true）
lsp_tools = false                      # 暴露 lsp 工具
codebase_indexing = true               # 代码图索引（默认：true）
two_pass_compaction = true             # 预触发两遍压缩（默认：true）
remote_fetch = true                    # 允许可选的在线模型目录抓取（默认：true；
                                       # 防火墙/隔离部署设为 false；后台
                                       # 托管配置同步有自己的开关：managed_config）

[session]
auto_compact_threshold_percent = 85    # 上下文窗口达到此百分比时自动压缩（默认：85）
load_envrc = true                      # 加载 .envrc 环境变量

[tools]
respect_gitignore = false              # 默认：false；设为 true 让每个工具跳过被 gitignore 的文件

# 单次模型步骤中并行媒体生成的可选上限。
# 按工具名。第一次 2× 或更多的突发：丢弃该步并重试一次。
# 任何其他超上限（包括第二次 2× 突发）保留前 K 个。
# 默认：图片 8，视频 4。
# 环境变量 GROK_MAX_PARALLEL_IMAGE_GEN_CALLS / GROK_MAX_PARALLEL_VIDEO_GEN_CALLS
# 覆盖这些值（见环境变量文档）。
# [tools.media_gen]
# max_parallel_image_gen_calls = 8
# max_parallel_video_gen_calls = 4
```

#### 输入模式

`[ui] simple_mode` 控制你如何在**提示框**——输入编辑器——里编辑文本。它与你如何在回看区移动无关；那是 [`vim_mode`](#vim-mode)。

| 值 | 行为 |
|-------|----------|
| `true`（默认） | **Readline 编辑。** 普通 readline 风格文本输入。 |
| `false` | **Vim 编辑（实验性）。** Vim 风格模态编辑（普通和插入模式）。提示为空时从普通模式开始，焦点在回看区。 |

要把提示框切到 vim 风格编辑：

```toml
[ui]
simple_mode = false
```

你也可以从设置窗格翻转它（`/settings` → **Disable vim input mode**）；Grok 会把你的选择写到 `[ui] simple_mode`。`simple_mode` 和 `vim_mode` 相互独立——一个管提示编辑器，另一个管回看导航。完整绑定参考见 [键盘快捷键](03-keyboard-shortcuts.md)。

#### 默认选中权限

当 agent 请求运行命令（或采取其他工具动作）时，批准菜单默认高亮一行。`[ui] default_selected_permission` 设置会话**第一次**提示时是哪一行。

| 值 | 预选行 |
|-------|-----------------|
| `always_allow_all_sessions`（默认） | 「Always allow on all sessions」行。 |
| `allow_command_always` | 「Always allow this command」行。 |
| `allow_once` | 「Yes」/ 仅允许一次行。 |
| `reject` | 拒绝行。 |

```toml
[ui]
default_selected_permission = "allow_once"
```

你回答第一次提示后，光标变成**粘性**：之后每次提示预选你上次确认的那一项（选一次「No」，后续提示从拒绝行开始），跨编辑 / bash / MCP 提示持续，直到重启。因此此设置只选起点。

值不区分大小写；未设置或无法识别的值回退到 `always_allow_all_sessions`。`allow_command_always` 行始终限定到正在批准的具体动作（命令 / 工具 / 域 / 编辑会话），从不是全局允许一切——那是 `always_allow_all_sessions` 的事。注意按命令的「Always allow」行在 `[ui] remember_tool_approvals` 启用时出现（默认；设为 `false` 可隐藏）。见 [22-permissions-and-safety.md](22-permissions-and-safety.md)。

你也可以用 `GROK_DEFAULT_SELECTED_PERMISSION` 覆盖，这对不应改写 `config.toml` 的无界面或 agent 测试运行很方便。优先级：环境变量 → `config.toml` → `always_allow_all_sessions`。

#### Vim 模式

`[ui] vim_mode` 控制 **回看区** 窗格是否启用 vim 风格绑定。它不影响提示框。

| 值 | 行为 |
|-------|----------|
| `false`（默认） | 回看区里抑制裸字母和 `Shift+letter` 键（`j`/`k`、`h`/`l`、`g`/`G`、`y`/`Y`、`o`/`O`、`r`、`x`、`e`/`E`、`H`/`L`，以及 `i`）：按其中一个会聚焦提示框并打出该字符。方向键、`Tab`、`Space`、`PageUp`/`PageDown` 以及所有 `Ctrl+letter` 快捷键仍可导航。`Esc` **不是**回看键——它从不取消正在运行的回合（那是 `Ctrl+C`），空闲时遵循清空 / rewind 策略（见 [键盘快捷键](03-keyboard-shortcuts.md#escape)）。 |
| `true` | 所有 vim 风格回看绑定都启用，与 [键盘快捷键](03-keyboard-shortcuts.md) 所列完全一致。两种设置下 Esc 行为相同。 |

运行时用 `/vim-mode` 切换，或从 `/settings` → **Vim scrollback navigation**。Grok 立即把变更写到 `[ui] vim_mode`，并应用到之后每一个 pager 会话，包括同一进程中的新 agent 和子 agent。没有按会话覆盖——下次启动时 `config.toml` 是真相来源。`vim_mode` 与 `simple_mode` 相互独立。

#### 屏幕模式

`[ui] screen_mode` 是直接运行 `grok` 的**默认渲染模式**。从 `/settings` → **Default screen mode** 设置（需要重启）或手工编辑 `config.toml`——两者都写文件。CLI 标志（`--minimal` / `--fullscreen`）和斜杠命令（`/minimal` / `/fullscreen`）是会话范围，**不会**写此键；斜杠切换后，反向命令只把该会话带回去。

| 值 | 行为 |
|-------|----------|
| 未设置 | 设置显示 **Fullscreen**。启动时没有粘性偏好：遗留的 `pager.toml` `[terminal] minimal` 仍可强制极简，泄漏鼠标报告的终端（JediTerm/Windows）可能自动打开极简，直到你设显式值。否则 alt-screen 策略选择全屏 vs 行内。 |
| `"fullscreen"` | 粘性非极简。全屏 vs 行内仍遵循 alt-screen 策略（`--no-alt-screen`、`[terminal] alt_screen`、终端自动检测）。 |
| `"minimal"` | 粘性极简（回看区原生）模式。 |

CLI 标志对该次调用始终压过配置值。

#### 发送时把提示吸到顶部

默认情况下，发送提示会把它滚到视口顶部，让回复从新的一页开始。设置 `[ui] page_flip_on_send = false`（或在 `/settings` → Appearance 中切换 **Snap prompt to top on send**）可在发送时保持滚动位置不动。下次发送即生效——无需重启。

#### 滚动

四个 `[ui]` 设置调节滚轮和触控板滚动。全部立即生效，并可从设置窗格编辑（`/settings` → **Scroll speed** / **Scroll input** / **Scroll lines** / **Invert scroll**）。

| 键 | 取值（默认） | 行为 |
|-----|------------------|----------|
| `scroll_speed` | `1`–`100`（`50`） | 滚轮和触控板的速度倍率。`50` = 1.0x，`1` = 0.1x，`100` = 6.0x。 |
| `scroll_mode` | `auto` \| `wheel` \| `trackpad`（`auto`） | 滚轮 vs 触控板检测是启发式的（终端滚动事件不带幅度）；自动检测误读设备时强制其一——例如一格滚轮跳太远，或触控板感觉一格一格的。 |
| `scroll_lines` | `1`–`10`（未设置） | 每次滚动滴答的行数，**同时**应用于滚轮和触控板。未设置时使用各终端自己的配置（例如 tmux 下保守的 1 行/事件）。提交任何值——即使是设置窗格显示的 `3`——都会永久切到该显式覆盖。 |
| `invert_scroll` | `false` \| `true`（`false`） | 反转垂直滚动方向（「自然」滚动）。 |

```toml
[ui]
scroll_speed = 50
scroll_mode = "auto"     # auto | wheel | trackpad
invert_scroll = false
# scroll_lines 默认未设置：由各终端配置负责。
# scroll_lines = 3
```

每个设置也有环境变量覆盖，仅在首次加载时应用（同样便于无界面 / 测试运行）：`GROK_SCROLL_SPEED`、`GROK_SCROLL_MODE`、`GROK_INVERT_SCROLL`（`1`/`true`/`0`/`false`）和 `GROK_SCROLL_LINES`。优先级：环境变量 → `config.toml` → 默认。无法识别的值回退到默认，超出范围的数字会被夹紧。

### 工具配置

```toml
[toolset.bash]
timeout_secs = 120.0                   # 前台命令超时（秒）（默认：120）
output_byte_limit = 20000              # 捕获输出的最大字节数（默认：20000）

[toolset.ask_user_question]
timeout_enabled = true                 # false = 永远等待答案（默认：true）
timeout_secs = 1800                    # 启用时等待的秒数（默认：1800 / 30 分钟）

[toolset.web_fetch]
proxy_endpoint = "https://proxy.example.com"   # 出口代理 URL
allowed_domains = ["docs.rs", "x.ai"]          # 覆盖内置允许名单
allow_local = false                            # true = 仅允许 localhost / 127.0.0.0/8 / ::1

[toolset.web_search]
# 把 web_search 限制到这些域（最多 5 个）。与 excluded_domains 互斥。
allowed_domains = ["docs.x.ai", "arxiv.org"]
# ……或改为屏蔽这些域（让 allowed_domains 保持未设置）：
# excluded_domains = ["reddit.com", "pinterest.com"]
```

`allow_local` 默认关闭（SSRF 失败即关）。打开它（或设 `GROK_WEB_FETCH_ALLOW_LOCAL=1`）后，`web_fetch` 只能到达**显式**回环主机——私有、链路本地和云元数据范围仍被挡住。解析：TOML > 环境 > 默认关闭。

`[toolset.web_search]` 约束 `web_search` 工具的域——搜索本身运行时的允许名单/屏蔽名单（不是事后过滤）。`allowed_domains` 和 `excluded_domains` **互斥**；若两者都设，允许名单赢，屏蔽名单会丢掉并记警告。空或缺失的列表不设界。这同时适用于后端托管搜索（带服务端搜索的模型）和客户端回退。已配置的策略是**权威的**：模型无法绕过——只要你在这里设置了 `allowed_domains` 或 `excluded_domains`，模型自己每次调用的 `allowed_domains` 都会被忽略（因此屏蔽名单是真屏蔽）。仅当你什么都没配置时，模型每次调用的允许名单才生效。解析：requirements → 用户 `config.toml` → 托管 → 默认（未设置）。配置在会话开始时读取，因此在启动会话前编辑——变更不会在会话中途生效。

`[toolset.ask_user_question]` 在 **requirements.toml**、**托管配置**和你的用户 **`config.toml`** 中都会被遵守。优先级：requirements → 环境（`GROK_ASK_USER_QUESTION_TIMEOUT_ENABLED` / `GROK_ASK_USER_QUESTION_TIMEOUT_SECS`）→ 用户配置 → 托管 → 默认。在用户配置里设 `timeout_enabled = false` 可为自己关闭自动问卷超时；`timeout_secs` 必须是正整数。你也可以从 `/settings` → **Ask-Question timeout**（在 Agent & Approval 下）切换 `timeout_enabled`；变更应用于新启动的会话。

### 认证

完整说明见 [认证](02-authentication.md)。

```toml
[auth]
auth_provider_command = "/usr/local/bin/my-auth-provider"
auth_provider_label = "Acme Corp"
auth_token_ttl = 3600

[grok_com_config.oidc]
issuer = "https://acme.okta.com"
client_id = "0oa1b2c3d4e5f6g7h8i9"
# scopes = ["openid", "profile", "email", "offline_access", "api:access"]
# audience = "https://api.acme.com"
```

### 自定义模型

添加自定义模型端点，以使用替代提供方或自托管模型。

```toml
[model.my-model]
model = "model-id"                    # 发给 API 的模型标识
base_url = "https://api.example.com/v1"  # OpenAI 兼容端点
name = "Display Name"                 # 显示在模型选择器中
description = "Model description"      # 可选
api_key = "sk-..."                    # 此提供方的 API key
env_key = "XAI_API_KEY"               # 保存 API key 的环境变量；字符串或数组（第一个已设置且非空的赢）
keychain_service = "grok"             # 系统钥匙串 service（可选；需同时设 keychain_account）
keychain_account = "new-api"          # 系统钥匙串 account（可选）
temperature = 0.7                     # 采样温度（0.0-2.0）
top_p = 0.95                          # nucleus 采样参数
max_completion_tokens = 8192          # 每次回复的最大 token
context_window = 128000               # 上下文窗口大小（用于自动压缩）
query_params = { api-version = "2026-07-22" } # 追加到每个请求 URL 的查询参数
env_http_headers = { "X-Tenant" = "TENANT_TOKEN" }    # 来自环境变量的请求头，在构建客户端时解析
extra_body = { enable_thinking = true }               # 合并进推理请求 JSON body 的额外字段
```

凭据解析：`api_key` > `env_key` > 钥匙串（`keychain_service` + `keychain_account`）> 已登录会话 token > `XAI_API_KEY`。`query_params`、`env_http_headers` 和 `extra_body` 见 [自定义模型](11-custom-models.md#request-query-parameters)，限制工具子进程继承哪些环境变量的 `[shell_environment_policy]` 见 [沙箱模式](18-sandbox.md#shell-environment-policy)。

要覆盖内置模型，用它的名字作为节键，并只设置你需要的字段：

```toml
[model.grok-4.6]
api_key = "my-api-key"
```

### MCP 服务器

通过 Model Context Protocol 配置外部工具集成。

```toml
[mcp_servers.github]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-github"]
env = { GITHUB_PERSONAL_ACCESS_TOKEN = "ghp_xxx" }
enabled = true                        # 启用/禁用（默认：true）
startup_timeout_sec = 30              # 初始化超时（秒）（默认：30）
tool_timeout_sec = 6000              # 工具调用超时（秒）（默认：6000）
tool_timeouts = { create_issue = 120 }  # 按工具的超时覆盖

[mcp_servers.postgres]
command = "npx"
args = ["-y", "@modelcontextprotocol/server-postgres", "postgresql://user:pass@localhost/db"]

[mcp_servers.my-streamable-server]
url = "https://mcp.example.com/api/mcp"  # HTTP/SSE 传输
headers = { "x-mcp-session-id" = "{{session_id}}" }
```

远程（HTTP/SSE）服务器会收到默认的 `User-Agent: grok-cli/<version>` 头；
`headers` 里有效的 `User-Agent` 条目会覆盖它（Figma 服务器收到裸的
`grok-cli`）。详见 [MCP 服务器](07-mcp-servers.md)。

MCP 服务器也可以在 `.grok/config.toml` 中按项目设置。项目范围配置贡献 `[mcp_servers]`、`[plugins]` 和 `[permission]` 规则；其他节只从 `~/.grok/config.toml` 加载。

`[mcp_servers]` 和 `[plugins]` 的优先级：`.grok/config.toml`（当前目录）> `<repo-root>/.grok/config.toml` > `~/.grok/config.toml`。`[permission]` 规则不按优先级覆盖——它们跨所有文件合并，`deny` > `ask` > `allow`（见 [22-permissions-and-safety.md](22-permissions-and-safety.md)）。

### 记忆

跨会话持久化知识。用 `GROK_MEMORY=1`、`[memory] enabled = true` 或托管远程设置启用记忆。

```toml
[memory]
enabled = false                       # 启用记忆

[memory.session]
save_on_end = true                    # 会话结束时写入元数据摘要

[memory.watcher]
enabled = true                        # 监视记忆文件的外部编辑

[memory.search]
max_results = 6                       # 默认结果数
min_score = 0.7                       # 最低相关性分数

[memory.initial_injection]
enabled = true                        # 第一回合自动注入记忆
min_score = 0.9                       # 第一回合注入的分数阈值

[memory.embedding]
# model 默认未设置，因此检索只用全文搜索
dimensions = 1024                     # 向量维度
```

### 子 agent

```toml
[subagents]
enabled = true
sampling_limit = 12                   # 每进程并发进行中的子 agent 采样调用；未设置时默认为 max_concurrent（32）（GROK_SUBAGENT_SAMPLING_LIMIT）

[subagents.toggle]
explore = true                        # 启用/禁用特定类型
plan = false

[subagents.models]
explore = "grok-4.6"               # 路由到不同模型
```

要钉住子 agent 使用的模型，在 `[subagents.models]` 下设置其条目。

### 目标模式与后台工作流

`/goal` 有两个驱动，由后台工作流设置选择。工作流启用时，宿主拥有的工作流引擎评估各轮并驱动完成验证；禁用时，`/goal` 回退到旧的面向模型的 `update_goal` 工具。`/goal` 是否可用是另一把开关（目标功能设置）。

后台工作流——`workflow` 工具、具名 `.grok/workflows/*.rhai` 脚本、`/deep-research` 和 `/workflow` 启动——**默认开启**。用配置、环境或远程设置关闭。

```toml
[workflows]
enabled = false                       # 关闭后台工作流（或 GROK_WORKFLOWS=0）
```

项目工作流从 `<repo-root>/.grok/workflows/` 发现；用户工作流从 `~/.grok/workflows/`。发现和调用按脚本的 `meta.name` 键控，因此让每个文件名与其 `meta.name` 对齐。内置赢过项目名，项目名赢过用户名，因此跨范围保持名称唯一。

每次启动得到一个会话唯一的显示句柄，例如 `deep-research-2`。你在 `/workflow runs` 仪表盘里看到的、以及传给 `/workflow pause`、`resume` 或 `stop` 的就是这个句柄——内部运行 ID 从不出现在命令里。编号句柄不是可复用的定义名，因此仪表盘会禁用 **save**，直到你选一个新的唯一 `meta.name` 并自己保存编辑后的脚本。示例见 [斜杠命令](04-slash-commands.md)。

### 技能

```toml
[skills]
paths = ["~/my-team-skills"]          # 额外要扫描的目录
ignore = ["~/my-team-skills/wip"]     # 要排除的路径
disabled = ["wip-skill"]              # 保持列出但不活动的技能名
```

### Harness 兼容

控制 Cursor、Claude 和 Codex 的厂商兼容。每个单元默认都是 `true`。会话单元保持暂存且惰性，直到外部会话扫描器消费它们；每个工具同时需要其 `sessions` 单元以及匹配的 `resume-claude`、`resume-codex` 或 `resume-cursor` 技能——缺少技能意味着零外部会话文件系统 I/O。

```toml
[compat.cursor]
skills = true     # 扫描 ~/.cursor/skills/ 和 <cwd>/.cursor/skills/
rules = true      # 扫描 ~/.cursor/rules/ 和 <dir>/.cursor/rules/
agents = true     # 扫描 ~/.cursor/ 中的具名说明文件
mcps = true       # 扫描 ~/.cursor/mcp.json 和 <cwd>/.cursor/mcp.json
hooks = true      # 扫描 ~/.cursor/hooks.json 和 <cwd>/.cursor/hooks.json
sessions = true   # 暂存；尚无扫描器消费者

[compat.claude]
skills = true     # 扫描 ~/.claude/skills/ 和 <cwd>/.claude/skills/
rules = true      # 扫描 ~/.claude/rules/ 和 <dir>/.claude/rules/
agents = true     # 扫描 ~/.claude/ 和 <dir>/.claude/CLAUDE*.md
mcps = true       # 扫描 ~/.claude.json 中的 MCP 服务器
hooks = true      # 扫描 ~/.claude/settings.json 中的 hooks
sessions = true   # 暂存；尚无扫描器消费者

[compat.codex]
sessions = true   # 暂存；尚无扫描器消费者
```

Codex 的 `skills`、`rules`、`agents`、`mcps` 和 `hooks` 单元是保留的，目前惰性——它们不会启用 `.codex` 发现。

对 Claude 和 Cursor，`rules` 和 `agents` 相互独立：关掉具名说明文件不会禁用家目录或项目规则目录，关掉规则也不会禁用具名文件。Claude 的 `agents` 单元门控家目录级 `~/.claude/` 具名文件和项目 `<dir>/.claude/CLAUDE*.md`；通用顶层 `Claude.md`、`CLAUDE.md` 和 `CLAUDE.local.md` 仍被识别。项目规则路径从仓库根到当前目录的每一层都会扫描。

每个单元都可通过环境变量或 `config.toml` 设置；名称见环境变量参考。解析：环境变量 > config.toml > 默认（开）。

`grok inspect` 把仍需会话启动解析的单元报告为 `?`，直到有值可用；带显式环境或 TOML 值的单元使用该值。受影响的发现条目在 JSON 中报告 `compatibilityStatus: "unresolved"`，在人类输出中报告 `[compat unresolved]`。

### 插件

```toml
[plugins]
paths = ["~/my-plugins/custom-tools"]
disabled = ["user/a1b2c3d4/noisy-plugin"]
```

### 提示偏好

`[hints]` 保存小型持久化 UI 偏好：记住的答案和模态框布局。Grok 会在你使用 TUI 时为你写入这些，但你可以手工编辑或删除；去掉一个键即恢复默认。

`[hints]` 从**有效配置合并**读取，优先级照常：系统托管 → 用户 `managed_config.toml` → 用户 `config.toml` → 用户 `requirements.toml` → 系统 `requirements.toml`，高层赢。TUI 只会把这些**写**到你的用户 `~/.grok/config.toml`。

```toml
[hints]
memory_modal_fullscreen = false        # 记住记忆模态框的全屏状态
new_session_worktree_mode = "never"    # /new 工作树提示："ask" | "always" | "never"
fork_worktree_mode = "ask"             # /fork 工作树提示："ask" | "always" | "never"
```

| 键 | 类型 | 默认 | 说明 |
|-----|------|---------|-------------|
| `memory_modal_fullscreen` | bool | `false` | 记住记忆模态框上次是否全屏打开。 |
| `new_session_worktree_mode` | string | `"never"` | `/new` 的工作树提示：`ask` 显示弹窗，`always` 创建工作树，`never` 跳过。 |
| `fork_worktree_mode` | string | `"ask"` | `/fork` 的工作树提示：`ask`、`always` 或 `never`。 |

### 通知

当 agent 完成一个回合或需要批准时发出终端通知。它们使用终端原生协议（OSC 9、OSC 99、OSC 777 或 BEL），默认按焦点门控，因此只在你没看终端时触发。

```toml
[ui.notifications]
method = "auto"           # auto|osc9|osc99|osc777|bel|none
condition = "unfocused"   # unfocused|always|never
idle_threshold_secs = 3   # 失焦多少秒后发出通知
events = ["turn_complete", "approval_required"]
sleep_prevention = true   # agent 回合期间防止显示器休眠
progress_bar = true       # 显示标签进度条（OSC 9;4）

[ui.notifications.title]
enabled = true
items = ["action-required", "spinner", "activity", "session-name", "grok"]
```

| 选项 | 类型 | 默认 | 说明 |
|--------|------|---------|-------------|
| `method` | string | `"auto"` | 通知协议。`auto` 为你的终端挑选最佳。 |
| `condition` | string | `"unfocused"` | 何时通知：`unfocused`（仅终端失焦时）、`always` 或 `never`。 |
| `idle_threshold_secs` | integer | `3` | 发出通知前最少失焦秒数。 |
| `events` | array | `["turn_complete", "approval_required"]` | 触发通知的事件。选项：`turn_complete`、`approval_required`、`session_ready`、`task_complete`、`agent_error`。 |
| `sleep_prevention` | bool | `true` | agent 工作时保持显示器唤醒（macOS/Linux）。 |
| `progress_bar` | bool | `true` | 在终端标签显示进度指示（OSC 9;4）。 |
| `title.enabled` | bool | `true` | 设置终端标题以反映 agent 状态。 |
| `title.items` | array | （见上） | 标题栏显示的项。选项：`action-required`、`spinner`、`activity`、`session-name`、`cwd`、`model`、`turn-timer`、`grok`。 |

#### 终端支持矩阵

| 终端 | 自动协议 | 焦点跟踪 | 进度条 |
|----------|---------------|----------------|--------------|
| iTerm2 | OSC 9 | 是 | 是 |
| Kitty | OSC 99 | 是 | 否 |
| Ghostty | OSC 777 | 是 | 是 |
| WezTerm | OSC 9 | 是 | 是 |
| Warp | OSC 9 | 是 | 否 |
| Alacritty | BEL | 是 | 否 |
| VS Code | BEL | 是 | 否 |
| Apple Terminal | BEL | 否 | 否 |
| VTE（GNOME Terminal） | OSC 777 | 是 | 否 |
| Grok Desktop | 无（原生） | 不适用 | 不适用 |
| 未知 | BEL | 否 | 否 |

`method = "auto"` 时，Grok 检测终端品牌并挑选最佳协议。显式设置 `method` 可覆盖。

#### 通知 hooks

事件触发时运行你自己的命令。hooks 在环境中收到 `$GROK_EVENT`、`$GROK_MESSAGE` 和 `$GROK_SESSION_ID`。

```toml
# macOS 原生通知
[[ui.notifications.hooks]]
command = "terminal-notifier -title 'Grok' -message '$GROK_MESSAGE'"
events = ["turn_complete", "approval_required"]
only_unfocused = true
timeout_secs = 10

# 推送到 ntfy 服务器
[[ui.notifications.hooks]]
command = "curl -s -d '$GROK_MESSAGE' ntfy.sh/my-grok-alerts"
events = ["turn_complete"]
only_unfocused = true
timeout_secs = 10

# 播放声音
[[ui.notifications.hooks]]
command = "afplay /System/Library/Sounds/Glass.aiff"
events = ["turn_complete"]
only_unfocused = true
timeout_secs = 5
```

| Hook 选项 | 类型 | 默认 | 说明 |
|-------------|------|---------|-------------|
| `command` | string | （必需） | 要运行的 shell 命令。 |
| `events` | array | `[]` | 触发此 hook 的事件（空 = 所有事件）。 |
| `only_unfocused` | bool | `true` | 仅在终端失焦时触发。 |
| `timeout_secs` | integer | `10` | 这么多秒后杀掉 hook 进程。 |

#### 排障

在受影响的会话里运行 `/doctor`。它显示检测到的通知和焦点问题、相关配置文件，以及解决步骤。显式的 `method = "bel"` 视为有意为之。`method = "none"` 关闭通知和焦点发现。

**防止休眠未生效：** 在 macOS 上，防止休眠通过 CoreFoundation 使用 `IOPMAssertionCreateWithName`；在 Linux 上使用 `systemd-inhibit`（必须在 `$PATH` 上）。确保相关工具可用。防止仅在 agent 回合期间生效，回合结束时自动释放。

### 状态行

全屏 pager 底部的可选一行，默认关闭。用 `[ui.status_line]` 开启：

```toml
[ui.status_line]
type = "builtin"                # builtin | command | disabled
items = ["cwd", "model", "context"]
```

其他键是 `items`（按顺序显示哪些内置段）、`command`、`padding` 和 `refresh_interval`（秒；按定时器重跑 `command` 行，让事故页或 CI 状态能到达空闲会话）。[状态行指南](25-status-line.md) 记录了全部内容，以及 `command` 脚本在 stdin 上读取的 JSON 约定和示例脚本。

极简模式没有状态行；它改用终端标签标题（见 [通知](#notifications) 的 `title.items`）。

### 键盘快捷键

键盘快捷键**不可**配置——所有绑定都是内置的。完整参考见 [键盘快捷键](03-keyboard-shortcuts.md)。

### 遥测

这些是相互独立的旋钮（见 [用量监控](24-monitoring-usage.md#related-settings)）：

- **`[features] telemetry`** / `GROK_TELEMETRY_ENABLED` —— 产品分析总开关。`/privacy` 不会改它。
- **Coding data, retention, and training** —— `/privacy` 打开的设置行；编码数据共享，与遥测分开。
- **`[telemetry] trace_upload`** / `GROK_TELEMETRY_TRACE_UPLOAD` —— 会话 traces；未设置时跟随遥测。
- **`[telemetry] otel_*`** / `GROK_EXTERNAL_OTEL` —— 到你自己收集器的外部 OTEL（见下）。

遥测开启时，运行自己收集器的企业可以在 `[telemetry]` 下重定向或关掉部分：

```toml
[telemetry]
events_url = "https://telemetry.your-company.com/events"  # 把事件发到你自己的收集器
events_api_key = "your-collector-token"                   # 收集器认证（如需要）
mixpanel_enabled = false                                  # 关闭 Mixpanel 产品分析
trace_upload = false                                      # 关闭会话/trace 上传（未设置时继承遥测开关）
```

仅在要把遥测指向你自己的基础设施或关掉部分时设置这些。内置端点和凭据由 Grok 管理——保持未设置即使用默认。

同一 `[telemetry]` 表也配置**外部 OpenTelemetry 流**，这是独立的选择加入（不需要上面的遥测开关），把精选、无内容的用量 schema 发到你*自己的* OTLP 收集器。收集器认证来自 `OTEL_EXPORTER_OTLP_HEADERS`，从不存盘。完整 schema、环境变量和隐私模型见 [监控与用量](24-monitoring-usage.md)。

```toml
[telemetry]
otel_enabled = true                                       # 外部 OTEL 总开关（= GROK_EXTERNAL_OTEL）
otel_metrics_exporter = "otlp"                            # otlp | console | none
otel_logs_exporter = "otlp"                               # otlp | console | none
otel_endpoint = "https://collector.corp.example:4318"     # OTLP 基端点
otel_protocol = "http/protobuf"                           # http/protobuf | grpc
otel_certificate = "/etc/ssl/corp-ca.pem"                 # 可选：信任私有 CA（仅路径）
otel_client_certificate = "/etc/ssl/client.crt"           # 可选：mTLS 客户端证书（仅路径）
otel_client_key = "/etc/ssl/client.key"                   # 可选：mTLS 客户端密钥（仅路径）
otel_log_user_prompts = false                             # 内容门闩（管理员通过 requirements 钉住）
otel_log_assistant_responses = false                      # 未设置跟随 prompts；钉住 false 则只记 prompts
otel_log_tool_details = true                              # 元数据/预览；企业默认开，便于 SIEM 关联
otel_log_tool_content = false                             # 全文门闩；与 details 独立 —— 不隐含名称/路径
```

已签名 `requirements.toml` 中列出的 `[telemetry] otel_*` 键会**钉住**并压过
进程环境（目的地锁定）。`managed_config.toml` 不会。没有
`headers` 键——收集器 token 留在 `OTEL_EXPORTER_OTLP_HEADERS`。见
[监控与用量](24-monitoring-usage.md)。

### 版本钉住

控制 CLI 可以自动更新到哪些版本，以及哪些版本可以运行。在
`[cli]` 中设置，或在托管层做全舰队策略。每个都有
环境覆盖，且只能收紧边界，供 CI 和测试使用。

> **已变更：** `minimum_version` 不再阻止启动。它现在是更新器的软
> 防降级下限。要硬下限阻止旧版本启动，使用
> `required_minimum_version`。

```toml
[cli]
minimum_version = "0.2.109"          # 更新器不会降到此版本以下
maximum_version = "0.2.180"          # 更新器不会安装此版本以上
required_minimum_version = "0.2.100" # 低于此版本拒绝启动
required_maximum_version = "0.2.200" # 高于此版本拒绝启动
```

- `minimum_version`（`GROK_MINIMUM_VERSION`）是软防降级下限。更新器
  跳过低于它的目标并保持当前版本。它从不阻止
  启动。
- `maximum_version`（`GROK_MAXIMUM_VERSION`）是软上限。更新器把
  目标封顶在它，从不安装更高版本。
- `required_minimum_version`（`GROK_REQUIRED_MINIMUM_VERSION`）和
  `required_maximum_version`（`GROK_REQUIRED_MAXIMUM_VERSION`）是硬边界。若
  运行版本在范围外，CLI 在启动时退出并指示
  用户安装已批准版本。`grok update` 和 `grok --version` 继续
  工作，以便超范围安装可以恢复。
- 边界跨配置层只收紧：下限取
  最高值，上限取最低，因此托管边界无法放松，
  用户或环境边界也无法取消托管硬边界。无效
  值会被忽略，以免坏策略挡住启动。
- 显式的 `grok update --version X` 允许高于上限，以便从
  过新的安装恢复，低于硬下限则拒绝。

### 企业部署

企业使用的完整配置：

```toml
[cli]
auto_update = false

[auth]
auth_provider_command = "/usr/local/bin/my-company-auth-provider"
auth_provider_label = "Acme Corp"
auth_token_ttl = 3600

[models]
default = "company-grok"

[model.company-grok]
model = "grok-4.6"
base_url = "https://grok-proxy.acme.com/"
name = "Grok 4.6 (Proxy)"
context_window = 128000

[features]
telemetry = false
```

---

## pager.toml（外观配置）

位置：`~/.grok/pager.toml`。这控制 TUI 的观感。变更在重启后生效。

### 终端

```toml
[terminal]
alt_screen = "auto"                   # 全屏模式："auto"、"always"、"never"
```

- `auto`（默认）：终端支持时使用备用屏幕。
- `always`：始终使用备用屏幕。
- `never`：在终端主回看缓冲区中行内运行。

### 动画

```toml
[animation]
fps = 30                              # 动画帧率（每秒滴答）
wave_rows = 32                        # 强调动画每个波浪周期的行数
```

### 提示框

```toml
[prompt]
collapse_unfocused = true             # 回看区聚焦时收起提示框
mouse_hover = true                    # 在提示控件上显示悬停高亮
show_prefix = true                    # 显示提示前缀字符
```

紧凑模式不在这里持久化——运行时用 `[ui] compact_mode` 或 `/compact-mode` 命令控制。

### 回看区

```toml
[scrollback.layout]
outer_vpad = 1                        # 垂直内边距
outer_hpad_left = 2                   # 左侧水平内边距
outer_hpad_right = 2                  # 右侧水平内边距
block_pad_left = 2                    # 块内、内容左侧的内边距
block_pad_right = 2                   # 块内、内容右侧的内边距

[scrollback.scrollbar]
enabled = true                        # 显示滚动条
gap_left = 0                          # 内容与滚动条之间的间隙
gap_right = 0                         # 滚动条与屏幕边缘之间的间隙

[scrollback.scroll]
margin = 0                            # 选中上下最少上下文行
min_page_fraction = 0                 # 最小滚动占视口的百分比（0-100）
follow_indicator = "center"           # ▼/▲ 滚动指示："center" 或 "none"
follow_auto_select = true             # 跟随模式自动选中最新条目
follow_by_overscroll = true           # 滚过底部进入跟随模式
anchor_on_fold = true                 # 折叠时保持块位置
respect_manual_folds = true           # 需开启（默认：false）：流式/结束期间保持手工折叠的块；跟随中展开会停止自动滚动

[scrollback.display]
sticky_headers = true                 # 把用户提示钉为粘性标题
tab_width = 4                         # 每个制表符的空格数
expandable_indicator = true           # 在可折叠条目上显示展开指示
expandable_indicator_running = true   # 在运行中的条目上显示指示
expandable_indicator_char = "›"       # 展开指示字符（默认："›"）
selection_buttons = false             # 选中时显示复制/查看按钮
line_under_last_entry = false         # 最后一条下方的水平线
group_selection_split = true          # 展开块时拆分选中框
highlight_overlays_border = false     # 高亮延伸到选中框边框
dim_accent = 0.5                      # 折叠强调的变暗系数（0.0-1.0）
```

`respect_manual_folds` 默认关闭。打开后，你亲手折叠的块会被钉住：流式更新和结束事件（例如思考块结束）不改其折叠状态；在跟随模式尾随新内容时展开一块会停止自动滚动，让视图停住。跟随通过 `Shift+G`、在最后一条上按 `j`、滚过底部或发送新提示恢复。`Shift+E` 清除所有钉住；`Ctrl+E` 清除思考块上的钉住。

### 块配置

```toml
[scrollback.blocks.edit]
indent = true                         # 缩进 diff 内容
vpad = false                          # 垂直内边距
# expanded_by_default = true          # 未设置：跟随 config.toml 的 [ui] collapsed_edit_blocks
                                      # （旗标开 = 折叠单行）；取消注释以钉住任一种形态
dual_line_numbers = false             # 双列行号（旧 + 新）
# line_summary = false                # 在折叠标题显示 +N/-M；未设置跟随同一旗标
hunk_separator = "…"                  # diff hunk 之间的分隔符（默认："…"）

[scrollback.blocks.prompt]
vpad = true                           # 垂直内边距
show_prefix = true                    # 显示提示前缀字符
min_lines = 2                         # 粘性模式下最少内容行

[scrollback.blocks.thinking]
animate = true                        # 思考时动画强调
truncated_lines = 3                   # 截断模式的行数
```

### 插件

```toml
disable_plugins = false               # 完全隐藏 hooks/plugins UI
```

---

## 环境变量

关键的那些。完整列表见 README。

### 认证

| 变量 | 说明 |
|----------|-------------|
| `XAI_API_KEY` | 来自 console.x.ai 的 API key |
| `GROK_AUTH_PROVIDER_COMMAND` | 外部认证二进制路径 |
| `GROK_AUTH_PROVIDER_LABEL` | TUI 登录屏上的显示名 |
| `GROK_AUTH_TOKEN_TTL` | token 寿命（秒） |
| `GROK_AUTH_EARLY_INVALIDATION_SECS` | 过期前多少秒刷新（默认：300） |
| `GROK_OIDC_ISSUER` | OIDC issuer URL |
| `GROK_OIDC_CLIENT_ID` | OIDC client ID |

### 端点

| 变量 | 说明 |
|----------|-------------|
| `GROK_CLI_CHAT_PROXY_BASE_URL` | 覆盖 API 代理基 URL |

### 功能

| 变量 | 说明 |
|----------|-------------|
| `GROK_MEMORY` | 启用（`1`）或关闭（`0`）跨会话记忆 |
| `GROK_SUBAGENTS` | 启用（`1`）或关闭（`0`）子 agent |
| `GROK_WORKFLOWS` | 启用（`1`）或关闭（`0`）后台工作流并选择 `/goal` 驱动（默认开：宿主拥有的工作流驱动；关：旧的 `update_goal`） |
| `GROK_WEB_FETCH` | 启用（`1`）或关闭（`0`）web_fetch 工具 |
| `GROK_WEB_FETCH_ALLOW_LOCAL` | 仅允许 `web_fetch` 到显式回环主机（`localhost` / `127.0.0.0/8` / `::1`）。与 `[toolset.web_fetch] allow_local` 相同。默认关；私有/元数据仍被挡住。 |
| `GROK_AGENT` | 自定义 agent 定义路径或名称 |
| `GROK_SANDBOX` | 沙箱配置（off、workspace、devbox、read-only、strict；或自定义配置名） |
| `GROK_EXIT_TIMEOUT_SECS` | 请求退出后，若拆卸挂起则强制退出前的秒数（默认：20，`0` 关闭；硬退出再等 5 秒） |

### 日志

| 变量 | 说明 |
|----------|-------------|
| `GROK_LOG_FILE` | 把日志写到此文件路径（按字面用作路径） |
| `RUST_LOG` | 日志级别过滤器（例如 `debug`）；控制 `GROK_LOG_FILE` 日志和无界面 stderr 输出 |

### 路径

| 变量 | 说明 |
|----------|-------------|
| `GROK_HOME` | 覆盖配置目录（默认：`~/.grok`） |
| `GROK_RESPECT_GITIGNORE` | 强制打开（`1`）或关闭（`0`）gitignore 过滤；覆盖 `[tools] respect_gitignore` |

### 遥测

| 变量 | 说明 |
|----------|-------------|
| `GROK_TELEMETRY_ENABLED` | 启用/关闭遥测 |
| `GROK_TELEMETRY_TRACE_UPLOAD` | 启用/关闭会话 trace 上传 |
| `GROK_TELEMETRY_MIXPANEL_ENABLED` | 专门启用/关闭 Mixpanel |
| `GROK_EXTERNAL_OTEL` | 到你收集器的外部 OTEL（见 [24-monitoring-usage.md](24-monitoring-usage.md)） |
| `GROK_FEEDBACK_ENABLED` | 启用/关闭反馈系统 |
| `GROK_DEPLOYMENT_KEY` | 企业用的管理 API key |

---

## 文件位置

| 路径 | 说明 |
|------|-------------|
| `~/.grok/config.toml` | 主配置文件 |
| `~/.grok/pager.toml` | TUI 外观配置 |
| `~/.grok/auth.json` | 认证凭据（自动管理） |
| `~/.grok/sessions/` | 已持久化的会话（按工作目录组织） |
| `~/.grok/memory/` | 跨会话记忆文件和索引 |
| `~/.grok/skills/` | 用户范围技能定义 |
| `~/.grok/plugins/` | 用户范围插件 |
| `~/.grok/agents/` | 用户范围 agent 定义 |
| `~/.grok/lsp.json` | LSP 服务器配置（用户范围） |
| `~/.grok/logs/` | 内部日志文件（例如 `unified.jsonl`、MCP 服务器日志） |
| `.grok/config.toml` | 项目范围 MCP 服务器、插件和权限规则 |
| `.grok/skills/` | 项目范围技能定义 |
| `.grok/plugins/` | 项目范围插件 |
| `.grok/agents/` | 项目范围 agent 定义 |
| `.grok/hooks/` | 项目范围 hooks |
| `.grok/lsp.json` | LSP 服务器配置 |

---

## 项目范围配置

部分设置可以通过在仓库的 `.grok/` 里放文件来按项目设置：

| 文件 | 它配置什么 |
|------|--------------------|
| `.grok/config.toml` | MCP 服务器、插件、权限规则，以及 `[mcp] max_output_bytes` 工具结果上限（其他节只从 `~/.grok/config.toml` 加载） |
| `.grok/skills/` | 项目特定技能 |
| `.grok/hooks/` | 项目特定生命周期 hooks |
| `.grok/agents/` | 项目特定 agent 定义 |
| `.grok/lsp.json` | LSP 服务器配置 |
| `.grok/sandbox.toml` | 自定义沙箱配置 |
| `AGENTS.md` | 项目说明（系统提示） |

项目范围 MCP 服务器会覆盖同名的全局服务器（完全替换，不是合并）。

---

## LSP 服务器

语言服务器驱动被动诊断和可选的 `lsp` 工具（见 [`lsp_tools`](#general-settings) 功能旗标）。定义来自三个来源，按服务器名合并：

| 来源 | 位置 | 范围 |
|--------|----------|-------|
| 用户 | `~/.grok/lsp.json` | 所有项目 |
| 项目 | `.grok/lsp.json` | 当前仓库 |
| 插件 | 受信任插件的 `.lsp.json` 文件，或其 `plugin.json` 中的内联 `lspServers` 块 | 插件启用的地方 |

同一服务器名来自多个来源时，按最高优先级优先解析：

1. **项目** —— `.grok/lsp.json`
2. **用户** —— `~/.grok/lsp.json`
3. **插件** —— 基于文件的 `.lsp.json`，然后是内联 `lspServers`，按插件加载顺序

项目和用户条目替换同名的较低优先级条目。插件条目只添加本地文件尚未定义的服务器名，因此本地 `lsp.json` 始终赢过插件。插件 LSP 服务器仅在插件受信任后加载（见 [插件](09-plugins.md)）。
