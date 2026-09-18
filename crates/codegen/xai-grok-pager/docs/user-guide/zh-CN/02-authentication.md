# 认证

Grok 支持多种认证方式，包括交互式浏览器登录、企业单点登录（SSO），以及无界面 CI/CD runner。

---

## 浏览器登录（默认）

首次启动时，Grok 会打开浏览器，让你在 grok.com 上认证：

```bash
grok
```

Grok 把凭据存到 `~/.grok/auth.json`，跨会话复用。Grok 会在后台自动刷新 access token。token 无法刷新时，Grok 会再提示你登录。没有服务器提供过期时间的凭据，会回退为 30 天有效期。

### 凭据存储

`~/.grok/auth.json` 中的 token（以及 `~/.grok/mcp_credentials.json` 中的 MCP OAuth token）以仅所有者可读写的权限写入（Unix 上是 `0600`）。任何能访问这些路径的人都能使用凭据，因此：

- 优先开启全盘加密（FileVault、BitLocker、LUKS 或同类方案）。
- 不要把 `auth.json` 或 `mcp_credentials.json` 复制到共享目录、工单或聊天里。
- 在多用户主机上，把 `$HOME` / `$GROK_HOME` 保持为仅本账户私有。

### 重新认证

要切换账户或解决认证问题，运行：

```bash
grok login
```

运行 `grok login` 会再次启动登录流程，并替换缓存的会话。默认打开浏览器，通过 `auth.x.ai` 上的 SpaceXAI OAuth 登录。传入标志可选择其他流程：

| 标志 | 说明 |
|------|-------------|
| `--oauth` | 通过 `auth.x.ai` 上的 SpaceXAI OAuth 登录。这是默认值，因此该标志可选。 |
| `--device-auth`（别名 `--device-code`） | 用设备码流程登录，适合无界面或远程环境。 |

要登出，运行 `grok logout`。它不接受标志，并清除缓存的凭据。

---

## API Key

对于 CI/CD、自动化，或没有浏览器的环境，使用来自 [console.x.ai](https://console.x.ai) 的 API key：

```bash
export XAI_API_KEY="xai-..."
grok
```

没有活动的会话 token 时，Grok 会把 API key 当作回退。若你已经交互式登录过，已存储的会话 token 优先。要回退到 API key，运行 `grok logout` 或删除 `~/.grok/auth.json`。

---

## OIDC（客户 SSO）

通过你自己的身份提供方（IdP）——例如 Okta、Azure AD 或 Auth0——认证开发者，而不是 grok.com。

### 1. 在 IdP 中注册公共客户端

- 授权类型：带 PKCE（Proof Key for Code Exchange）的 Authorization Code
- Redirect URI：`http://127.0.0.1/callback` —— 回环地址。Grok 在登录时绑定随机端口，大多数 IdP 按 [RFC 8252](https://tools.ietf.org/html/rfc8252) 把回环重定向视为与端口无关。
- 不要客户端密钥。PKCE 替代它。

### 2. 配置 CLI

通过配置文件：

```toml
# ~/.grok/config.toml
[grok_com_config.oidc]
issuer = "https://acme.okta.com"
client_id = "0oa1b2c3d4e5f6g7h8i9"
```

或通过环境变量：

```bash
export GROK_OIDC_ISSUER="https://acme.okta.com"
export GROK_OIDC_CLIENT_ID="0oa1b2c3d4e5f6g7h8i9"
```

也可以覆盖 API 端点，指向你自己的代理：

```bash
export GROK_CLI_CHAT_PROXY_BASE_URL="https://grok-proxy.acme.com/v1"
```

### 3. 运行 `grok`

CLI 通过 `{issuer}/.well-known/openid-configuration` 发现端点，打开 IdP 登录页，并把 token 存到 `~/.grok/auth.json`。token 会通过已存储的 `refresh_token` 静默自动刷新。

### 可选字段

| 字段 | 默认 | 备注 |
|-------|---------|-------|
| `scopes` | `["openid", "profile", "email", "offline_access", "api:access"]` | `offline_access` 启用静默 token 刷新 |
| `audience` | 无 | 部分 IdP（例如 Auth0）要求 |

---

## 外部认证提供方

当无法用浏览器登录时——例如沙箱 VM、CI runner 或隔离网络——把认证委托给外部二进制或脚本。

### 工作方式

```
+--------------+     sh -c     +------------------------+
|     Grok     |-------------->|  your auth binary      |
|              |               |                        |
|  reads       |<-- stdout ----|  prints token          |
|  auth.json   |               |                        |
|              |   (stderr)    |  prints status/URLs    |--> surfaced to user
+--------------+               +------------------------+
```

1. Grok 通过 `sh -c "<command>"` 运行你的命令
2. 你的二进制执行它需要的认证流程（SSO、设备码、证书交换）
3. **stderr** 承载人类可读输出，例如登录 URL 和状态消息。Grok 读取 stderr 并展示给用户；在 TUI 里，它会把第一个 `https://` URL 变成可点击的登录链接。
4. **stdout** 由 Grok 捕获，并保存为 access token
5. 退出码 0 = 成功；非零退出 = Grok 回退到交互式登录

### stdout / stderr 约定

| 流 | 应打印什么 | 谁会看到 |
|--------|---------------|-------------|
| **stdout** | token —— 不要打印别的 | Grok（解析并存入 auth.json） |
| **stderr** | 登录 URL、状态消息、错误 | 用户（Grok 读取 stderr，并在 TUI 里把登录 URL 显示为可点击链接） |

**除 token 外，不要向 stdout 打印任何内容。** 不要进度消息，不要调试输出。Grok 读取 stdout，去掉首尾空白，并把结果解析为 token。

### stdout Token 格式

**裸字符串** —— 就是原始 token：

```
eyJhbGciOiJSUzI1NiIs...
```

**JSON** —— 可带可选的 refresh token、过期时间和 issuer：

```json
{"access_token": "eyJhbGciOi...", "refresh_token": "ref-tok", "expires_in": 3600, "issuer": "https://idp.example.com"}
```

若 token 会过期，并且希望 Grok 在过期前自动重跑该二进制，请使用 JSON。

JSON 字段：

| 字段 | 必需 | 含义 |
|-------|----------|---------|
| `access_token` | 是 | Grok 发给 xAI API 的 Bearer token |
| `refresh_token` | 否 | 仅作参考存储。Grok 通过重跑你的二进制刷新，而不是用 OAuth refresh grant |
| `expires_in` | 否 | token 寿命（秒）；用于在过期前主动刷新 |
| `issuer` | 否 | 标识 token 的签发方 |

### 配置

通过配置文件：

```toml
# ~/.grok/config.toml
[auth]
auth_provider_command = "/usr/local/bin/my-auth-provider"
auth_provider_label = "Acme Corp"   # 可选 —— 自定义 TUI 登录按钮
auth_token_ttl = 3600               # 可选 —— token 寿命（秒）
```

或通过环境变量：

```bash
export GROK_AUTH_PROVIDER_COMMAND="/usr/local/bin/my-auth-provider"
export GROK_AUTH_PROVIDER_LABEL="Acme Corp"
export GROK_AUTH_TOKEN_TTL=3600
```

### Token 刷新

Grok 按两套不同约定运行你的二进制，并用 `GROK_AUTH_EXPIRED` 区分它们。每次运行都会完整替换已存储的凭据，因此每次调用（包括刷新）都要输出相同的 JSON 字段（例如 `issuer`）。

- **`GROK_AUTH_EXPIRED=1` — 无界面刷新。** Grok 正在对已持有的凭据重新签发：临近过期的轮换，或服务器拒绝的 token。没人在看。stdin 已关闭，stderr 会被吞掉，二进制只有几秒就会被杀掉。请静默签发，或非零退出——绝不要阻塞。
- **未设置 — 登录。** `grok login`、登录屏，或无界面运行签发出错后 Grok 的升级流程。用户在等待，stderr 会到达他们，你有 300 秒——足够一次浏览器往返或设备码。

```bash
#!/bin/sh
if [ "$GROK_AUTH_EXPIRED" = "1" ]; then
    # 无界面：只做静默刷新。当 SSO 会话已失效、只有用户能续期时，
    # 拒绝是最快、也最正确的做法。
    echo "Refreshing token..." >&2
    TOKEN=$(my-company-auth --refresh --silent) || exit 1
else
    echo "Authenticating via Acme Corp SSO..." >&2
    TOKEN=$(my-company-auth --login --interactive)
fi

if [ -z "$TOKEN" ]; then
    echo "Authentication failed" >&2
    exit 1
fi

echo "{\"access_token\": \"$TOKEN\", \"expires_in\": 3600}"
```

无界面运行签不出 token 时，Grok 不再把已存凭据当作可用，并改走登录流程——和从未登录过的机器一样，会展示你二进制的 stderr，因此设备码 URL 或浏览器提示能到达你。在 `GROK_AUTH_EXPIRED=1` 时尽快退出，交接才会快；若二进制反而阻塞，每次启动都要等刷新超时。会话中途，该回合会失败并弹出重新认证提示，`/login` 会以交互方式重跑该二进制。

有一种情况仍然含糊，且仅出现在 **leader 模式**（`--leader`，或 `[cli] use_leader = true`；默认关闭）：完全没有凭据时，leader 会在刚启动后于后台多试一次，那次变量未设置，像登录一样。能自己签发的二进制（服务账户、keytab、已挂载 token）会成功，会话自行恢复。必须提示用户的二进制只会空等，直到 300 秒登录上限——没有东西在等它，登录屏已经亮着，那次运行的 stderr 会进 `~/.grok/leader.log` 而不是给你。

### 环境变量

| 变量 | 说明 |
|----------|-------------|
| `GROK_AUTH_PROVIDER_COMMAND` | 你的认证二进制路径 |
| `GROK_AUTH_PROVIDER_LABEL` | TUI 登录屏上的显示名（例如 "Acme Corp"） |
| `GROK_AUTH_TOKEN_TTL` | token 寿命（秒）（用于没有 `expires_in` 的裸字符串 token） |
| `GROK_AUTH_EXPIRED` | 无界面刷新时设为 `1`：不要提示，也不要交回缓存 token。登录时未设置，此时有用户在场 |
| `GROK_AUTH_EARLY_INVALIDATION_SECS` | 过期前多少秒主动刷新（默认：300） |

---

## 设备码流程

适用于本地没有浏览器的无界面环境（SSH 会话、Docker 容器、远程 VM）：

```bash
grok login --device-auth    # 或：grok login --device-code
```

这会在终端打印 URL 和代码。在任意设备打开该 URL，输入代码并完成认证。Grok 会轮询直到登录确认。

你也可以通过 [外部认证提供方](#external-auth-provider) 实现设备码流程，以获得完整控制。

---

## 自动凭据刷新

Grok 会自动刷新过期凭据：

- **过期前：** 若认证提供方返回了 `expires_in`（JSON 输出），或你设置了 `auth_token_ttl`，Grok 会在过期前约 5 分钟重跑认证二进制。
- **认证错误时：** 若服务器返回 401 Unauthorized，Grok 会刷新凭据并重试请求。
- **OIDC：** 若有 `refresh_token`，Grok 会通过你的 IdP 静默刷新，不再打开浏览器。

调整刷新缓冲：

```bash
# 过期前 5 分钟刷新（默认）
export GROK_AUTH_EARLY_INVALIDATION_SECS=300

# 关闭主动缓冲：到过期或遇到 401 再刷新（设为 0）
export GROK_AUTH_EARLY_INVALIDATION_SECS=0
```

---

## 热重载

Grok 会自动拾取 `~/.grok/auth.json` 的变更。若你在外部更新凭据（例如脚本写入新 token），Grok 会在下一次 API 调用时使用新凭据，无需重启。

---

## 认证优先级

Grok 按以下顺序解析每次请求的凭据，从高到低：

1. **按模型的 `api_key`、`env_key` 或钥匙串** —— 在 `config.toml` 的 `[model.<name>]` 下设置。非空 `api_key` 优先，然后是第一个已设置的 `env_key`，再然后是系统钥匙串项（`keychain_account`，service 默认为 `grok`）。钥匙串里没有该项，等同于环境变量未设置：该来源不产生密钥（不会用空凭据去发请求）。
2. **活动会话 token** —— 通过浏览器、OIDC/OAuth2 或外部提供方登录获得，并存在 `~/.grok/auth.json`。
3. **`XAI_API_KEY`** —— 没有活动会话 token 时的回退。

### 系统钥匙串（macOS 钥匙串、Windows 凭据管理器）

把密钥存进系统凭据库，再在 `config.toml` 里引用该项。Grok 只在发请求时读取，不会把密钥写回磁盘或打进日志。

```toml
# ~/.grok/config.toml
[model.codex]
base_url = "https://new-api.example/v1"
keychain_account = "new-api"
```

在 macOS 上写入。`-s grok` 必须与默认 service 一致；`-a` 是 `keychain_account`：

```bash
security add-generic-password -a "new-api" -s "grok" -w
# 或非交互：
security add-generic-password -a "new-api" -s "grok" -w "sk-..."
```

`keychain_service` 可选，默认是 `grok`，不建议改。如果改了，`-s` 必须与覆盖值一致。Windows 上创建目标/服务名为 `grok`、用户名为 `new-api` 的泛型凭据。本构建未链接 Linux Secret Service（需要 libdbus）；在 Linux 上配置钥匙串 account 不会得到密钥，等同于未设置的 `env_key`。

项缺失或为空时，Grok 把该模型视为没有钥匙串凭据（与未设置的 `env_key` 相同），并继续往后回退。只有 `api_key` / `env_key` / 钥匙串都解析不到时，具名的 `[auth_provider.<name>]` 助手才会运行。

配置了多种登录流程时，Grok 按从高到低的第一个可用来源填充会话 token：

1. **外部认证提供方**（`auth_provider_command`）
2. **企业 OIDC** —— 配置了 OIDC 时，通过 `config.toml` 的 `[grok_com_config.oidc]`，或 `GROK_OIDC_ISSUER` 和 `GROK_OIDC_CLIENT_ID` 环境变量
3. **SpaceXAI OAuth2 浏览器登录** —— 默认

会话期间，活动方法负责所有会话中途的刷新。

---

## Grove Git 凭据（不是本页的 `grok login`）

本页的一切都是让 Grok 认证到模型 API。Grove 挂载背后的 Git 远程——[`grok clone`](27-grok-clone.md) 拉取的对象——是另一个世界，归 Grove daemon 所有。

**`~/.grok/auth.json` 从不用于 Git。** `grok login` 不会创建 Git 凭据，`grok logout` 也不会吊销；daemon 根据 Grove 配置里的 `auth_mode` 建立自己的凭据单元。这些凭据用 `grove status` 和 `grove reload-credentials` 管理——失败类别及下一步见 [grok clone](27-grok-clone.md#authentication)。

---

## 相关设置

编码数据共享——设置里的 **Coding data, retention, and training**，
由 `/privacy` 打开——不会改这些配置旋钮：

| 设置 | 如何设置 |
|---------|---------------|
| `[features] telemetry` | `config.toml` 或 `GROK_TELEMETRY_ENABLED` |
| `[telemetry] trace_upload` | `config.toml` 或 `GROK_TELEMETRY_TRACE_UPLOAD` |
| 外部 OpenTelemetry | `GROK_EXTERNAL_OTEL` / `[telemetry] otel_*`。见 [用量监控](24-monitoring-usage.md)。 |

在团队账户上，只有团队管理员能改编码数据共享。
团队管理员也可以为团队启用或关闭 Zero Data Retention (ZDR)。
见 [How to enable ZDR](https://docs.x.ai/developers/faq/security#how-to-enable-zdr)。
开启 ZDR 后，编码数据共享完全不能改——设置
行会用 `ZDR` 代替取值。ZDR 不会关掉外部 OTEL
或 `user.email`——见 [ZDR and this stream](24-monitoring-usage.md#zdr-and-this-stream)。

见 [用量监控](24-monitoring-usage.md#related-settings) 和 [配置](05-configuration.md#telemetry)。

---

## 排障

### 调试日志

设置 `RUST_LOG` 可控制文件日志和无界面 stderr 输出的详细程度。（TUI 屏幕上的 tracing 面板使用固定过滤器，忽略 `RUST_LOG`。）在 TUI 里，文件日志默认是 `DEBUG`；无界面模式（`-p`）下，`RUST_LOG` 默认是 `off`，因此只打印答案——设 `RUST_LOG=error`（或更宽）才能在 stderr 看到日志。

在 TUI 里，把 `GROK_LOG_FILE` 设为绝对路径，即可把日志写到该文件：

```bash
GROK_LOG_FILE=/tmp/grok.log RUST_LOG=debug grok
tail -f /tmp/grok.log
```

`GROK_LOG_FILE` 被当作字面文件路径。相对值例如 `1` 会在当前目录写出名为 `1` 的文件。

无界面模式下，日志走 stderr。重定向到文件：

```bash
RUST_LOG=debug grok -p "hello" 2> /tmp/grok.log
```

### 常见日志消息

| 日志消息 | 含义 |
|-------------|---------------|
| `auth: running external auth provider (headless refresh)` / `(interactive login)` | Grok 正在运行你的二进制，以及按哪套约定 |
| `auth: external auth provider returned fresh token` | Grok 已解析并存储 token |
| `auth: external auth provider failed` | 二进制非零退出，或 stdout 为空 |
| `auth: external auth provider timed out (likely needs interactive auth), killing` | 二进制在超时前未退出，已被杀掉 |
| `auth: failed to start external auth provider` | 无法启动命令（找不到二进制） |

### 常见修复

- **"Authentication failed"** —— 运行 `grok logout` 清除缓存凭据，再运行 `grok login` 重新登录。
- **Token 过期太快** —— 设置 `auth_token_ttl`，或在认证提供方的 JSON 输出中返回 `expires_in`。
- **OIDC 重定向失败** —— 确保 IdP 允许回环 redirect URI（`http://127.0.0.1/callback`）。
- **找不到外部认证提供方** —— 检查 `auth_provider_command` 路径是否正确，以及二进制是否可执行。
