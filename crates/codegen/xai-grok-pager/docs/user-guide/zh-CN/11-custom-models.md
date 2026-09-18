# 自定义模型

Grok 可以连接自定义模型端点，用于替代提供商、自托管模型，以及覆盖内置设置。本指南说明如何选择模型、配置端点，以及接入第三方提供商。

---

## 默认模型

默认情况下，Grok 使用 SpaceXAI 托管的模型，新会话从 `grok-4.5` 开始。默认模型无需配置。用 `grok login` 或 API 密钥认证后即可开始会话。

列出所有可用模型：

```bash
grok models
```

---

## 选择模型

### CLI 标志

```bash
grok -p "Hello" -m grok-4.6
```

### 斜杠命令

在 TUI 中，会话进行时切换模型：

```
/model grok-4.6
```

或使用别名：

```
/m grok-4.6
```

### 模型选择器（Ctrl+M）

在回看窗格按 `Ctrl+M` 打开模型选择器。它列出所有可用模型，包括内置和自定义，一次按键即可切换。输入框聚焦时，`Ctrl+M` 改为开关多行输入——用 `/model` 即可不离开输入框完成切换。

### 集群允许列表（`requirements.toml`）

企业主机可以在签名的 `requirements.toml` 中钉死**可选**集合——不只是默认值。该列表**替换**任何用户 `allowed_models`（不是并集），因此 `/model`、`Ctrl+M` 和 `-m` 不能提供列表之外的模型。

```toml
[models]
default = "grok-4.5"
allowed_models = ["grok-4.5", "grok-4*"]
```

集群固定值匹配的是**模型 id**（不是用户自选的目录键），因此本地 `[model.<name>]` 条目不能扩大集合。用户配置的 `allowed_models` 仍匹配目录键或模型 id。省略该键则保留用户配置。空数组表示不受限。存在但不可读的固定值会失败即封锁（没有任何可选模型）。默认值或 `-m` 的值若在固定集合之外，会在拉取模型目录后被拒绝——联系管理员；该列表用户不可编辑。

### 配置默认值

在 `~/.grok/config.toml` 中设置持久默认：

```toml
[models]
default = "grok-4.5"
```

---

## 支持的 API 后端

Grok 支持三种 API 后端。在 `[model.*]` 配置中设置 `api_backend`，选择该模型使用的协议：

| 值 | API | 默认 |
|-------|-----|---------|
| `"chat_completions"` | OpenAI Chat Completions（`/v1/chat/completions`） | 是 |
| `"responses"` | OpenAI Responses（`/v1/responses`） | |
| `"messages"` | Anthropic Messages（`/v1/messages`） | |

省略 `api_backend` 时，Grok 使用 `chat_completions`。

要发送提供商特定的认证或版本头——例如 Anthropic 的 `x-api-key`——使用下面描述的 `extra_headers` 字段。Grok 会原样把这些头随每一次请求发到端点。

---

## 配置自定义模型

在 `~/.grok/config.toml` 的 `[model.<name>]` 段中添加自定义模型端点：

```toml
[model.my-model]
model = "model-id"                        # 发给 API 的模型标识符
base_url = "https://api.example.com/v1"   # OpenAI 兼容端点
name = "Display Name"                     # 显示在模型选择器中
description = "Model description"          # 可选描述
api_key = "sk-..."                        # 该提供商的 API 密钥（可选）
env_key = "XAI_API_KEY"                   # 存放 API 密钥的环境变量（可选；字符串或数组）
keychain_service = "grok"                 # 系统钥匙串 service（可选；需同时设 keychain_account）
keychain_account = "new-api"              # 系统钥匙串 account（可选）
api_backend = "chat_completions"          # "chat_completions"、"responses" 或 "messages"
temperature = 0.7                         # 采样温度
top_p = 0.95                              # 核采样参数
max_completion_tokens = 8192              # 每次响应的最大 token 数
context_window = 128000                   # 总上下文窗口（token）
extra_headers = { "x-api-key" = "sk-..." } # 额外请求头，原样发送（可选）
query_params = { api-version = "2026-07-22" } # 追加到每个请求 URL 的查询参数（可选）
env_http_headers = { "X-Tenant" = "TENANT_TOKEN" }    # 来自环境变量的头，在构建客户端时解析（可选）
extra_body = { enable_thinking = true }               # 合并进推理请求 JSON body 的额外字段（可选）
```

### 凭据解析

Grok 按此顺序解析 API 密钥：

1. 模型配置中的 `api_key` 字段
2. `env_key` 命名的环境变量——单个字符串或名称数组。第一个已设置且非空的值胜出（例如 `env_key = ["ANTHROPIC_AUTH_TOKEN", "LC_ANTHROPIC_AUTH_TOKEN"]`，用于 SSH `LC_*` 转发）
3. `keychain_service` + `keychain_account` 指向的系统钥匙串项（macOS 钥匙串或 Windows 凭据管理器）。项缺失或为空时，等同于未设置的 `env_key`
4. 你的已登录会话令牌（来自 `grok login`），适用于没有自己的 `api_key`/`env_key`/钥匙串的模型
5. `XAI_API_KEY` 环境变量（全局回退；Grok 也为向后兼容接受 `GROK_CODE_XAI_API_KEY`）

不想把密钥写进 `config.toml` 时，存进钥匙串再引用：

```toml
[model.codex]
model = "gpt-5.1-codex"
base_url = "https://new-api.example/v1"
keychain_service = "grok"
keychain_account = "new-api"
```

```bash
# macOS
security add-generic-password -a "new-api" -s "grok" -w "sk-..."
```

Windows 的写入方式见 [认证](02-authentication.md#系统钥匙串macos-钥匙串windows-凭据管理器)。两个钥匙串字段必须都设置。Grok 不会记录或回写读到的密钥。

### 上下文窗口

`context_window` 值告诉 Grok 何时触发自动压缩。覆盖已知模型时，Grok 继承该模型的上下文窗口。定义新模型且省略 `context_window` 时，Grok 默认为 200,000 token，因此请显式设置以匹配你的提供商。

### 全局默认头

要对目录中的*每一个*模型——内置、从 `/v1/models` 预取，或自定义——应用相同的头，在全局 `[models]` 段设置一次，而不要按模型重复：

```toml
[models]
extra_headers = { "X-Request-Tags" = "team=example,env=prod" }
```

它们作为每个模型推理请求的基线。按模型的 `[model.<id>].extra_headers` 条目**按键**覆盖全局默认（不区分大小写匹配）：模型上设置的键胜出，仅全局有的键仍由该模型继承。与按模型字段一样，它们只挂在该模型的推理调用上——不挂到图像生成或视频生成等独立服务——因此适合做归因标签（例如费用追踪），而不必每次出现新模型都再声明一遍。

### 全局默认值

一些常见的按模型设置也可以在 `[models]` 下设一次，作为*每一个*模型的默认。按模型的 `[model.<id>]` 值始终胜出；全局只在模型（或服务器的模型列表）未设置该字段时填补：

```toml
[models]
temperature                 = 0.7
top_p                       = 0.95
max_completion_tokens       = 8192
max_retries                 = 8
rate_limit_retry_threshold  = 4
inference_idle_timeout_secs = 600
subagent_rate_limit_max_attempts = 8
stream_tool_calls           = true
```

这是一小套固定的环境级旋钮。标识特定模型的设置（`model`、`base_url`、`api_key`、`context_window` 等）不能这样给默认值；另有专门配置的几项——自动压缩（`[session]`）、系统提示标签（`[agent]`）和推理力度（`[models].default_reasoning_effort`）——仍留在原处。

`rate_limit_retry_threshold` 和 `subagent_rate_limit_max_attempts` 为子 agent 选择不同的 429 重试路径。配置 `rate_limit_retry_threshold` 后由采样器接管这些重试，并关闭单独的子 agent 等待循环，包括其 150 秒累计等待预算和等待遥测。`subagent_rate_limit_max_attempts` 仅在采样器阈值未设置时生效。

> **关于 `stream_tool_calls` 的说明：** 这一项影响请求*形态*，不只是采样。少数端点（某些 BYOK 提供商）期望它保持未设置；若全局 `stream_tool_calls = true` 让这类模型出问题，在其 `[model.<id>]` 块中用 `stream_tool_calls = false` 把它排除。

### 请求查询参数

有些网关用查询字符串做路由或版本。`query_params` 会把百分号编码的查询参数追加到 Grok 为该模型发出的每一个请求。例如，用这种方式选择 API 版本的网关：

```toml
[model.my-gateway]
model = "my-model"
base_url = "https://gateway.example/v1"
api_backend = "responses"
env_key = "GATEWAY_API_KEY"
query_params = { api-version = "2026-07-22" }
```

也出现在 `base_url` 查询字符串中的键会被覆盖（后值胜出）而不是重复。查询参数会保存在会话中，因此不要把密钥放进去：密钥请用 `env_http_headers`。

### 环境变量头

`env_http_headers` 把请求头映射到提供其值的环境变量名，因此按请求的密钥不必写进 `config.toml`：

```toml
[model.gateway]
model = "my-model"
base_url = "https://gateway.example/v1"
env_http_headers = { "X-Tenant-Token" = "GATEWAY_TENANT_TOKEN" }
```

Grok 在为会话构建客户端时读取每个变量，并且只把值放进请求头，从不落盘。变量未设置或为空时跳过该头；解析出的值会覆盖同名的 `extra_headers` 条目。静态值用 `extra_headers`，来自环境的用 `env_http_headers`。

这两个字段也适用于共享的 `[model_providers.<id>]` 块。用 `model_provider = "<id>"` 指向提供商的模型，在自己未设置时继承提供商的 `query_params`、`env_http_headers` 和 `extra_body`，与 `extra_headers` 的继承方式相同。

### 自定义请求体字段

有些第三方网关（例如把 Codex 转成 OpenAI 兼容协议的 new-api）要求在推理请求 JSON body 里带上额外字段。`extra_body` 会把这些键合并进发往 `chat_completions` / `responses` / `messages` 的请求体——不是请求头，也不是查询字符串。

```toml
[models]
extra_body = { provider_tag = "global-default" }

[model.codex]
model = "gpt-5.1-codex"
base_url = "https://new-api.example/v1"
api_backend = "chat_completions"
env_key = "NEW_API_KEY"

[model.codex.extra_body]
enable_thinking = true
tags = ["codex", "via-new-api"]

[model.codex.extra_body.custom_params]
foo = "bar"
n = 1
```

按模型的 `[model.<id>].extra_body` **按顶层键**覆盖全局 `[models].extra_body`：模型上设置的键胜出，仅全局有的键仍由该模型继承。允许嵌套表和数组。只有请求里尚不存在的键才会被插入；保留字段（`model`、`messages`、`input`、`tools`、`stream`、`stream_options`）会被跳过，因此 `extra_body` 不能替换对话或工具载荷。

与 `extra_headers` 一样，这些字段只挂在该模型的推理调用上。模型只要自己设置了任何 `extra_body`，就不会整表继承提供商的值。

---

## 覆盖内置模型

你可以覆盖内置模型的特定字段，而不必重新定义全部。只写要改的字段：

```toml
# 只覆盖默认模型的 API 密钥
[model.grok-4.6]
api_key = "my-api-key"

# 覆盖温度并添加自定义 API 密钥
[model.grok-4.6]
temperature = 0.5
api_key = "sk-custom"
```

覆盖内置模型时，Grok 从默认配置（包括正确的 `base_url`）开始，然后只应用你指定的字段。未指定的字段从默认继承。

### 优先级

1. 你的配置（`[model.*]`）——最高优先级
2. 从远程 `/v1/models` 预取的模型
3. 硬编码默认值——最低优先级

---

## 提供商示例

### Anthropic（Claude）

通过 Anthropic Messages API 直接使用 Claude 模型：

```toml
[model.claude-opus]
model = "claude-opus-4-6"
base_url = "https://api.anthropic.com/v1"
name = "Claude Opus 4.6"
api_backend = "messages"
context_window = 200000
extra_headers = { "x-api-key" = "sk-ant-...", "anthropic-version" = "2023-06-01" }
```

`messages` 后端使用 Anthropic Messages 协议。Anthropic 用 `x-api-key` 头认证，而不是 `Authorization: Bearer`，因此通过 `extra_headers` 传递密钥，Grok 会原样发送。

### OpenAI（Chat Completions）

```toml
[model.gpt-4o]
model = "gpt-4o"
base_url = "https://api.openai.com/v1"
name = "GPT-4o"
env_key = "OPENAI_API_KEY"
```

`api_backend` 默认为 `"chat_completions"`，因此 OpenAI 不必显式设置。

### OpenAI（Responses API）

若你的提供商支持较新的 Responses API：

```toml
[model.gpt-4o-responses]
model = "gpt-4o"
base_url = "https://api.openai.com/v1"
name = "GPT-4o (Responses)"
api_backend = "responses"
env_key = "OPENAI_API_KEY"
```

### Ollama（本地模型）

用 [Ollama](https://ollama.ai) 在本地运行模型：

```toml
[model.ollama-codellama]
model = "codellama"
base_url = "http://localhost:11434/v1"
name = "CodeLlama (Ollama)"
```

确保 Ollama 正在运行（`ollama serve`）并且模型已拉取（`ollama pull codellama`）。

### Together AI

```toml
[model.together-mixtral]
model = "mistralai/Mixtral-8x7B-Instruct-v0.1"
base_url = "https://api.together.xyz/v1"
name = "Mixtral 8x7B"
env_key = "TOGETHER_API_KEY"
```

### 本地 OpenAI 兼容服务器

任何实现了 OpenAI Chat Completions 或 Responses API 的服务器：

```toml
[model.local-llama]
model = "llama-3.1-70b"
base_url = "http://localhost:8080/v1"
name = "Local Llama"
temperature = 0.8
```

---

## 自定义模型端点

把 Grok 指向自定义的 OpenAI 兼容 `/v1/models` 端点，而不是默认端点。当你的模型位于公司网关或自托管推理服务后面时使用。

### 环境变量

| 变量 | 必需 | 说明 |
|----------|----------|-------------|
| `GROK_MODELS_BASE_URL` | 是 | 推理的基础 URL。Grok 从 `{base_url}/models` 拉取模型列表。 |
| `XAI_API_KEY` | 是 | 作为 `Authorization: Bearer` 发送的 API 密钥。Grok 也接受 `GROK_CODE_XAI_API_KEY`。 |
| `GROK_MODELS_LIST_URL` | 否 | 当模型列表 URL 与 `{base_url}/models` 不同时覆盖它。 |

### 设置

```bash
export GROK_MODELS_BASE_URL="https://api.acme.com/v1"
export XAI_API_KEY="xai-..."
grok
```

### 配置文件替代

```toml
[endpoints]
models_base_url = "https://api.acme.com/v1"

# 只覆盖特定模型的 API 密钥
[model.grok-4.6]
api_key = "my-api-key"
```

使用 `[endpoints]` 配合部分模型覆盖时，Grok 从端点配置继承 `base_url`，因此不必在每个 `[model.*]` 段中指定。

### 认证行为

设置 `models_base_url` 时，Grok 使用 API 密钥认证（`Authorization: Bearer`）而不是会话认证。不需要 `grok login`——API 密钥就够了。

---

## 网络搜索模型

`web_search` 工具使用单独的模型。这样配置：

```toml
[models]
web_search = "grok-4.5"
```

或通过环境变量：

```bash
export GROK_WEB_SEARCH_MODEL="grok-4.5"
```

若把网络搜索指向自定义模型，还需要一条 `[model.*]` 条目，Grok 才能连上它。服务端（「backend」）网络搜索仅在模型设置了 `supports_backend_search = true`（且构建启用了 backend search）时运行；它不依赖 `api_backend`：

```toml
[models]
web_search = "my-custom-model"

[model.my-custom-model]
model = "my-custom-model"
supports_backend_search = true
```

---

## 使用自定义模型

```bash
# 列出可用模型（包括自定义）
grok models

# 在 TUI 中通过斜杠命令使用
/model my-model

# 在无头模式中使用
grok -p "Hello" -m my-model

# 在 config.toml 中设为默认：
[models]
default = "my-model"
```

---

## 企业部署

带自定义模型的企业部署完整配置：

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

## 故障排除

### 找不到模型

```bash
# 列出可用模型
grok models

# 检查 config.toml 里 [model.*] 段是否有拼写错误
```

### 连接错误

确认端点可达：

```bash
curl -s https://api.example.com/v1/models \
  -H "Authorization: Bearer $XAI_API_KEY"
```

### 调试日志

```bash
RUST_LOG=debug GROK_LOG_FILE=/tmp/grok.log grok
tail -f /tmp/grok.log
```

查找包含 `model` 或 `sampling` 的日志条目，以追踪模型选择和 API 调用。
