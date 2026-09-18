# 配置参考

本文件随 CLI 一并分发，启动时会提取到 `~/.grok/docs/user-guide/26-config-reference.md`。它是 `config.toml`、`managed_config.toml` 和 `requirements.toml` 的完整字段列表。概念说明见 [05-configuration.md](05-configuration.md)。

## 如何配置

三份文件配置 Grok Build，由不同的人编写。

| 文件 | 谁来写 | 存放位置 | 用来 |
| --- | --- | --- | --- |
| `config.toml` | 开发者 | `~/.grok/config.toml`，以及项目里的 `.grok/config.toml` | 设置个人默认值。这里的任何内容都可以由使用这台机器的人改掉。 |
| `managed_config.toml` | 你，通过控制台或部署工具 | `/etc/grok/managed_config.toml` | 给机群提供起点。开发者自己的文件会覆盖它。 |
| `requirements.toml` | 你，带签名 | `/etc/grok/requirements.toml`，或 macOS 设备管理 | 设置开发者无法更改的值。下表标为 `pin` 的键会压过其他所有文件、环境变量和命令行。 |

需要别人还能调整的默认值用 `managed_config.toml`，不能改的用 `requirements.toml`。

Grok Build 还会读这些层，后一行胜出，除非 requirements 的 pin 或 Managed 列另有说明。

1. 编译进程序的默认值。
2. `/etc/grok/managed_config.toml`，然后 `$GROK_HOME/managed_config.toml`（机群默认值；由控制台同步）。
3. `$GROK_HOME/config.toml`（你的设置；`/settings` 写到这里）。默认 `$GROK_HOME` 是 `~/.grok`。
4. 项目 `.grok/config.toml`：只贡献 `[mcp_servers]`、`[plugins]`、`[permission]`，以及 `[mcp] max_output_bytes`。
5. `GROK_CONFIG`（内联 JSON）或 `GROK_CONFIG_PATH`（JSON 或 TOML 文件）。仅允许名单内的键。
6. `$GROK_HOME/requirements.toml`，然后 `/etc/grok/requirements.toml`，然后 macOS MDM `ai.x.grok`。管理员层。表中标为 `pin` 的键无法被覆盖；标为 `yes` 的键也可以写在这份文件里。
7. `GROK_*` 环境变量。
8. CLI 标志，例如 `--model`、`--sandbox`、`--yolo`。

运行 `grok inspect` 或 `grok inspect --json` 可查看哪些文件和值最终生效。

## config.toml

用户级配置在 `$GROK_HOME/config.toml`（默认 `~/.grok/config.toml`；Windows 为 `%USERPROFILE%\.grok\config.toml`）。项目级覆盖在 `.grok/config.toml`，只贡献 `[mcp_servers]`、`[plugins]`、`[permission]`，以及 `[mcp] max_output_bytes`。

**Requirements** 标明同一键能否写在 `requirements.toml`：`pin` 无法被覆盖（解析器尊重 pin 时也包括环境和 CLI）；`yes` 可被该文件接受；`—` 不会从 `requirements.toml` 读取。**Managed** 标明机群 `managed_config.toml` 的值是否站住（`fleet`），还是用户文件胜出（`user`）。

### `agent`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `agent.definition` | `string (path)` | `yes` | `user` | 指向带 YAML frontmatter 的 agent 定义 markdown 文件的路径。 |
| `agent.name` | `string` | `yes` | `user` | 内置或已发现的 agent 定义名称。也可用 GROK_AGENT 和 `--agent-profile`。 |
| `agent.system_prompt_label` | `string` | `yes` | `user` | 全局 system-prompt 身份；按模型的覆盖优先。 |

### `announcements`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `announcements` | `array of tables` | `—` | `user` | 加载时消费的远程公告载荷。不是用户编写的表。 |

### `auth`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `auth` | `table` | `yes` | `user` | `[grok_com_config]` 的别名；每个 `grok_com_config.*` 键也可写成 `auth.*`。 |
| `auth.auth_provider_command` | `string` | `yes` | `user` | 外部认证二进制；stdout 为 token。也可用 GROK_AUTH_PROVIDER_COMMAND；也可写成 `grok_com_config.auth_provider_command`。 |
| `auth.auth_provider_label` | `string` | `yes` | `user` | 外部认证提供方的登录按钮文案。也可用 GROK_AUTH_PROVIDER_LABEL；也可写成 `grok_com_config.auth_provider_label`。 |
| `auth.auth_token_ttl` | `number` | `yes` | `user` | 返回裸 token 的提供方的 token TTL（秒）。也可用 GROK_AUTH_TOKEN_TTL；也可写成 `grok_com_config.auth_token_ttl`。 |
| `auth.disable_api_key_auth` | `boolean` | `pin` | `user` | 拒绝 API-key 认证，从而只有部署的 IdP 能登录。也可用 GROK_DISABLE_API_KEY_AUTH；也可写成 `grok_com_config.disable_api_key_auth`。 |
| `auth.force_login_team_uuid` | `string / string[]` | `pin` | `user` | 要求登录到此 team UUID，或数组中的任意一个；空数组失败即关闭。也可用 GROK_FORCE_LOGIN_TEAM_ID；也可写成 `grok_com_config.force_login_team_uuid`。 |
| `auth.grok_ws_origin` | `string` | `yes` | `user` | grok.com 的 websocket origin。也可用 GROK_WS_ORIGIN；也可写成 `grok_com_config.grok_ws_origin`。 |
| `auth.grok_ws_url` | `string` | `yes` | `user` | 中继 websocket URL。也可用 GROK_WS_URL；也可写成 `grok_com_config.grok_ws_url`。 |
| `auth.oauth2` | `table` | `yes` | `user` | 未设置企业 OIDC 时使用的 OAuth2 提供方；也可写成 `grok_com_config.oauth2`。 |
| `auth.oauth2.client_id` | `string` | `yes` | `user` | OAuth2 client id。也可用 GROK_OAUTH2_CLIENT_ID；也可写成 `grok_com_config.oauth2.client_id`。 |
| `auth.oauth2.issuer` | `string` | `yes` | `user` | OAuth2 issuer URL。也可用 GROK_OAUTH2_ISSUER；也可写成 `grok_com_config.oauth2.issuer`。 |
| `auth.oauth2.principal_id` | `string` | `yes` | `user` | 设置了 `principal_type` 时必需的 principal id。也可用 GROK_OAUTH2_PRINCIPAL_ID；也可写成 `grok_com_config.oauth2.principal_id`。 |
| `auth.oauth2.principal_type` | `string` | `yes` | `user` | Token 的 principal 类型，例如 Team。也可用 GROK_OAUTH2_PRINCIPAL_TYPE；也可写成 `grok_com_config.oauth2.principal_type`。 |
| `auth.oauth2.referrer` | `string` | `yes` | `user` | OAuth 用量归因的 referrer。也可用 GROK_OAUTH2_REFERRER；也可写成 `grok_com_config.oauth2.referrer`。 |
| `auth.oauth2.scopes` | `string[]` | `yes` | `user` | OAuth2 scopes。也可用 GROK_OAUTH2_SCOPES；也可写成 `grok_com_config.oauth2.scopes`。 |
| `auth.oidc` | `table` | `yes` | `user` | 客户 OIDC 身份提供方设置；也可写成 `grok_com_config.oidc`。 |
| `auth.oidc.audience` | `string` | `yes` | `user` | 可选的 OIDC audience。也可用 GROK_OIDC_AUDIENCE；也可写成 `grok_com_config.oidc.audience`。 |
| `auth.oidc.client_id` | `string` | `yes` | `user` | OIDC client id。也可用 GROK_OIDC_CLIENT_ID；也可写成 `grok_com_config.oidc.client_id`。 |
| `auth.oidc.issuer` | `string` | `yes` | `user` | OIDC issuer URL。也可用 GROK_OIDC_ISSUER；也可写成 `grok_com_config.oidc.issuer`。 |
| `auth.oidc.scopes` | `string[]` | `yes` | `user` | OIDC scopes。也可用 GROK_OIDC_SCOPES；也可写成 `grok_com_config.oidc.scopes`。 |
| `auth.preferred_method` | `api_key / oidc` | `yes` | `user` | 把自动认证钉死到一种方法，不回退；也可写成 `grok_com_config.preferred_method`。 |
| `auth.token_header` | `string` | `yes` | `user` | 携带 CLI 认证 token 的请求头名；默认 `xai-grok-cli`；也可写成 `grok_com_config.token_header`。 |

### `auth_provider`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `auth_provider.<name>` | `table` | `yes` | `user` | 供 `[model.<id>] auth_provider` 使用的具名凭据助手。 |

### `auto_mode`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `auto_mode.enabled` | `boolean` | `yes` | `user` | 启用 Auto 权限模式。 |

### `campaigns`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `campaigns` | `array of tables` | `yes` | `user` | 在 requirements 之下应用的具名 campaign 补丁。由部署发布。 |

### `cli`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `cli.auto_update` | `boolean` | `pin` | `user` | 启动时检查 CLI 更新。也可用 GROK_DISABLE_AUTOUPDATER 关闭。 |
| `cli.channel` | `stable / alpha` | `pin` | `user` | 发布通道偏好。 |
| `cli.grove_worktree` | `boolean` 或 `grove` / `grove-fuse` / `grove-nfs` / `nfs` / `copy` / `true` / `false` / `1` / `0` / `on` / `off` | `yes` | `user` | 会话 / `-w` 的 Grove 与 copy。默认 copy。与创建模式的 `cli.worktree_type` 不同。也可用 `GROK_WORKTREE_TYPE`。层顺序：request → env → local → remote-true；然后最后一刀：远程 `grove_worktree = false` → copy（`remote_kill`）；缺少远程设置 → copy（`remote_unavailable`）。不会启用 `grok clone`。 |
| `cli.installer` | `string` | `—` | `user` | 上次安装此 CLI 的安装器，用来选择更新路径。 |
| `cli.maximum_version` | `string` | `pin` | `user` | 仍能运行、不会被硬拦截的最高 CLI 版本。也可用 GROK_MAXIMUM_VERSION。 |
| `cli.minimum_version` | `string` | `pin` | `user` | 仍能运行、不会被硬拦截的最低 CLI 版本。也可用 GROK_MINIMUM_VERSION。 |
| `cli.npm_registry` | `string` | `yes` | `user` | 自动更新器用的 npm registry。 |
| `cli.nfs_worktree` | 与 `cli.grove_worktree` 相同 | `yes` | `user` | `cli.grove_worktree` 的读取别名。 |
| `cli.required_maximum_version` | `string` | `pin` | `user` | CLI 硬上限版本。也可用 GROK_REQUIRED_MAXIMUM_VERSION。 |
| `cli.required_minimum_version` | `string` | `pin` | `user` | CLI 硬下限版本。也可用 GROK_REQUIRED_MINIMUM_VERSION。 |
| `cli.session_picker_grouped` | `boolean` | `yes` | `user` | 在选择器和 CLI 列表中按仓库分组会话。 |
| `cli.session_registry` | `boolean` | `yes` | `user` | 加入跨进程会话注册表。 |
| `cli.show_tips` | `boolean` | `pin` | `user` | 启动提示。 |
| `cli.use_leader` | `boolean` | `pin` | `user` | 用 leader 进程做配置重载和 MCP 监视。 |
| `cli.worktree_type` | `string` | `yes` | `user` | 设为 `linked`、`standalone` 或 `git` 时是创建模式。`grove`、`grove-fuse`、`grove-nfs`、`nfs` 和 `copy` 这些写法也会喂给会话 / `-w` 的 Grove 门闩（与 `cli.grove_worktree` 相同）；它们不是创建模式的值。 |

### `compat`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `compat.claude.agents` | `boolean` | `yes` | `user` | 扫描 CLAUDE.md。也可用 GROK_CLAUDE_AGENTS_ENABLED。 |
| `compat.claude.hooks` | `boolean` | `yes` | `user` | 扫描 Claude hooks。也可用 GROK_CLAUDE_HOOKS_ENABLED。 |
| `compat.claude.mcps` | `boolean` | `yes` | `user` | 扫描 Claude MCP 配置。也可用 GROK_CLAUDE_MCPS_ENABLED。 |
| `compat.claude.rules` | `boolean` | `yes` | `user` | 扫描 Claude rules。也可用 GROK_CLAUDE_RULES_ENABLED。 |
| `compat.claude.skills` | `boolean` | `yes` | `user` | 扫描 Claude skills。也可用 GROK_CLAUDE_SKILLS_ENABLED。 |
| `compat.codex.hooks` | `boolean` | `yes` | `user` | 存在时扫描 Codex hooks。 |
| `compat.codex.skills` | `boolean` | `yes` | `user` | 存在时扫描 Codex skills 目录。 |
| `compat.cursor.agents` | `boolean` | `yes` | `user` | 从 Cursor 兼容来源扫描 agent 定义。也可用 GROK_CURSOR_AGENTS_ENABLED。 |
| `compat.cursor.hooks` | `boolean` | `yes` | `user` | 扫描 Cursor hooks。也可用 GROK_CURSOR_HOOKS_ENABLED。 |
| `compat.cursor.mcps` | `boolean` | `yes` | `user` | 扫描 Cursor mcp.json。也可用 GROK_CURSOR_MCPS_ENABLED。 |
| `compat.cursor.rules` | `boolean` | `yes` | `user` | 扫描 `.cursor/rules/`。也可用 GROK_CURSOR_RULES_ENABLED。 |
| `compat.cursor.skills` | `boolean` | `yes` | `user` | 扫描 Cursor skills 目录。也可用 GROK_CURSOR_SKILLS_ENABLED。 |

### `dashboard`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `dashboard.enabled` | `boolean` | `yes` | `user` | 显示 agent 仪表盘。 |
| `dashboard.grouping` | `state / directory` | `yes` | `user` | 仪表盘行如何分组。 |

### `default_auto_mode`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `default_auto_mode` | `boolean` | `yes` | `user` | 没有按会话覆盖时，以 auto 权限模式启动会话。 |

### `diagnostics`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `diagnostics.crash_handler` | `boolean` | `yes` | `user` | 把 panic 报告写到 `$GROK_HOME/crash/`。也可用 GROK_CRASH_HANDLER。 |

### `disable_web_search`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `disable_web_search` | `boolean` | `yes` | `user` | 对本进程丢掉 web_search 工具。也可用 `--disable-web-search`。 |

### `disabled_mcp_servers`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `disabled_mcp_servers` | `string[]` | `yes` | `user` | 要跳过的 MCP 服务器名称，而不删除它们的 `[mcp_servers]` 块。 |

### `disabled_mcp_tools`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `disabled_mcp_tools` | `map<string, string[]>` | `yes` | `user` | 按服务器名称键控的 MCP 工具拒绝列表。 |

### `doom_loop_recovery`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `doom_loop_recovery.enabled` | `boolean` | `yes` | `user` | 对自信的工具调用循环重新采样；设为 false 可关闭。 |

### `endpoints`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `endpoints.cli_chat_proxy_base_url` | `string` | `pin` | `user` | 会话服务 API 的 base URL。 |
| `endpoints.deployment_key` | `string` | `pin` | `user` | 企业部署的管理密钥。也可用 GROK_DEPLOYMENT_KEY。 |
| `endpoints.feedback_base_url` | `string` | `yes` | `user` | 反馈提交的去向。也可用 GROK_FEEDBACK_BASE_URL。 |
| `endpoints.managed_config_url` | `string` | `yes` | `user` | 覆盖托管配置端点。也可用 GROK_MANAGED_CONFIG_URL。 |
| `endpoints.models_base_url` | `string` | `pin` | `user` | 自定义推理 base URL。也可用 GROK_MODELS_BASE_URL。 |
| `endpoints.models_list_url` | `string` | `pin` | `user` | 覆盖模型列表 URL。也可用 GROK_MODELS_LIST_URL。别名 `models_endpoint`。 |
| `endpoints.trace_upload_bucket` | `string` | `yes` | `user` | traces 的直接 gs:// 或 s3:// bucket；绕过代理。也可用 GROK_TRACE_UPLOAD_BUCKET。 |
| `endpoints.trace_upload_credentials` | `string` | `yes` | `user` | 该 bucket 的内联 GCS service-account JSON 或 AWS 凭据；优先于 `trace_upload_credentials_file`，且没有环境变量。 |
| `endpoints.trace_upload_credentials_file` | `string (path)` | `yes` | `user` | 该 bucket 的 GCS service-account JSON 或 AWS 凭据文件路径。也可用 GROK_TRACE_UPLOAD_CREDENTIALS_FILE。 |
| `endpoints.trace_upload_endpoint_url` | `string` | `yes` | `user` | s3:// bucket 上传的自定义 S3 兼容端点。也可用 GROK_TRACE_UPLOAD_ENDPOINT_URL。 |
| `endpoints.trace_upload_region` | `string` | `yes` | `user` | s3:// bucket 上传的 AWS region；默认 us-east-1。也可用 GROK_TRACE_UPLOAD_REGION。 |
| `endpoints.trace_upload_url` | `string` | `pin` | `user` | 未设置直接 bucket 时 traces 的代理目的地。也可用 GROK_TRACE_UPLOAD_URL。 |
| `endpoints.xai_api_base_url` | `string` | `pin` | `user` | 公开 xAI API 的 base。也可用 GROK_XAI_API_BASE_URL。 |

### `features`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `features.active_agent_messages` | `boolean` | `pin` | `user` | 启用或禁用 `active_agent_messages`。默认 false。也可用 `GROK_ACTIVE_AGENT_MESSAGES`。 |
| `features.ask_user_question` | `boolean` | `pin` | `user` | 启用或禁用 `ask_user_question`。默认 true。也可用 `GROK_ASK_USER_QUESTION`。 |
| `features.auto_wake` | `boolean` | `pin` | `user` | 启用或禁用 `auto_wake`。默认 true。也可用 `GROK_AUTO_WAKE`。 |
| `features.backend_tools` | `boolean` | `pin` | `user` | 启用或禁用 `backend_tools`。默认 true。也可用 `GROK_BACKEND_SEARCH`。 |
| `features.campaigns` | `boolean` | `yes` | `user` | 启用远程 campaign 补丁。即使 requirements 把它设为 true，`GROK_CAMPAIGNS=0` 仍会关闭。 |
| `features.cancel_rewind` | `boolean` | `pin` | `user` | 启用或禁用 `cancel_rewind`。默认 true。也可用 `GROK_CANCEL_REWIND`。 |
| `features.codebase_indexing` | `boolean / string[]` | `pin` | `user` | 代码库图索引；true 会索引 git 仓库，也可传入 include/exclude glob。 |
| `features.compaction_detail` | `none / minimal / balanced / verbose` | `yes` | `user` | `segments` 压缩的逐字详细程度。也可用 GROK_COMPACTION_DETAIL。 |
| `features.compaction_mode` | `summary / transcript / segments` | `yes` | `user` | 压缩策略。也可用 GROK_COMPACTION_MODE。 |
| `features.compaction_tool_choice` | `string` | `yes` | `user` | 压缩时使用的 tool-choice 提示。 |
| `features.compaction_verbatim_input` | `boolean` | `pin` | `user` | 启用或禁用 `compaction_verbatim_input`。默认 true。也可用 `GROK_COMPACTION_VERBATIM_INPUT`。 |
| `features.dock` | `boolean` | `pin` | `user` | 启用或禁用 `dock`。默认 false。也可用 `GROK_DOCK`。 |
| `features.feedback` | `boolean` | `pin` | `user` | 启用或禁用 `feedback`。默认 true。也可用 `GROK_FEEDBACK_ENABLED`。 |
| `features.feedback_trace_card` | `boolean` | `pin` | `user` | `/feedback` 后显示 trace 上传同意问题。默认 false。也可用 `GROK_FEEDBACK_TRACE_CARD`。 |
| `features.image_edit_model_override` | `string` | `yes` | `user` | image_edit 用的 Imagine 模型 id。 |
| `features.image_gen` | `boolean` | `pin` | `user` | 启用 image_gen / `/imagine`。 |
| `features.image_gen_model_override` | `string` | `yes` | `user` | image_gen 用的 Imagine 模型 id。留空则回退到远程配置的默认。 |
| `features.lsp_tools` | `boolean` | `pin` | `user` | 启用或禁用 `lsp_tools`。默认 false。也可用 `GROK_LSP_TOOLS`。 |
| `features.managed_config` | `boolean` | `yes` | `user` | 从部署拉取 managed_config.toml 和 requirements.toml。 |
| `features.mcp_auto_restart` | `boolean` | `yes` | `user` | 传输失败后自动重启 stdio MCP 服务器。也可用 GROK_MCP_AUTO_RESTART。 |
| `features.mcp_liveness_watchers` | `boolean` | `yes` | `user` | 轮询 MCP 传输并推送 server_status 更新。为 false 时是紧急开关。 |
| `features.mcp_push_server_status` | `boolean` | `yes` | `user` | pager 订阅 MCP server_status 推送。启动时进程环境变量 GROK_MCP_PUSH_SERVER_STATUS 优先。 |
| `features.mcp_recursive_config_watch` | `boolean` | `yes` | `user` | 监视 `<cwd>/` 和 `<cwd>/.grok/` 上的项目 MCP 配置改动。名称有误导；监视不是递归的。 |
| `features.non_git_warning` | `boolean` | `yes` | `user` | 在 Git 仓库外启动 Grok 时显示阻塞警告。 |
| `features.remember_mode` | `boolean` | `—` | `—` | 跨会话记住上次的权限模式。只从用户 `config.toml` 读取。 |
| `features.remote_fetch` | `boolean` | `pin` | `fleet` | 钉住远程模型目录和资源拉取。两边都设置时 Managed 优先于用户文件。 |
| `features.repo_status_in_system_prompt` | `boolean` | `pin` | `user` | 启用或禁用 `repo_status_in_system_prompt`。默认 true。也可用 `GROK_REPO_STATUS_IN_SYSTEM_PROMPT`。 |
| `features.session_recap` | `boolean` | `pin` | `user` | 启用或禁用 `session_recap`。默认 true。也可用 `GROK_SESSION_RECAP`。 |
| `features.session_search` | `boolean` | `pin` | `user` | 启用或禁用 `session_search`。默认 true。也可用 `GROK_SESSION_SEARCH`。 |
| `features.subagent_worktree_snapshot` | `boolean` | `pin` | `user` | 启用或禁用 `subagent_worktree_snapshot`。默认 false。也可用 `GROK_SUBAGENT_WORKTREE_SNAPSHOT`。 |
| `features.support_permission` | `boolean` | `yes` | `user` | 允许 agent 为工具执行请求权限。 |
| `features.telemetry` | `boolean / session_metrics / off` | `pin` | `user` | 产品遥测模式。企业默认为 off。 |
| `features.terminal_theme` | `boolean` | `pin` | `user` | 在推出期间露出终端原生的 `terminal` 颜色主题。默认 false。也可用 `GROK_TERMINAL_THEME`。 |
| `features.title_refresh` | `boolean` | `pin` | `user` | 会话早期自动刷新标题。在 requirements 里 pin 此项可压过 GROK_TITLE_REFRESH。 |
| `features.turn_summary` | `boolean` | `pin` | `user` | 启用或禁用 `turn_summary`。默认 true。也可用 `GROK_TURN_SUMMARY`。 |
| `features.two_pass_compaction` | `boolean` | `pin` | `user` | 启用或禁用 `two_pass_compaction`。默认 true。也可用 `GROK_TWO_PASS_COMPACTION`。 |
| `features.video_gen` | `boolean` | `pin` | `user` | 启用视频工具 / `/imagine-video`。 |
| `features.voice_mode` | `boolean` | `pin` | `user` | 启用或禁用 `voice_mode`。默认 true。也可用 `GROK_VOICE_MODE`。 |
| `features.web_fetch` | `boolean` | `pin` | `user` | 启用或禁用 `web_fetch`。默认 false。也可用 `GROK_WEB_FETCH`。 |
| `features.write_file` | `boolean` | `pin` | `user` | 启用或禁用 `write_file`。默认 true。也可用 `GROK_WRITE_FILE`。 |
| `features.zdr_access_enabled` | `boolean` | `pin` | `user` | 团队处于 Zero Data Retention 时仍宣传与 ZDR 不兼容的工具。也可用 `GROK_ZDR_ACCESS_ENABLED`。 |

### `feedback`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `feedback.user.command` | `string` | `yes` | `user` | 打印姓名和邮箱 JSON 供反馈提交的 shell 命令。 |
| `feedback.user.email` | `string[]` | `yes` | `user` | 反馈作者邮箱的来源（`git_email` 或字面量）。 |
| `feedback.user.name` | `string[]` | `yes` | `user` | 反馈作者姓名的来源（`os_user` 或字面量）。 |

### `goal`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `goal.enabled` | `boolean` | `yes` | `user` | 启用 `/goal`。 |

### `grok_com_config`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `grok_com_config` | `table` | `yes` | `user` | Grok.com 的 websocket 与 OAuth/OIDC 设置。`[auth]` 是别名。 |
| `grok_com_config.auth_provider_command` | `string` | `yes` | `user` | 外部认证二进制；stdout 为 token。也可用 GROK_AUTH_PROVIDER_COMMAND。 |
| `grok_com_config.auth_provider_label` | `string` | `yes` | `user` | 外部认证提供方的登录按钮文案。也可用 GROK_AUTH_PROVIDER_LABEL。 |
| `grok_com_config.auth_token_ttl` | `number` | `yes` | `user` | 返回裸 token 的提供方的 token TTL（秒）。也可用 GROK_AUTH_TOKEN_TTL。 |
| `grok_com_config.disable_api_key_auth` | `boolean` | `pin` | `user` | 拒绝 API-key 认证，从而只有部署的 IdP 能登录。也可用 GROK_DISABLE_API_KEY_AUTH。 |
| `grok_com_config.force_login_team_uuid` | `string / string[]` | `pin` | `user` | 要求登录到此 team UUID，或数组中的任意一个；空数组失败即关闭。也可用 GROK_FORCE_LOGIN_TEAM_ID。 |
| `grok_com_config.grok_ws_origin` | `string` | `yes` | `user` | grok.com 的 websocket origin。也可用 GROK_WS_ORIGIN。 |
| `grok_com_config.grok_ws_url` | `string` | `yes` | `user` | 中继 websocket URL。也可用 GROK_WS_URL。 |
| `grok_com_config.oauth2` | `table` | `yes` | `user` | 未设置企业 OIDC 时使用的 OAuth2 提供方。 |
| `grok_com_config.oauth2.client_id` | `string` | `yes` | `user` | OAuth2 client id。也可用 GROK_OAUTH2_CLIENT_ID。 |
| `grok_com_config.oauth2.issuer` | `string` | `yes` | `user` | OAuth2 issuer URL。也可用 GROK_OAUTH2_ISSUER。 |
| `grok_com_config.oauth2.principal_id` | `string` | `yes` | `user` | 设置了 `principal_type` 时必需的 principal id。也可用 GROK_OAUTH2_PRINCIPAL_ID。 |
| `grok_com_config.oauth2.principal_type` | `string` | `yes` | `user` | Token 的 principal 类型，例如 Team。也可用 GROK_OAUTH2_PRINCIPAL_TYPE。 |
| `grok_com_config.oauth2.referrer` | `string` | `yes` | `user` | OAuth 用量归因的 referrer。也可用 GROK_OAUTH2_REFERRER。 |
| `grok_com_config.oauth2.scopes` | `string[]` | `yes` | `user` | OAuth2 scopes。也可用 GROK_OAUTH2_SCOPES。 |
| `grok_com_config.oidc` | `table` | `yes` | `user` | 客户 OIDC 身份提供方设置。 |
| `grok_com_config.oidc.audience` | `string` | `yes` | `user` | 可选的 OIDC audience。也可用 GROK_OIDC_AUDIENCE。 |
| `grok_com_config.oidc.client_id` | `string` | `yes` | `user` | OIDC client id。也可用 GROK_OIDC_CLIENT_ID。 |
| `grok_com_config.oidc.issuer` | `string` | `yes` | `user` | OIDC issuer URL。也可用 GROK_OIDC_ISSUER。 |
| `grok_com_config.oidc.scopes` | `string[]` | `yes` | `user` | OIDC scopes。也可用 GROK_OIDC_SCOPES。 |
| `grok_com_config.preferred_method` | `api_key / oidc` | `yes` | `user` | 把自动认证钉死到一种方法，不回退。 |
| `grok_com_config.token_header` | `string` | `yes` | `user` | 携带 CLI 认证 token 的请求头名；默认 `xai-grok-cli`。 |

### `harness`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `harness.wait_for_uploads` | `boolean` | `yes` | `user` | 在返回提示响应之前等待回合结束的 trace 上传。默认关闭；一次性无头运行改为在退出时排空待处理的回合结束上传，并受强制最低预算约束（约 150s：解析窗口加上一次上传尝试）；`upload_flush_timeout_secs` 更大时会延长该预算。 |
| `harness.disable_workspace_teleport` | `boolean` | `pin` | `user` | 每回合工作区快照的紧急开关。 |

### `hints`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `hints.fork_worktree_mode` | `ask / always / never` | `yes` | `user` | `/fork` 是否提供工作树。 |
| `hints.new_session_worktree_mode` | `ask / always / never` | `yes` | `user` | `/new` 是否提供工作树。 |

### `hooks`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `hooks.<event>` | `array of tables` | `yes` | `user` | 生命周期事件（如 PreToolUse 或 Stop）的 matcher 组。见 Hooks。 |
| `hooks.<event>[].hooks[].command` | `string` | `yes` | `user` | 此 hook 要运行的命令。加载时不展开 `$VAR`。 |
| `hooks.<event>[].hooks[].type` | `command` | `yes` | `user` | Hook 处理类型。支持 command hooks。 |
| `hooks.<event>[].matcher` | `string` | `yes` | `user` | 此 hook 组的工具名 matcher。 |

### `managed_mcps`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `managed_mcps.enabled` | `boolean` | `pin` | `user` | 启动时拉取托管 MCP 配置。也可用 GROK_MANAGED_MCPS_ENABLED。 |
| `managed_mcps.gateway_tools_enabled` | `boolean` | `yes` | `user` | 暴露托管 MCP 网关工具。也可用 GROK_MANAGED_MCP_GATEWAY_TOOLS_ENABLED。 |

### `marketplace`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `marketplace.sources` | `array of tables` | `yes` | `user` | `[[marketplace.sources]]` 插件 marketplace 仓库。 |
| `marketplace.require_sha` | `boolean` | `yes` | `user` | 只收紧：远程插件安装和更新必须钉住完整 commit sha。也可用 `GROK_MARKETPLACE_REQUIRE_SHA`。此键和环境变量都不能把这道门再关掉。 |

### `mcp`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `mcp.max_output_bytes` | `number` | `yes` | `user` | 限制 MCP 工具输出大小（字节）。项目文件可以设置此项。 |

### `mcp_servers`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `mcp_servers.<name>.args` | `string[]` | `yes` | `user` | stdio 或 HTTP MCP 服务器上 `[mcp_servers.<name>]` 的 `args`。 |
| `mcp_servers.<name>.bearer_token_env_var` | `string` | `yes` | `user` | stdio 或 HTTP MCP 服务器上 `[mcp_servers.<name>]` 的 `bearer_token_env_var`。 |
| `mcp_servers.<name>.command` | `string` | `yes` | `user` | stdio 或 HTTP MCP 服务器上 `[mcp_servers.<name>]` 的 `command`。 |
| `mcp_servers.<name>.cwd` | `string` | `yes` | `user` | stdio 或 HTTP MCP 服务器上 `[mcp_servers.<name>]` 的 `cwd`。 |
| `mcp_servers.<name>.enabled` | `boolean` | `yes` | `user` | stdio 或 HTTP MCP 服务器上 `[mcp_servers.<name>]` 的 `enabled`。 |
| `mcp_servers.<name>.env` | `table` | `yes` | `user` | stdio 或 HTTP MCP 服务器上 `[mcp_servers.<name>]` 的 `env`。 |
| `mcp_servers.<name>.expose_image_base64` | `boolean` | `yes` | `user` | stdio 或 HTTP MCP 服务器上 `[mcp_servers.<name>]` 的 `expose_image_base64`。 |
| `mcp_servers.<name>.headers` | `table` | `yes` | `user` | stdio 或 HTTP MCP 服务器上 `[mcp_servers.<name>]` 的 `headers`。 |
| `mcp_servers.<name>.oauth` | `table` | `yes` | `user` | stdio 或 HTTP MCP 服务器上 `[mcp_servers.<name>]` 的 `oauth`。 |
| `mcp_servers.<name>.oauth_client_id` | `string` | `yes` | `user` | stdio 或 HTTP MCP 服务器上 `[mcp_servers.<name>]` 的 `oauth_client_id`。 |
| `mcp_servers.<name>.oauth_client_secret_env_var` | `string` | `yes` | `user` | stdio 或 HTTP MCP 服务器上 `[mcp_servers.<name>]` 的 `oauth_client_secret_env_var`。 |
| `mcp_servers.<name>.oauth_scopes` | `string[]` | `yes` | `user` | stdio 或 HTTP MCP 服务器上 `[mcp_servers.<name>]` 的 `oauth_scopes`。 |
| `mcp_servers.<name>.setup` | `table` | `yes` | `user` | stdio 或 HTTP MCP 服务器上 `[mcp_servers.<name>]` 的 `setup`。 |
| `mcp_servers.<name>.startup_timeout_sec` | `number` | `yes` | `user` | stdio 或 HTTP MCP 服务器上 `[mcp_servers.<name>]` 的 `startup_timeout_sec`。 |
| `mcp_servers.<name>.tool_timeout_sec` | `number` | `yes` | `user` | stdio 或 HTTP MCP 服务器上 `[mcp_servers.<name>]` 的 `tool_timeout_sec`。 |
| `mcp_servers.<name>.tool_timeouts` | `table` | `yes` | `user` | stdio 或 HTTP MCP 服务器上 `[mcp_servers.<name>]` 的 `tool_timeouts`。 |
| `mcp_servers.<name>.type` | `string` | `yes` | `user` | stdio 或 HTTP MCP 服务器上 `[mcp_servers.<name>]` 的 `type`。 |
| `mcp_servers.<name>.url` | `string` | `yes` | `user` | stdio 或 HTTP MCP 服务器上 `[mcp_servers.<name>]` 的 `url`。 |

### `memory`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `memory.enabled` | `boolean` | `pin` | `user` | 跨会话记忆总开关。也可用 GROK_MEMORY。 |
| `memory.mode` | `"legacy"`、`"v2"` | — | `user` | 为新会话选择持久记忆实现。默认：`"legacy"`。`"v2"` 是实验性的；其 legacy 的 search、flush 和 Dream 路径已关闭。 |

### `model`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `model.<id>` | `table` | `yes` | `user` | 按模型覆盖或 BYOK 定义。优先用 `env_key` 而不是内联 `api_key`。 |
| `model.<id>.agent_type` | `string` | `yes` | `user` | 与此模型关联的 agent 定义类型。 |
| `model.<id>.api_backend` | `chat_completions / responses / messages` | `yes` | `user` | 此模型的线协议。 |
| `model.<id>.api_base_url` | `string` | `yes` | `user` | 与 XAI_API_KEY 解析一起使用的备用 API base。 |
| `model.<id>.api_key` | `string` | `yes` | `user` | 内联 API key。优先用 `env_key`。不要把密钥放进共享仓库。 |
| `model.<id>.auth_provider` | `string` | `yes` | `user` | 为此模型签发 bearer token 的 `[auth_provider.<name>]` 助手名称。 |
| `model.<id>.auto_compact_threshold_percent` | `integer` | `yes` | `user` | 按模型的自动压缩阈值（0-100）。 |
| `model.<id>.base_url` | `string` | `yes` | `user` | 提供方端点的 base URL。 |
| `model.<id>.compaction_at_tokens` | `number / table` | `yes` | `user` | 触发此模型压缩的 token 阈值。 |
| `model.<id>.compactions_remaining` | `string / table` | `yes` | `user` | 压缩后剩余上下文如何发送。别名 `send_compactions_remaining`。 |
| `model.<id>.context_window` | `number` | `yes` | `user` | 上下文窗口 token 数；驱动自动压缩时机。 |
| `model.<id>.description` | `string` | `yes` | `user` | 选择器中显示的可选描述。 |
| `model.<id>.env_http_headers` | `map<string,string>` | `yes` | `user` | 已设置时从环境变量填充的 HTTP 头。 |
| `model.<id>.env_key` | `string / string[]` | `yes` | `user` | 存放提供方 API key 的环境变量名。 |
| `model.<id>.extra_body` | `table` | `yes` | `user` | 合并进此模型推理请求 JSON body 的额外字段。允许嵌套表/数组。保留键（`model`、`messages`、`input`、`tools`、`stream` 等）会被跳过。 |
| `model.<id>.extra_headers` | `map<string,string>` | `yes` | `user` | 此模型的按请求头。 |
| `model.<id>.hidden` | `boolean` | `yes` | `user` | 在选择器中隐藏此模型。仍可通过 `-m` 使用。 |
| `model.<id>.inference_idle_timeout_secs` | `number` | `yes` | `user` | 此模型流式推理的空闲超时。 |
| `model.<id>.keychain_account` | `string` | `yes` | `user` | 系统钥匙串 account/用户名。单独设置即可；未写 `keychain_service` 时查找 service `grok`。项缺失时等同于空的 `env_key`。 |
| `model.<id>.keychain_service` | `string` | `yes` | `user` | 可选的系统钥匙串 service 覆盖。默认 `grok`，不建议改。在 `api_key`/`env_key` 之后、会话 token 之前解析。密钥不会被记录。 |
| `model.<id>.max_completion_tokens` | `number` | `yes` | `user` | 按模型的最大 completion token 数。 |
| `model.<id>.max_retries` | `number` | `yes` | `user` | 此模型的推理重试次数。 |
| `model.<id>.model` | `string` | `yes` | `user` | 发给 API 的模型 id。 |
| `model.<id>.model_family` | `string` | `yes` | `user` | 用于压缩和能力分组的 family id。 |
| `model.<id>.model_provider` | `string` | `yes` | `user` | 此模型的具名 `[model_providers.<name>]` 提供方 id。 |
| `model.<id>.mtls_cert_dir` | `string` | `yes` | `user` | 包含模型端点 mTLS 身份的目录，文件为 `client.crt` 和 `client.key`，或 `tls.crt` 和 `tls.key`；除非同一模型只有一个 HTTPS `base_url`、没有 `api_base_url`，且请求不跟随重定向，否则配置会被拒绝。 |
| `model.<id>.name` | `string` | `yes` | `user` | 模型选择器中显示的标签。 |
| `model.<id>.query_params` | `map<string,string>` | `yes` | `user` | 此模型请求上的额外查询参数。 |
| `model.<id>.rate_limit_retry_threshold` | `number` | `yes` | `user` | 限速请求的总尝试上限，受已解析的 `max_retries` 限制；配置后会关闭单独的子 agent 429 等待循环。 |
| `model.<id>.reasoning_effort` | `string` | `yes` | `user` | 已弃用的按模型 effort；请用 `reasoning_efforts`。 |
| `model.<id>.reasoning_efforts` | `array of tables` | `yes` | `user` | 此模型允许的 reasoning-effort 取值。 |
| `model.<id>.show_model_fingerprint` | `boolean` | `yes` | `user` | 存在时在 UI 中显示提供方模型指纹。 |
| `model.<id>.stream_tool_calls` | `boolean` | `yes` | `user` | 按模型的工具调用流式请求形态。 |
| `model.<id>.subagent_rate_limit_max_attempts` | `number` | `yes` | `user` | 未设置 `rate_limit_retry_threshold` 时，子 agent 429 等待循环的最大尝试次数；默认 8，最大 32，`0` 关闭等待循环。 |
| `model.<id>.supported_in_api` | `boolean` | `yes` | `user` | 此目录条目是否作为公开 API 模型提供。 |
| `model.<id>.supports_backend_search` | `boolean` | `yes` | `user` | 端点是否支持 Grok 托管的服务端搜索工具。 |
| `model.<id>.supports_reasoning_effort` | `boolean` | `yes` | `user` | 已弃用；请用 `reasoning_efforts`。 |
| `model.<id>.system_prompt_label` | `string` | `yes` | `user` | 按模型的 system-prompt 身份标签。 |
| `model.<id>.temperature` | `number` | `yes` | `user` | 按模型的采样 temperature。 |
| `model.<id>.top_p` | `number` | `yes` | `user` | 按模型的 top_p。 |
| `model.<id>.use_concise` | `boolean` | `yes` | `user` | 为此模型使用简洁工具描述包。 |

### `model_providers`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `model_providers.<name>` | `table` | `yes` | `user` | 具名自定义模型提供方定义。 |

### `models`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `models.agent_type` | `string` | `yes` | `user` | 没有按模型覆盖时的回退 agent_type。 |
| `models.allowed_models` | `string[]` | `pin` | `user` | 模型选择器、默认值和 `-m` 的 glob 允许名单。空表示不限制。 |
| `models.default` | `string` | `pin` | `user` | 新会话使用的模型。也可用 `GROK_DEFAULT_MODEL`、`--model`、`-m`。 |
| `models.default_reasoning_effort` | `string` | `yes` | `user` | 默认模型支持时的默认 reasoning effort。 |
| `models.disabled_models` | `string[]` | `yes` | `user` | 从目录中移除这些模型 ID。优先于 `hidden_models`。 |
| `models.extra_body` | `table` | `yes` | `user` | 应用于每个模型的额外 JSON body 字段；按模型的顶层键优先。 |
| `models.extra_headers` | `map<string,string>` | `yes` | `user` | 应用于每个模型的请求头；按模型的键优先。 |
| `models.hidden_models` | `string[]` | `yes` | `user` | 在选择器中隐藏这些模型 ID；`-m` 仍可选择。 |
| `models.image_description` | `string` | `yes` | `user` | 用于转写用户提供图片的视觉模型。 |
| `models.inference_idle_timeout_secs` | `number` | `yes` | `user` | 模型未设置时，流式推理的全局空闲超时。 |
| `models.max_completion_tokens` | `number` | `yes` | `user` | 模型未设置时的全局最大 completion token 默认值。 |
| `models.max_retries` | `number` | `yes` | `user` | 模型未设置时的全局推理重试默认值。 |
| `models.prompt_suggestion` | `string` | `yes` | `user` | 下一条提示幽灵文本的模型钉死。未设置则回退到远程，再回退到会话模型。 |
| `models.rate_limit_retry_threshold` | `number` | `yes` | `user` | 模型未设置时，限速请求的全局总尝试上限，受已解析的 `max_retries` 限制；配置后会关闭单独的子 agent 429 等待循环。 |
| `models.session_summary` | `string` | `yes` | `user` | 用于会话标题和摘要的模型。 |
| `models.stream_tool_calls` | `boolean` | `yes` | `user` | 全局工具调用流式请求形态；某些 BYOK 端点需要 false。 |
| `models.subagent_rate_limit_max_attempts` | `number` | `yes` | `user` | 未设置 `rate_limit_retry_threshold` 时，子 agent 429 等待循环尝试次数的全局默认；默认 8，最大 32，`0` 关闭等待循环。 |
| `models.temperature` | `number` | `yes` | `user` | 模型未设置时的全局采样 temperature 默认值。 |
| `models.top_p` | `number` | `yes` | `user` | 模型未设置时的全局 top_p 默认值。 |
| `models.web_search` | `string` | `pin` | `user` | 客户端 `web_search` 工具使用的模型。也可用 `GROK_WEB_SEARCH_MODEL`。 |

### `path_not_found_hints`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `path_not_found_hints` | `boolean` | `yes` | `user` | 用 CWD 提醒和近似名称建议丰富 path-not-found 错误。 |

### `paths`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `paths.extra_rule_dirs` | `string[]` | `yes` | `user` | 更多规则目录（每个包含 `*.md`）。 |
| `paths.extra_skill_dirs` | `string[]` | `yes` | `user` | 更多 skill 目录（每个包含 `<skill>/SKILL.md`）。 |

### `permission`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `permission.allow` | `string[]` | `yes` | `user` | 紧凑 allow 规则，例如 `Bash(git *)`。deny 压过 ask，ask 压过 allow。项目文件可以设置此项。 |
| `permission.ask` | `string[]` | `yes` | `user` | 紧凑 ask 规则。项目文件可以设置此项。 |
| `permission.deny` | `string[]` | `yes` | `user` | 紧凑 deny 规则。项目文件可以设置此项。 |
| `permission.rules` | `array of tables` | `yes` | `user` | 详细的 action/tool/pattern 对象规则。项目文件可以设置此项。 |

### `plugins`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `plugins.disabled` | `string[]` | `yes` | `user` | 要发现但不加载的插件 ID。项目文件可以设置此项。 |
| `plugins.enabled` | `string[]` | `yes` | `user` | 要启用的插件 ID；默认关闭的项目插件需要此项。 |
| `plugins.paths` | `string[]` | `yes` | `user` | 额外的插件目录。文件夹受信任时项目文件可以设置此项。 |

### `privacy`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `privacy.privacy_banner_acked` | `string` | `—` | `—` | 本地隐私横幅被关闭时的 RFC 3339 UTC 时间戳。pager 只读用户 `config.toml`。 |

### `relay`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `relay.enabled` | `boolean` | `yes` | `user` | 启用会话中继同步。 |

### `sandbox`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `sandbox.auto_allow_bash` | `boolean` | `pin` | `user` | 沙箱配置文件启用时跳过 bash 权限提示。也可用 GROK_SANDBOX_AUTO_ALLOW_BASH。 |
| `sandbox.profile` | `off / workspace / read-only / strict / string` | `pin` | `user` | 文件系统沙箱配置。也可用 `--sandbox` 和 GROK_SANDBOX。 |

### `session`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `session.auto_compact_threshold_percent` | `integer` | `yes` | `user` | 上下文用量达到此百分比（0–100）时自动压缩。 |
| `session.load_envrc` | `boolean` | `yes` | `user` | 把 `.envrc` 变量注入 bash。 |
| `session.title_prompt` | `string` | `yes` | `user` | 生成会话标题时使用的提示。未设置则用内置标题提示。 |
| `session.title_refresh_turns` | `integer[]` | `yes` | `user` | 刷新自动标题然后冻结的真实用户回合数。默认 `3`、`6`。 |

### `shell_environment_policy`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `shell_environment_policy.exclude` | `string[]` | `yes` | `user` | 从 bash 中丢掉的环境变量名。overlay 允许名单内。 |
| `shell_environment_policy.ignore_default_excludes` | `boolean` | `yes` | `user` | 跳过内置环境变量拒绝名单。overlay 允许名单内。 |
| `shell_environment_policy.include_only` | `string[]` | `yes` | `user` | 若设置，bash 只继承这些环境变量名。overlay 允许名单内。 |
| `shell_environment_policy.inherit` | `string` | `yes` | `user` | bash 继承哪些父进程环境变量名。overlay 允许名单内；不能注入值。 |
| `shell_environment_policy.set` | `map<string,string>` | `yes` | `user` | 向 bash 注入环境变量值。不在 overlay 允许名单内。 |

### `skills`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `skills.disabled` | `string[]` | `yes` | `user` | 要发现但不激活的 skill 名称。 |
| `skills.paths` | `string[]` | `yes` | `user` | 额外的 skill 目录。 |

### `storage`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `storage` | `table` | `yes` | `user` | 本地会话存储清理策略。 |

### `subagents`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `subagents.enabled` | `boolean` | `pin` | `user` | 子 agent / task 工具总开关。也可用 GROK_SUBAGENTS。 |
| `subagents.limit_behavior` | `queue / fail` | `yes` | `user` | 达到并发子 agent 上限时怎么做。 |
| `subagents.max_concurrent` | `integer` | `yes` | `user` | 最大并发子 agent 数。 |
| `subagents.max_depth` | `integer` | `yes` | `user` | 最大嵌套子 agent 深度（下限 1）。 |
| `subagents.models.<name>` | `string` | `yes` | `user` | 按子 agent 的模型 id 覆盖。 |
| `subagents.toggle.<name>` | `boolean` | `yes` | `user` | 启用或禁用单个子 agent 类型。省略的默认开启。 |

### `telemetry`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `telemetry.otel_enabled` | `boolean` | `pin` | `user` | 外部 OTEL 总开关。也可用 GROK_EXTERNAL_OTEL。 |
| `telemetry.otel_metrics_exporter` | `otlp / console / none` | `pin` | `user` | 外部 OTEL 指标导出器。也可用 OTEL_METRICS_EXPORTER。 |
| `telemetry.otel_logs_exporter` | `otlp / console / none` | `pin` | `user` | 外部 OTEL 日志导出器。也可用 OTEL_LOGS_EXPORTER。 |
| `telemetry.otel_endpoint` | `string` | `pin` | `user` | 外部 OTLP base 端点。也可用 OTEL_EXPORTER_OTLP_ENDPOINT。Pin 会剥掉开发者环境和未列出的用户/托管文件兄弟项，已列出的除外。 |
| `telemetry.otel_logs_endpoint` | `string` | `pin` | `user` | 日志信号的 OTLP 端点（原样）。也可用 OTEL_EXPORTER_OTLP_LOGS_ENDPOINT。 |
| `telemetry.otel_metrics_endpoint` | `string` | `pin` | `user` | 指标信号的 OTLP 端点（原样）。也可用 OTEL_EXPORTER_OTLP_METRICS_ENDPOINT。 |
| `telemetry.otel_protocol` | `http/protobuf / grpc` | `pin` | `user` | 外部 OTLP 传输。也可用 OTEL_EXPORTER_OTLP_PROTOCOL。Pin 会剥掉按信号的协议环境和未列出的文件兄弟项，已列出的除外。 |
| `telemetry.otel_logs_protocol` | `http/protobuf / grpc` | `pin` | `user` | 日志信号的 OTLP 协议。也可用 OTEL_EXPORTER_OTLP_LOGS_PROTOCOL。 |
| `telemetry.otel_metrics_protocol` | `http/protobuf / grpc` | `pin` | `user` | 指标信号的 OTLP 协议。也可用 OTEL_EXPORTER_OTLP_METRICS_PROTOCOL。 |
| `telemetry.otel_timeout` | `number` | `pin` | `user` | 导出超时（毫秒）。也可用 OTEL_EXPORTER_OTLP_TIMEOUT。 |
| `telemetry.otel_metric_export_interval` | `number` | `pin` | `user` | 指标导出间隔（毫秒）。也可用 OTEL_METRIC_EXPORT_INTERVAL。 |
| `telemetry.otel_certificate` | `string` | `pin` | `user` | 收集器额外 CA 证书的 PEM 路径。也可用 OTEL_EXPORTER_OTLP_CERTIFICATE。CA pin **不会**剥掉端点。 |
| `telemetry.otel_logs_certificate` | `string` | `pin` | `user` | 日志信号的 CA PEM 路径。也可用 OTEL_EXPORTER_OTLP_LOGS_CERTIFICATE。 |
| `telemetry.otel_metrics_certificate` | `string` | `pin` | `user` | 指标信号的 CA PEM 路径。也可用 OTEL_EXPORTER_OTLP_METRICS_CERTIFICATE。 |
| `telemetry.otel_client_certificate` | `string` | `pin` | `user` | mTLS 客户端证书的 PEM 路径。也可用 OTEL_EXPORTER_OTLP_CLIENT_CERTIFICATE。Pin 会剥掉凭据副本、开发者端点和未列出的文件兄弟项。 |
| `telemetry.otel_client_key` | `string` | `pin` | `user` | mTLS 客户端密钥的 PEM 路径。Token 从不写在此文件里。也可用 OTEL_EXPORTER_OTLP_CLIENT_KEY。 |
| `telemetry.otel_logs_client_certificate` | `string` | `pin` | `user` | 日志信号的 mTLS 客户端证书 PEM 路径。也可用 OTEL_EXPORTER_OTLP_LOGS_CLIENT_CERTIFICATE。 |
| `telemetry.otel_logs_client_key` | `string` | `pin` | `user` | 日志信号的 mTLS 客户端密钥 PEM 路径。也可用 OTEL_EXPORTER_OTLP_LOGS_CLIENT_KEY。 |
| `telemetry.otel_metrics_client_certificate` | `string` | `pin` | `user` | 指标信号的 mTLS 客户端证书 PEM 路径。也可用 OTEL_EXPORTER_OTLP_METRICS_CLIENT_CERTIFICATE。 |
| `telemetry.otel_metrics_client_key` | `string` | `pin` | `user` | 指标信号的 mTLS 客户端密钥 PEM 路径。也可用 OTEL_EXPORTER_OTLP_METRICS_CLIENT_KEY。 |
| `telemetry.otel_metrics_include_session_id` | `boolean` | `pin` | `user` | 把 session.id 附到指标上。也可用 OTEL_METRICS_INCLUDE_SESSION_ID。 |
| `telemetry.otel_log_user_prompts` | `boolean` | `pin` | `user` | grok_code.user_prompt 上提示文本的内容门闩。也可用 OTEL_LOG_USER_PROMPTS。pin 任一内容门闩而不列出兄弟项时，未列出的兄弟项默认关闭。 |
| `telemetry.otel_log_tool_details` | `boolean` | `pin` | `user` | 工具参数预览、路径和原名的元数据门闩。建议为 SIEM 关联打开。也可用 OTEL_LOG_TOOL_DETAILS。不含完整正文。 |
| `telemetry.otel_log_assistant_responses` | `boolean` | `pin` | `user` | grok_code.assistant_response 文本的内容门闩。未设置时跟随 otel_log_user_prompts，除非 requirements 里 pin 了兄弟门闩。仅用环境变量 OTEL_LOG_USER_PROMPTS=1 时，必须把此项设为 0（或 pin 为 false）才能得到只含提示的流。也可用 OTEL_LOG_ASSISTANT_RESPONSES。 |
| `telemetry.otel_log_tool_content` | `boolean` | `pin` | `user` | tool_input、tool_output、full_command 和 error_message 的正文门闩。与 details 独立；默认关闭。只有 CONTENT 会丢掉 MCP 原名和路径。也可用 OTEL_LOG_TOOL_CONTENT。 |
| `telemetry.trace_upload` | `boolean` | `pin` | `user` | 上传会话 traces。Requirements 的 pin 压过用户配置。 |

### `tools`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `tools.disable_zdr_incompatible_tools` | `boolean` | `yes` | `user` | 在 ZDR 下限制需要 xAI 托管输出的工具。也可用 GROK_DISABLE_ZDR_INCOMPATIBLE_TOOLS。 |
| `tools.media_gen.max_parallel_image_gen_calls` | `integer` | `yes` | `user` | 限制一个模型步骤中并行的 image_gen/image_edit 调用。也可用 GROK_MAX_PARALLEL_IMAGE_GEN_CALLS。 |
| `tools.media_gen.max_parallel_video_gen_calls` | `integer` | `yes` | `user` | 限制一个模型步骤中并行的 video_gen 调用。也可用 GROK_MAX_PARALLEL_VIDEO_GEN_CALLS。 |
| `tools.respect_gitignore` | `boolean` | `pin` | `user` | 为 true 时，搜索和读取工具跳过被 gitignore 的文件。也可用 GROK_RESPECT_GITIGNORE。 |
| `tools.zdr_video_output_s3` | `table` | `yes` | `user` | ZDR 视频输出的团队 S3 bucket。见 ZDR Video Storage。 |

### `toolset`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `toolset.ask_user_question.timeout_secs` | `number` | `yes` | `user` | ask_user_question 工具的超时。 |
| `toolset.bash.auto_background_on_timeout` | `boolean` | `yes` | `user` | 前台超时触发时把命令放到后台。 |
| `toolset.bash.login_shell_capture` | `boolean` | `yes` | `user` | 为 bash 捕获用户登录 shell 环境。overlay 允许名单内。 |
| `toolset.bash.max_timeout_secs` | `number` | `yes` | `user` | 模型请求的前台超时上限。 |
| `toolset.bash.output_byte_limit` | `number` | `yes` | `user` | 捕获的 bash 输出最大字节数。 |
| `toolset.bash.timeout_secs` | `number` | `yes` | `user` | 前台 bash 命令超时（秒）。 |
| `toolset.file_toolset` | `standard / hashline` | `yes` | `user` | 文件编辑工具方案。 |
| `toolset.web_fetch.allowed_domains` | `string[]` | `yes` | `user` | web_fetch 的域名允许名单覆盖。 |
| `toolset.web_fetch.proxy_endpoint` | `string` | `yes` | `user` | web_fetch 的出口代理 URL。也可用 GROK_WEB_FETCH_PROXY。 |
| `toolset.web_search.allowed_domains` | `string[]` | `yes` | `user` | 客户端 web_search 的域名允许名单。overlay 允许名单内。 |
| `toolset.web_search.excluded_domains` | `string[]` | `yes` | `user` | 客户端 web_search 的域名拒绝名单。overlay 允许名单内。 |

### `ui`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `ui.approval_mode` | `string` | `yes` | `user` | 已弃用；请用 `ui.permission_mode`。 |
| `ui.auto_dark_theme` | `string` | `yes` | `user` | `theme = auto` 且操作系统为深色时的主题。 |
| `ui.auto_light_theme` | `string` | `yes` | `user` | `theme = auto` 且操作系统为浅色时的主题。 |
| `ui.cancel_subagents_on_turn_cancel` | `ask / always_stop / always_continue` | `yes` | `user` | 取消父回合时对正在运行的子 agent 怎么做。 |
| `ui.collapsed_edit_blocks` | `boolean` | `yes` | `user` | 把编辑显示为一行 +N/-M 摘要。也可用 GROK_COLLAPSED_EDIT_BLOCKS。 |
| `ui.combine_queued_prompts` | `boolean` | `yes` | `user` | 把连续的纯后续合并成一个回合。 |
| `ui.compact_mode` | `boolean` | `yes` | `user` | 更密的消息间距。也可用 `/compact-mode`。 |
| `ui.confirm_before_rewind` | `boolean` | `yes` | `user` | 回退对话历史前询问。 |
| `ui.contextual_hints.image_input` | `boolean` | `yes` | `user` | 模型接受图片时的剪贴板图片粘贴提示。 |
| `ui.contextual_hints.plan_mode` | `boolean` | `yes` | `user` | 对规划类提示建议 plan 模式（Shift+Tab）。 |
| `ui.contextual_hints.send_now` | `boolean` | `yes` | `user` | 排队中回合后续之后，在空提示上按 Enter 立即发送。 |
| `ui.contextual_hints.small_screen` | `boolean` | `yes` | `user` | 在矮终端上建议 `/compact-mode`。 |
| `ui.contextual_hints.ssh_wrap` | `boolean` | `yes` | `user` | SSH 没有剪贴板接收端时建议 `grok wrap`。 |
| `ui.contextual_hints.undo` | `boolean` | `yes` | `user` | Ctrl+Z 恢复被清空的提示草稿的提示。 |
| `ui.contextual_hints.word_select` | `boolean` | `yes` | `user` | 在 fold/nav 选择下双击后，指向设置里的 Word select。 |
| `ui.cursor_blink` | `boolean` | `yes` | `user` | 强制闪烁（true）或静止（false）块光标。未设置则继承终端。 |
| `ui.default_selected_permission` | `string` | `yes` | `user` | 会话第一次提示时预选的批准行。也可用 GROK_DEFAULT_SELECTED_PERMISSION。 |
| `ui.display_refresh.auto_cadence_enabled` | `boolean` | `yes` | `user` | 把流式/滚动节奏匹配到显示器刷新率。也可用 GROK_DISPLAY_REFRESH_AUTO_CADENCE。 |
| `ui.follow_up_behavior` | `queue / steer` | `yes` | `user` | 中回合后续的路由。 |
| `ui.fork_secondary_model` | `string` | `yes` | `user` | fork 时次要 agent 的模型。默认用主默认模型。 |
| `ui.group_tool_verbs` | `boolean` | `yes` | `user` | 折叠连续的 read/search/list 工具行。也可用 GROK_GROUP_TOOL_VERBS。 |
| `ui.hunk_tracker_mode` | `agent_only / all_dirty / off` | `yes` | `user` | 文件变更 hunk 跟踪。也可用 GROK_HUNK_TRACKER 和 `--hunk-tracker-mode`。 |
| `ui.invert_scroll` | `boolean` | `yes` | `user` | 反转垂直滚动方向。也可用 GROK_INVERT_SCROLL。 |
| `ui.keep_text_selection` | `flash / hold / word_select` | `yes` | `user` | 应用内选择：短暂闪一下、保持，或双击选词。 |
| `ui.max_thoughts_width` | `number` | `yes` | `user` | 思考面板的列宽（40–500）。 |
| `ui.mouse_reporting_toggle` | `boolean` | `yes` | `user` | 在回滚中按 Ctrl+R 切换终端鼠标捕获。也可用 GROK_MOUSE_REPORTING_TOGGLE。 |
| `ui.page_flip_on_send` | `boolean` | `yes` | `user` | 把已发送的提示吸到视口顶部。 |
| `ui.permission_mode` | `default / ask / auto / always-approve` | `yes` | `user` | 默认工具权限行为。企业锁定用 requirements.toml。 |
| `ui.prompt_suggestions` | `boolean` | `yes` | `user` | 每回合后的下一条提示幽灵文本。也可用 GROK_PROMPT_SUGGESTIONS；远程紧急开关可在整机群关闭。 |
| `prompt_suggestions.max_output_tokens` | `number` | `yes` | `user` | 建议调用的可见输出 token 数；限制在 16–256，默认 64，另有 reasoning 预留。可被远程覆盖。 |
| `prompt_suggestions.temperature` | `number` | `yes` | `user` | 建议调用的采样 temperature（默认 0.2）。可被远程覆盖。 |
| `prompt_suggestions.reasoning_effort` | `none / minimal / low / medium / high` | `yes` | `user` | 建议调用的 reasoning effort；默认和 `none` 关闭 reasoning，其他值使用模型支持的 effort。可被远程覆盖。 |
| `ui.remember_tool_approvals` | `boolean` | `yes` | `user` | 显示按工具的 Always allow 选项。也可用 GROK_REMEMBER_TOOL_APPROVALS。 |
| `ui.render_mermaid` | `auto / on / off` | `yes` | `user` | mermaid 代码块如何渲染：可点击打开行或原始源码。 |
| `ui.screen_mode` | `fullscreen / minimal` | `yes` | `user` | 普通 `grok` 的默认渲染模式。需要重启。 |
| `ui.scroll_lines` | `integer` | `yes` | `user` | 每次滚动的行数（1–10）。也可用 GROK_SCROLL_LINES。 |
| `ui.scroll_mode` | `auto / wheel / trackpad` | `yes` | `user` | 滚动输入分类。也可用 GROK_SCROLL_MODE。 |
| `ui.scroll_speed` | `integer` | `yes` | `user` | 鼠标/触控板滚动速度倍率（1–100）。也可用 GROK_SCROLL_SPEED。 |
| `ui.show_thinking_blocks` | `boolean` | `yes` | `user` | 流式输出时显示 thinking/reasoning 块。也可用 GROK_SHOW_THINKING_BLOCKS。 |
| `ui.show_timeline` | `boolean` | `yes` | `user` | 用每回合刻度条代替滚动条。 |
| `ui.show_timestamps` | `boolean` | `yes` | `user` | 消息旁的时钟时间。也可用 `/timestamps`。 |
| `ui.simple_mode` | `boolean` | `yes` | `user` | 为 true 时用 readline 编辑提示；为 false 时用实验性 vim 提示键。 |
| `ui.status_line.command` | `string` | `yes` | `user` | `command` 状态行的脚本。Campaigns 会剥掉此路径；requirements 层仍会合并它。 |
| `ui.status_line.type` | `disabled / command` | `yes` | `user` | 快捷键栏上方可选的状态行。默认关闭。见状态行用户指南。 |
| `ui.theme` | `string` | `yes` | `user` | 颜色主题名，或用 `auto`/`system` 跟随操作系统。也可用 `/theme` 和 GROK_THEME。 |
| `ui.ui_theme` | `string` | `yes` | `user` | `ui.theme` 的遗留别名。 |
| `ui.vim_mode` | `boolean` | `yes` | `user` | 回滚中的 vim 键，不是提示里的。也可用 `/vim-mode`。 |
| `ui.voice_capture_mode` | `hold / toggle` | `yes` | `user` | 按住说话或按一下切换语音采集。 |
| `ui.voice_keybind_enabled` | `boolean` | `yes` | `user` | 启用 Ctrl+Space / F8 做语音听写。为 false 时 `/voice` 仍可用。 |
| `ui.voice_stt_language` | `string` | `yes` | `user` | 语音转文字语言代码或 `auto`。对本会话覆盖 `[voice].language`。 |
| `ui.yolo` | `boolean` | `pin` | `user` | 始终批准工具调用。Requirements 可以 pin 为 false 并拦截 `--yolo`。 |

### `version_overrides`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `version_overrides` | `array of tables` | `yes` | `user` | 合并前按 CLI 版本应用的配置补丁。见 `[[version_overrides]]`。 |

### `voice`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `voice.api_base` | `string` | `yes` | `user` | 语音转文字的 HTTPS API 根。未设置则继承 `[endpoints].xai_api_base_url`。 |
| `voice.language` | `string` | `yes` | `user` | 首选 STT 语言目录代码或 `auto`。 |
| `voice.sample_rate` | `number` | `yes` | `user` | STT 采集采样率（Hz）。 |

### `workflows`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `workflows.enabled` | `boolean` | `yes` | `user` | 启用 workflows。 |

### `worktree`

| 键 | 类型 / 取值 | Requirements | Managed | 说明 |
| --- | --- | --- | --- | --- |
| `worktree.auto_gc` | `table` | `yes` | `user` | 自动工作树垃圾回收策略。 |

## managed_config.toml

`managed_config.toml` 接受上表中的每一个键。它设置机群默认值，因此开发者自己的 `config.toml` 会覆盖它。需要别人还能调整的值用它，不能改的用 `requirements.toml`。

这条规则有一个例外：

| 键 | 行为 |
| --- | --- |
| `features.remote_fetch` | 托管值优先于开发者的。 |

Grok Build 先读 `/etc/grok/managed_config.toml`，再读 `$GROK_HOME/managed_config.toml`，后者由控制台保持同步。第二份里的值替换第一份里的值。

上表的 **Managed** 列是按键的答案：`fleet` 表示机群值站住，`user` 表示用户文件胜出，`—` 表示忽略此文件。

## requirements.toml

`requirements.toml` 是管理员强制的文件。位置：`$GROK_HOME/requirements.toml`（已签名缓存），然后 `/etc/grok/requirements.toml`，然后 macOS MDM `ai.x.grok`。`config.toml` 各表上的 **Requirements** 列列出此文件接受的每一个 `config.toml` 键（`pin` 或 `yes`）。省略的键不受约束。

这些键只存在于 `requirements.toml`：

| 键 | 类型 / 取值 | 默认 | 说明 |
| --- | --- | --- | --- |
| `fail_closed` | `boolean` | `false` | 已签名的 requirements 或 version_overrides 无法应用时拒绝启动；默认 false。 |
| `features.image_edit` | `boolean` | — | 钉住 image_edit 是否可用。仅 requirements；用户文件中的条目不被识别，未设置则保留远程配置的默认。 |
| `ui.disable_bypass_permissions_mode` | `boolean` | — | 锁定 always-approve 为关。该锁定只从 requirements 层强制执行；用户或托管文件中的 true 会被忽略。 |

## 设置被拒绝时会发生什么

| 情况 | Grok Build 会怎么做 |
| --- | --- |
| 开发者设置了你 pin 的键 | 使用 pin 住的值。`grok inspect` 会列出贡献该值的 requirements 文件。 |
| 开发者设置了你在 `managed_config.toml` 里下发的键 | 用他们的值，`features.remote_fetch` 除外。必须站住就改 pin 该键。 |
| `requirements.toml` 缺失或其签名无法验证 | pin 不生效，Grok Build 在没有它们的情况下启动。改设 `fail_closed = true` 则改为拒绝启动。 |
| pin 住的键给出了此版本无法识别的值 | 忽略该键，文件其余部分仍生效。 |

## 检查当前生效的内容

在开发者的机器上运行 `grok inspect`。它会列出每一个贡献过的配置文件，包括 requirements 和托管层，因此一条没有生效的策略用一条命令就能看见。
