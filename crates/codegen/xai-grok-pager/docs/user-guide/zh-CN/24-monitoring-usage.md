# 监控用量（外部 OpenTelemetry）

> **状态：alpha。** 下面的 schema 已版本化（`grok_code.schema.version = v1`）；
> 可能在不通知的情况下发生加法变更，重命名/删除会 bump
> 版本并在 changelog 里点名。

Grok CLI 可以把用量**指标**和**事件**导出到你组织自己的
OpenTelemetry collector，让平台团队在整支车队上监控采用、token
消耗、工具权限决定和错误——而不让任何数据流经 SpaceXAI。

## 相关设置

这些旋钮彼此独立（也独立于本指南的外部 OTEL 流）：

| 设置 | 如何设置 |
|---------|---------------|
| 遥测总开关 | `[features] telemetry` / `GROK_TELEMETRY_ENABLED` |
| 编码数据、保留与训练 | 设置 — `/privacy` 打开该行 |
| 跟踪上传 | `[telemetry] trace_upload` / `GROK_TELEMETRY_TRACE_UPLOAD` |
| 外部 OpenTelemetry | `GROK_EXTERNAL_OTEL` / `[telemetry] otel_*`（本指南） |

另见 [认证](02-authentication.md#related-settings) 和
[配置](05-configuration.md#telemetry)。

## 外部 OTEL 流

外部流是：

- **默认关闭**，并且需要*双重选择加入*（总开关**以及**
  显式的导出器选择）。
- **默认无内容**：没有提示、没有助手散文、没有代码、没有文件
  路径（只有扩展名）、没有工具参数、没有 bash 命令，MCP/skill/plugin
  名称折叠成类别。可选的内容门闩会重新启用其中一些。
- **在结构上与 SpaceXAI 内部遥测分开**：其导出器只携带
  你配置的头，从不携带 SpaceXAI 凭据。
- **独立于 SpaceXAI 数据保留选择退出**：即使
  `telemetry` 已禁用，以及对于 ZDR（零数据保留）团队，它也能工作。那些
  设置管辖 SpaceXAI 侧的保留；外部流只由你自己的
  OTEL 配置管辖。

### ZDR 与此流

`/privacy` 和 Zero Data Retention **不会**禁用此流。ZDR 关掉
SpaceXAI 侧的保留（产品分析、会话跟踪上传、
编码数据共享）。它不会静音 `GROK_EXTERNAL_OTEL`。

流开启时：

- `user.id`、`session.id` 以及组织/团队/部署 id 始终导出。
- 只要 OAuth/网关认证有非空地址，`user.email` 就会附在日志
  **和**指标上。它是身份，不是内容门闩，除了关掉整条流
  之外无法钉住。
- 提示文本、助手 `response` 和工具正文只有在其门闩开启时才导出。第一方
  产品分析永远收不到那些正文。

要让 ZDR 机器对 collector 保持静默，钉住 `otel_enabled = false`（或不
启用该流）。对于只指标的 SIEM，把全部四个 `otel_log_*` 键钉为
`false`。

## 快速开始

```bash
export GROK_EXTERNAL_OTEL=1                  # master switch
export OTEL_METRICS_EXPORTER=otlp
export OTEL_LOGS_EXPORTER=otlp
export OTEL_EXPORTER_OTLP_PROTOCOL=http/protobuf  # or grpc
export OTEL_EXPORTER_OTLP_ENDPOINT=https://collector.corp.example:4318
export OTEL_EXPORTER_OTLP_HEADERS="Authorization=Bearer <collector-token>"
grok
```

单独的 `GROK_EXTERNAL_OTEL=1` **什么也不启用**——你还必须至少选择
一个导出器。反过来，没有总开关时，单独的 `OTEL_*` 变量
什么也不启用。

## 环境变量

| 变量 | 默认 | 含义 |
|---|---|---|
| `GROK_EXTERNAL_OTEL` | `0` | 总开关。与控制 SpaceXAI 内部产品分析的 `GROK_TELEMETRY_ENABLED` 不同——两者管辖指向相反的数据流。 |
| `OTEL_METRICS_EXPORTER` | `none` | `otlp` \| `console` \| `none`。 |
| `OTEL_LOGS_EXPORTER` | `none` | `otlp` \| `console` \| `none`。门闩事件流。 |
| `OTEL_EXPORTER_OTLP_PROTOCOL` | `http/protobuf` | `http/protobuf` \| `grpc`。两种信号的基协议。 |
| `OTEL_EXPORTER_OTLP_LOGS_PROTOCOL` / `..._METRICS_PROTOCOL` | — | 按信号的协议覆盖（取值与基协议相同）。无法识别的值会禁用该流。 |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | HTTP 为 `http://localhost:4318`，gRPC 为 `http://localhost:4317` | 基端点。对 `http/protobuf`，按 OTLP 规范追加 `/v1/logs` 和 `/v1/metrics`；对 `grpc`，collector 端点原样使用。路径追加使用**该信号的**协议。 |
| `OTEL_EXPORTER_OTLP_LOGS_ENDPOINT` / `..._METRICS_ENDPOINT` | — | 按信号的覆盖，原样使用。对 gRPC 这些通常应是不带 `/v1/...` 路径的 collector 端点。 |
| `OTEL_EXPORTER_OTLP_HEADERS`（+ 按信号的变体） | — | Collector 认证（`k=v,k2=v2`）。外部导出器发送的**唯一**头，也是唯一支持的 collector 认证机制（没有配置文件 headers 键——令牌从不落盘）。 |
| `OTEL_EXPORTER_OTLP_CERTIFICATE`（+ 按信号的变体） | — | 含额外受信任 CA 证书的 PEM 包路径，用于验证 collector——用于私有/企业 CA 后面的 collector。叠加到默认信任根（系统存储和嵌入的 Mozilla 根）。也可通过 `[telemetry] otel_certificate` 设置。 |
| `OTEL_EXPORTER_OTLP_CLIENT_CERTIFICATE` / `OTEL_EXPORTER_OTLP_CLIENT_KEY`（+ 按信号的 `…_LOGS_…` / `…_METRICS_…` 变体） | — | mTLS 客户端身份的 PEM **路径**。证书和密钥都必须设置（基座或同一信号）；半配置会被忽略并给出警告。仅未加密的 PEM 密钥。也可通过 `[telemetry] otel_client_certificate` / `otel_client_key` 设置。 |
| `OTEL_EXPORTER_OTLP_TIMEOUT` | `10000`（ms） | 导出超时。 |
| `OTEL_METRIC_EXPORT_INTERVAL` | `60000`（ms） | 指标导出间隔。 |
| `OTEL_BLRP_SCHEDULE_DELAY`（或别名 `OTEL_LOGS_EXPORT_INTERVAL`） | `5000`（ms） | 日志批间隔。 |
| `OTEL_EXPORTER_OTLP_METRICS_TEMPORALITY_PREFERENCE` | `delta` | `delta` \| `cumulative`。 |
| `OTEL_METRICS_INCLUDE_SESSION_ID` | `1` | 把 `session.id` 附到指标上（基数选择退出）。 |
| `OTEL_METRICS_INCLUDE_VERSION` | `0` | 把 `app.version` 附到指标上。 |
| `OTEL_LOG_USER_PROMPTS` | `0` | 内容门闩：`grok_code.user_prompt` 上的提示文本（60 KB 上限，秘密已擦除）。 |
| `OTEL_LOG_ASSISTANT_RESPONSES` | 未设置时跟随提示 | 内容门闩：`grok_code.assistant_response` 上的 `response`（60 KB 上限，秘密已擦除）。未设置时跟随 `OTEL_LOG_USER_PROMPTS`；显式 `0` 在提示保持开启时关掉回复。`response_length` 始终导出。带 `OTEL_LOG_USER_PROMPTS=1` 的仅环境变量车队必须设 `OTEL_LOG_ASSISTANT_RESPONSES=0`（或在 requirements 里钉住关闭）才能保持只提示的流。 |
| `OTEL_LOG_TOOL_DETAILS` | `0` | 元数据门闩：4 KB `tool_parameters` 预览、完整文件路径、逐字 MCP/skill/plugin 名称。**不包括**完整正文。DETAILS 开、CONTENT 关是企业默认。 |
| `OTEL_LOG_TOOL_CONTENT` | `0` | 正文门闩：`tool_input`、`tool_output`、`full_command`，以及失败的 `error_message`（秘密已擦除；input/output/command 为 60 KB，error_message 为 4 KB）。独立于 DETAILS——CONTENT **并不**隐含 DETAILS。默认关闭。 |

### 推荐的车队门闩

企业默认是 **DETAILS 开、CONTENT 关**：元数据（路径、4 KB
`tool_parameters`、逐字 MCP/skill/plugin 名称），没有 Read、bash 或
MCP 结果正文。只有当 collector 必须存储那些正文时才打开 CONTENT。
CONTENT **并不**隐含 DETAILS——只有 CONTENT 会得到 60 KB 的
`tool_output`，而 `tool_name` / `mcp_tool.name` 仍折叠为
`mcp_tool`。

```toml
[telemetry]
otel_log_user_prompts = false
otel_log_assistant_responses = false   # unset would follow prompts; pin off
otel_log_tool_details = true           # metadata for SIEM join
otel_log_tool_content = false          # bodies; independent of details
```

`OTEL_RESOURCE_ATTRIBUTES` 被有意忽略：资源由固定的、经审计的
属性集构建。

> **迁移说明：** 较旧的发行版可能与产品自己的分析流水线共享
> `OTEL_EXPORTER_OTLP_*`。该行为已弃用：当
> `GROK_EXTERNAL_OTEL` 已设置时，产品分析忽略那些变量，并且
> CLI 拒绝在产品分析已经消费过它们的任何配置里激活外部流——
> 你的 collector 只接收你选择加入的外部流。

## 配置文件

组织默认放在 `config.toml` 里已有的 `[telemetry]` 表下
（环境变量胜出）。键是其他 `[telemetry]` 设置的
`otel_` 前缀对等项：

```toml
[telemetry]
otel_enabled = true
otel_metrics_exporter = "otlp"
otel_logs_exporter = "otlp"
otel_endpoint = "https://collector.corp.example:4318"
otel_protocol = "http/protobuf"  # or "grpc"
# Optional PEM *paths* for private-CA trust and mTLS (never PEM contents):
otel_certificate = "/etc/ssl/corp-ca.pem"
otel_client_certificate = "/etc/ssl/client.crt"
otel_client_key = "/etc/ssl/client.key"
otel_log_user_prompts = false   # admins can pin these via requirements
otel_log_assistant_responses = false
otel_log_tool_details = false   # code default; SIEM fleets usually true — see Recommended fleet gates
otel_log_tool_content = false
```

配置键是 `[telemetry]` 下的 `otel_*`；**环境变量保持其
标准 OTEL 名称**（`GROK_EXTERNAL_OTEL`、`OTEL_*`）以便与生态
互操作，因此两层有意使用不同的命名空间。
`otel_protocol` 配置键映射到 `OTEL_EXPORTER_OTLP_PROTOCOL`。对 CA 和
客户端身份，环境变量胜过配置文件路径。

有意没有 `headers` 键：通过 `OTEL_EXPORTER_OTLP_HEADERS` 提供
collector 认证，这样令牌从不存盘。证书和密钥配置键
**只是路径**——永远不要把私钥材料嵌进 TOML。

签名的 `requirements.toml` 里每一个**存在的** `[telemetry] otel_*` 键都是
**钉住**（环境无法覆盖它）。`managed_config.toml` 不是锁——那里
环境仍胜出。钉住 `otel_endpoint` 会剥掉开发者通用的和
按信号的端点环境变量，**以及未列出的用户/托管文件兄弟项**，除了
你也列出的端点。钉住客户端
证书/密钥也会剥掉开发者端点和未列出的文件兄弟项。钉住 CA（`otel_certificate`）
**不会**剥掉端点。若 requirements 列出
`otel_log_user_prompts` / `otel_log_tool_details` /
`otel_log_assistant_responses` / `otel_log_tool_content` 中的任何一个并省略兄弟项，被省略的门闩
默认**关闭**（不要依赖跨该边界的
提示→回复回退）。

外部流只导出**日志和指标**（没有面向客户的
跟踪导出器）。

车队启用是一份签名的 `requirements.toml`（目的地、导出器
和内容门闩在一起）。`user.email` 不是钉住键——它跟随
OAuth/网关身份。头留在剥离后启动器给出的进程环境里，
从不进这份 TOML。

## 启动抑制（为什么前几秒什么都到不了）

因为 xAI 可以在整支车队上强制禁用此流，CLI 在启动时保持发射
关闭，直到它知道该开关是否已设置——它从 `/v1/settings` 获取
车队策略，然后才开始导出。在健康的设置里这远不到一秒，而且看不见。

**等待是有界的**，因此无法到达 xAI 的部署仍会导出：

- 若车队策略根本无法应用——`[features] remote_fetch = false`，或
  `[endpoints] cli_chat_proxy_base_url` 指向 xAI 以外的地方——该
  流立刻开始，由你的本地配置管辖。
- 若策略获取失败或从未完成（防火墙主机、离线
  笔记本），尝试耗尽后发射仍会开始，并且在所有
  情况下不晚于启动后 30 秒。

之后到达的车队策略仍会应用；它永远只能
*收紧*（禁用该流或强制关掉内容门闩），从不启用
你本地配置没有启用的东西。

若你的 collector 完全收不到任何东西，检查调试日志
（`grok --debug`）里的 `external otel:` 行——它们记录该流
是否解析了配置，以及它是在导出还是被抑制。

## 资源属性

| 属性 | 值 |
|---|---|
| `service.name` | `grok-cli` |
| `service.version`、`client.version` | 构建/客户端版本 |
| `app.entrypoint` | `cli` \| `headless` \| `agent` |
| `terminal.type` | 终端模拟器品牌 |
| `grok_code.schema.version` | `v1` |

身份属性（`user.id`，以及已知时的 `organization.id` / `team.id` /
`deployment.id`）在认证完成后附到每个指标数据点和每个事件上。
只要会话用 OAuth 或带非空地址的网关账户登录，`user.email` 就会附在日志
**和**指标上——它是身份，不是内容门闩，并且从不取自
git、API key 或部署密钥。`prompt.id`（按提示的 UUID）只出现在
事件上，从不出现在指标上。

## 指标（meter 范围 `ai.xai.grok_code`）

| 指标 | 单位 | 属性 |
|---|---|---|
| `grok_code.session.count` | `{session}` | 仅基属性 |
| `grok_code.token.usage` | `{token}` | `type` = `input` \| `output` \| `reasoning` \| `cache_read`；`model` |
| `grok_code.turn.count` | `{turn}` | `outcome` = `completed` \| `cancelled` \| `error`；`model` |
| `grok_code.turn.ttft` | `ms` | `model` |
| `grok_code.turn.ttfm` | `ms` | `model` |
| `grok_code.tool.decision` | `{decision}` | `tool_name`，`decision` = `allow` \| `deny` \| `cancelled` \| `followup`，`access_kind`，`permission_mode` |
| `grok_code.tool.usage` | `{call}` | `tool_name`，`outcome` |
| `grok_code.error.count` | `{error}` | `error_category`，`model` |
| `grok_code.startup.total` | `ms` | `outcome` = `ok` \| `timeout` \| `error`；`auth_mode` |
| `grok_code.startup.interactive` | `ms` | `auth_mode` |
| `grok_code.startup.phase_duration` | `ms` | `phase`，`outcome`，`auth_mode` |
| `grok_code.startup.timeout` | `{timeout}` | `stuck_in`，`auth_mode` |

`startup.total` 衡量从进程启动到可用会话，每个进程记录一次；
`outcome` = `timeout` 或 `error` 表示启动结束时没有可用会话。
`startup.interactive` 记录从进程启动到活动循环确认写出的第一帧，
每个进程一次。
`phase_duration` 按步骤拆分连接尝试（`config_load`、
`managed_policy`、`bootstrap`、`model_catalog`、`worker_spawn`、
`leader_connect`、`acp_initialize`、`eager_auth`）；按其 `outcome`
过滤（`ok` | `timeout` | `cancelled` | `error`），以免截断样本扭曲
`ok` 百分位。后面的 `app_init`
和 `session_create` 阶段出现在日志时间线和摘要
字符串里，不在此指标中。超时上的 `stuck_in` 点名尚未
完成的步骤。那往往不是耗时最长的步骤，因为不暂停就跑完的步骤
会在超时被记录之前结束。Grok 打印的错误
消息会点名最长的步骤，因此同一次超时两者可能点名
不同步骤。用 `phase_duration` 比较它们。
`auth_mode` 是 `personal`、`team`、`deployment` 或 `unknown`：
启动成本因种类而异，所以比较前先按它拆分。

`turn.ttft` 是从回合开始到任一通道第一个 token
（推理、文本或工具调用）的时间，`turn.ttfm` 是从回合开始到第一条
助手文本消息（排除推理和工具调用）的时间，同一时钟上每回合一个样本，
因此 `ttft` 从不超过 `ttfm`。没有产生模型输出的回合不记录
`ttft`；只有推理或只有工具的回合记录 `ttft`
但不记录 `ttfm`。

没有 `cost.usage` 指标：把 `grok_code.token.usage` 与你自己的
价目表连接。`lines_of_code.count` 和 `active_time.total` 计划在
后续阶段。

`tool_name` 取值：内置工具名原样通过；除非
`OTEL_LOG_TOOL_DETAILS=1`，MCP 工具折叠为 `mcp_tool`，其他非内置工具
折叠为 `custom_tool`。

## 事件（OTLP 日志记录）

每个事件都携带 `event.sequence`、`session.id`、`turn_number`（回合内）、
`prompt.id`，以及身份属性。门闩图例：**details** =
需要 `OTEL_LOG_TOOL_DETAILS`，**prompts** = 需要
`OTEL_LOG_USER_PROMPTS`，**responses** = 需要 `OTEL_LOG_ASSISTANT_RESPONSES`
（未设置时跟随 prompts 门闩），**content** = 需要 `OTEL_LOG_TOOL_CONTENT`
（独立于 details；默认关闭）；流活动时其他一切始终导出。

| `event.name` | 属性 |
|---|---|
| `grok_code.session_start` | `model`，`permission_mode`，`mcp_server_count`，`plugin_count`，`skill_count`，`hook_count`，`memory_enabled`，`is_git_repo`，`client_identifier` |
| `grok_code.session_end` | `duration_secs`，`turn_count`，`tool_call_count`，`compaction_count`，`model` |
| `grok_code.user_prompt` | `prompt_length`，`model`，`screen_mode?`（`fullscreen` \| `inline` \| `minimal` \| `headless` \| `other`），`command_name?`（斜杠/skill 名，始终开启的元数据）；`prompt`（**prompts**） |
| `grok_code.assistant_response` | `response_length`；`response`（**responses**；仅工具的回合省略） |
| `grok_code.turn_completed` | `outcome`，`duration_ms`，`tool_call_count`，`model`，`error_category?`，`cancellation_category?` |
| `grok_code.api_request` | `model`，`duration_ms`，`stop_reason?`，`input_tokens`，`output_tokens`，`reasoning_tokens`，`cache_read_tokens` |
| `grok_code.api_error` | `error_category`，`model`，`status_code?`，`duration_ms?` |
| `grok_code.tool_result` | `tool_name`，`outcome`，`success`，`duration_ms`，`file_extension`，`tool_use_id`；缩减的 `mcp_tool.name` / `mcp_server.name` 始终（**details** 下为逐字）；`tool_parameters` 预览 + `file_path`（**details**）；`tool_input`，`tool_output`，`full_command`，`error_message`（**content**） |
| `grok_code.tool_decision` | `tool_name`，`decision`，`access_kind`，`permission_mode`，`source`，`tool_use_id`；缩减的 MCP 名始终（**details** 下为逐字）；`tool_parameters` 预览（**details**）；`tool_input`，`full_command`（**content**） |
| `grok_code.mcp_server_connection` | `status`，`transport_type`，`duration_ms`，`tool_count?`，`error_type?`；缩减的 `mcp_server.name` 始终（**details** 下为逐字）；`error_message`（**content**） |
| `grok_code.permission_mode_changed` | `from_mode`，`to_mode`，`trigger` |
| `grok_code.skill_activated` | `skill_source`，`trigger` = `slash_command` \| `skill_md_read` \| `skill_tool`；`skill.name`（**details**） |
| `grok_code.plugin_loaded` | `install_kind?`，`success`，`error_category?`；`plugin_name`（**details**） |
| `grok_code.compaction` | `duration_ms`，`tokens_before`，`tokens_after`，`model?` |
| `grok_code.subagent` | `phase` = `launched` \| `completed`，`subagent_type?`，`outcome?`，`duration_ms?` |
| `grok_code.auth` | `auth_method` |
| `grok_code.internal_error` | `error_type`（仅类别——没有消息，没有位置） |
| `grok_code.model_switched` | `from_model`，`to_model`，`success`，`error_code?` |

## 隐私模型

三套独立的失败关闭机制守护线上格式：

1. **类型化 schema**：属性键是封闭枚举；其外的任何东西
   都不能被附加。
2. **发射时擦除**：每个字符串都经过秘密形状擦除和
   家目录擦除，并截断（嵌套工具参数字符串每条 512→128 字符，
   DETAILS 的 4 KB `tool_parameters` 预览 / CONTENT 的 `error_message`，
   `prompt` / `response` / `tool_input` / `tool_output` /
   `full_command` 为 60 KB）。
3. **导出时校验**：任何携带非 schema 键、已关闭门闩键
   或未擦除秘密形状的记录，在离开进程前会被丢掉；带有
   schema 外属性键的指标导出整批丢掉。

从不导出：思考/推理文本、原始 API 请求正文、`api_key.id`、
机器指纹、订阅档位。提示文本、助手 `response`、
文件路径、工具参数预览，以及 CONTENT 正文（`tool_input`、`tool_output`、
`full_command`、`error_message`）只有在其门闩开启时才导出；第一方
产品分析永远收不到那些正文。
只要 OAuth/网关认证有地址，`user.email` 就会导出（不是内容
门闩）。

## Collector 配置示例

```yaml
receivers:
  otlp:
    protocols:
      http:
        endpoint: 0.0.0.0:4318
      grpc:
        endpoint: 0.0.0.0:4317

processors:
  batch:

exporters:
  prometheus:
    endpoint: 0.0.0.0:9464

service:
  pipelines:
    metrics:
      receivers: [otlp]
      processors: [batch]
      exporters: [prometheus]
    logs:
      receivers: [otlp]
      processors: [batch]
      exporters: []   # point at your log backend (loki, elasticsearch, …)
```

示例查询（PromQL，配合上面的 Prometheus 导出器）：

```promql
# Tokens by model and type across the org, 1h rate
sum by (model, type) (rate(grok_code_token_usage_total[1h]))

# Sessions per team per day
sum by (team_id) (increase(grok_code_session_count_total[1d]))

# Tool-permission denial ratio
sum(rate(grok_code_tool_decision_total{decision="deny"}[1h]))
  / sum(rate(grok_code_tool_decision_total[1h]))
```

## 调试

设 `OTEL_LOGS_EXPORTER=console` / `OTEL_METRICS_EXPORTER=console` 把
已擦除的记录打印到 **stderr**（在 `agent`/`headless` 入口被抑制，
以保持捕获的日志干净）。导出错误从不出现在 TUI 里；请检查
调试日志。
