# 插件

插件把技能、斜杠命令、agent、钩子和 MCP 服务器打成一个可安装单元。你从市场获取插件，安装需要的那些，Grok 再加载它们带来的能力。要构建并分享自己的插件，见 [创建自己的市场](#create-your-own-marketplace)。

---

## 市场如何运作

市场是某人发布并分享的插件目录。使用市场分两步，类似添加应用商店：先添加市场才能浏览其中的插件，再选择要安装哪些。

1. **添加市场**，让 Grok 能展示它提供的内容。此时还不会安装任何东西。
2. **安装你想要的插件**，一次一个。

插件在你安装并启用之前保持关闭；插件的钩子和 MCP 服务器在你 [信任](#trust-and-security) 它之前保持不活动。

---

## 添加市场

市场源可以是 GitHub 仓库、任意主机上的 git URL，或本地文件夹。从命令行添加：

```bash
grok plugin marketplace add my-org/team-plugins                  # GitHub 简写（owner/repo）
grok plugin marketplace add https://gitlab.com/acme/plugins.git  # 任意 git 主机，需包含 https:// 和 .git
grok plugin marketplace add ./my-marketplace                     # 本地文件夹
```

用 `grok plugin marketplace list`、`grok plugin marketplace update [<name>]` 和 `grok plugin marketplace remove <url>` 列出、刷新和移除源。

也可以在配置里声明源，让它们始终存在。

### 在 config.toml 中

每个源需要一个 `name`，以及 `git` URL（可选 `branch`）或本地 `path` 二者之一：

```toml
[[marketplace.sources]]
name = "My Team Plugins"
git = "https://github.com/my-org/plugins.git"

[[marketplace.sources]]
name = "Local Dev"
path = "~/dev/my-plugins"
```

### 在 settings.json 中

把源加在 `extraKnownMarketplaces` 下，按名称作为键。每条的 `source` 是 `git`（带 `url`）、`github`（带 `repo`）或 `local`（带 `path`）之一：

```json
{
  "extraKnownMarketplaces": {
    "my-marketplace": {
      "source": { "source": "git", "url": "git@github.com:my-org/plugins.git" }
    }
  }
}
```

把该文件放在 `~/.grok/settings.json` 或 `~/.claude/settings.json`。

---

## 安装并使用插件

市场添加后，按名称安装插件。也可以直接从仓库或本地路径安装：

```bash
grok plugin install deploy-tools --trust
```

安装源接受多种形式：

- `owner/repo`（GitHub 简写）、`owner/repo@v1.0`（一个 ref）、`owner/repo@<commit-sha>`（精确提交，拉取后校验），或 `owner/repo#subdir`
- 完整 git URL（`https://github.com/user/repo.git`）或 SSH（`git@github.com:user/repo.git`）
- 本地路径（`./local-dir` 或 `/absolute/path`）

运行 `grok plugin install <source>` 且不带 `--trust` 时，Grok 会显示来源，警告安装会激活该插件的钩子、MCP 服务器和技能，然后停止。加上 `--trust` 才会继续。只从你信任的来源安装插件（见 [信任与安全](#trust-and-security)）。

插件的技能会出现在斜杠菜单里。技能名有歧义时，Grok 会显示带插件名前缀的限定形式，例如 `/deploy-tools:release`。要让新安装的插件生效，在插件标签页按 `r`，或开一个新会话。

---

## 管理插件

### 从命令行

```bash
grok plugin list [--json] [--available]   # 已安装插件（--available 需要 --json）
grok plugin uninstall <name> [--confirm] [--keep-data]   # 别名：rm、remove
grok plugin update [<name>]               # 省略名称则更新所有插件
grok plugin enable <name>
grok plugin disable <name>
grok plugin details <name>                # 显示插件的组件清单
```

### 在终端 UI 中

用 `Ctrl+L`（VS Code 系列以外）或 `/plugins`（任意终端；VS Code 系列必须用这个）打开插件弹窗。它有六个标签页：**钩子**、**插件**、**市场**、**技能**、**工作流** 和 **MCP 服务器**；用 `Tab` / `Shift+Tab` 切换。`/hooks`、`/marketplace`、`/skills`、`/workflows` 和 `/mcps` 会打开弹窗并落到对应标签页。

在 **插件** 标签页，按 `Enter` 展开插件，查看名称、版本、作用域（`cli`、`project`、`user`、`custom path`，或市场源名称）、技能、agent、钩子、MCP 服务器（插件未受信任时显示为 `blocked`）、描述和路径。然后：

| 按键 | 动作 |
|-----|--------|
| `r` | 重新加载所有插件 |
| `a` | 从 `owner/repo`、URL 或本地路径添加插件 |
| `Space` | 启用或禁用选中的插件 |
| `x` | 卸载选中的插件 |
| `f` | 按状态过滤（全部、已启用或已禁用） |
| `/` | 按名称搜索 |

在 **市场** 标签页，浏览并安装你的源中的插件：

| 按键 | 动作 |
|-----|--------|
| `i` | 安装选中的插件 |
| `d` | 卸载选中的插件 |
| `a` | 添加市场源 |
| `x` | 移除选中的源及其插件 |
| `r` | 刷新源 |
| `u` | 更新选中的插件 |

市场标签页里的组件摘要，只对发布了 [`plugin-index.json`](#add-a-catalog-optional) 编目的市场显示。破坏性操作会要求确认：按小写 `y` 确认，其他任意键（包括 `Esc`）取消。

在 **工作流** 标签页（用 `/workflows` 直接打开，或从上面的命令按 `Tab`），浏览 Grok 发现的已保存工作流：内置、项目 `.grok/workflows/`，以及用户 `~/.grok/workflows/`。每行显示工作流的名称、来源和描述；按 `Enter` 展开路径和适用场景说明，按 `r` 重新加载列表，按 `/` 搜索。这些行只能浏览——用 `/workflow <name>` 或它自己的斜杠命令来运行。

### 在配置中开关插件

在 `~/.grok/config.toml` 中设置：

```toml
[plugins]
paths = ["~/my-plugins/custom-tools"]        # 额外的插件目录
disabled = ["user/a1b2c3d4/noisy-plugin"]    # 要跳过的名称或 ID
enabled = ["project/9f8e7d6c/team-tools"]    # 要强制开启的名称或 ID
```

插件默认关闭，所以要开启就列入 `enabled`，要发现但不加载就列入 `disabled`。每条是普通插件名（来自 `grok plugin list`）或完整 ID（`<scope>/<hash>/<name>`）。

要完全隐藏插件和钩子界面，在 `~/.grok/pager.toml` 中设置 `disable_plugins = true`。

---

## 信任与安全

插件以你的权限运行，因此要像安装任何软件一样对待：只添加你信任的来源的市场，并只安装来自这些来源的插件。

已启用的插件需要信任才能加载技能、命令、钩子、MCP 服务器和 LSP 服务器。未受信任的插件 agent 仍会列出，但只有 frontmatter。Grok 自动信任 `~/.grok/plugins/` 中的插件；`.grok/plugins/` 中的项目插件需要信任。安装时加 `--trust` 即可授予：

```bash
grok plugin install <source> --trust
```

受信任插件的 `.mcp.json` 服务器会像其他 MCP 配置一样挂到会话上，子 agent 会继承它们。插件 agent（`plugin-name:agent-name`）默认使用父会话的 MCP 服务器，与 `~/.grok/agents/` 下的用户 agent 相同；用 `mcpInheritance` frontmatter 限制（见 [子 Agent](16-subagents.md#mcp-inheritance)）。为安全起见，插件 agent 的 frontmatter 不能声明 `mcpServers` 或钩子，也不能设置 `permissionMode: bypassPermissions`。

---

## 创建自己的市场

市场是一个列出一组插件的 git 仓库（或本地文件夹）。添加市场就像添加应用商店：让别人浏览你的插件，并由他们选择安装哪些。发布自己的市场，是团队或组织从一处共享技能、命令、agent、钩子和 MCP 服务器的方式。

你需要三样东西：一个 git 仓库、每个插件一个文件夹，以及一份列出它们的索引文件。

### 搭建仓库

1. **创建一个 git 仓库。** 私有仓库也可以；访问使用每个人自己的 git 凭据。
2. **把每个插件加成一个文件夹。** 插件文件夹可以包含 `skills/`、`commands/`、`agents/`、`hooks/hooks.json`、`.mcp.json`，以及可选的 `plugin.json` 清单（见 [插件包含什么](#what-a-plugin-contains)）。
3. **在 `.grok-plugin/marketplace.json` 中列出插件。** 这是 Grok 读取的索引。
4. **推送仓库。**

典型布局：

```
my-org-plugins/
  .grok-plugin/
    marketplace.json      # Grok 读取的索引（必需）
    plugin-index.json     # 可选编目，用于更丰富的浏览
  plugins/
    gdrive/
      plugin.json         # 可选清单
      skills/gdrive/SKILL.md
      .mcp.json           # 该插件添加的 MCP 服务器
```

Grok 从 `.grok-plugin/marketplace.json` 读取索引。它也接受 `.grok-plugin/plugin.json` 以及 `.claude-plugin/` 的等价物。

### 编写索引

`marketplace.json` 命名市场并列出每个插件：

```json
{
  "name": "My Org Plugins",
  "description": "Internal skills and tools",
  "owner": { "name": "Platform Team", "email": "platform@example.com" },
  "plugins": [
    {
      "name": "gdrive",
      "description": "Search and edit Google Drive, Docs, Sheets, and Slides",
      "category": "productivity",
      "source": { "type": "local", "path": "./plugins/gdrive" }
    }
  ]
}
```

每个插件的 `source` 指向其文件，有两种方式：

- **在本仓库中**：`{ "type": "local", "path": "./plugins/gdrive" }`。纯字符串 `"./plugins/gdrive"` 也可以。
- **在另一个仓库中**：`{ "source": "url", "url": "https://github.com/my-org/gdrive.git", "sha": "<full commit sha>" }`。钉死一个 `sha`，安装才能复现（当你 [要求固定版本](#require-pinned-versions) 时必需）。

可选的每插件字段：`version`、`author`、`homepage`、`tags` 和 `keywords`。

### 添加编目（可选）

`plugin-index.json` 编目让市场浏览器在任何人安装之前就能展示每个插件的技能、命令、钩子和 agent。它只用于展示，没有它安装也能工作，团队通常在 CI 里生成：

```json
{
  "version": 1,
  "plugins": {
    "gdrive": {
      "components": {
        "skills": [{ "name": "gdrive", "description": "Google Drive access" }]
      }
    }
  }
}
```

### 检查并分享

发布前用 `grok plugin validate [<path>]` 校验插件，并用 `grok plugin tag [<path>] [--push]` 按清单版本打发布标签。然后把仓库地址告诉别人。他们添加一次，再安装想要的插件：

```bash
grok plugin marketplace add my-org/my-org-plugins   # GitHub 简写、git URL 或本地路径
grok plugin install gdrive --trust
```

若要自动为所有人安装，而不是逐人操作，见 [在组织内分发](#distribute-across-an-organization)。

---

## 在组织内分发

管理员通过 grok 的 TOML 层以及可选的 Claude 策略文件控制插件、市场和 MCP 服务器：

- **`managed_config.toml` / `requirements.toml`**（以及 macOS MDM）是**原生**策略。当 grok 应对每个服务器和市场（包括用户自己配置或插件定义的）强制执行时，把允许列表、拒绝列表和固定值放在这里。`requirements.toml` / MDM 是防篡改层级；用户可写的 `~/.grok` 副本只是自我约束。
- **Claude `managed-settings.json`** 是**建议性**的。它的 MCP 和市场限制只约束**外来**对象——项目文件（`.grok/config.toml`、`.mcp.json`）、导入的 Claude 配置、CLI 覆盖，以及客户端注入的服务器。它们从不约束 grok 原生对象（用户/系统 `config.toml`、插件提供的定义、管理员固定值）。**添加**市场或安装新源始终被视为外来，因此建议性的严格列表仍会拒绝未列出的 `marketplace add` / `plugin install` 来源。

各层按**最严为准**合并：任何拒绝都生效，每个受限源都必须允许，布尔固定值只会收紧（`false` 粘住；后面的 `true` 不能解除固定）。TOML 同时接受 CamelCase 的 Claude 键和 snake_case 的 grok 键。

`grok inspect`（以及 `grok inspect --json`）显示已加载的 MCP/市场列表、`allowManagedMcpServersOnly` 是 `off` / `advisory` / `enforced`、额外的市场固定值，以及 **Enforced by policy** 下只收紧的固定值。

### 向所有人推出市场

在 `managed_config.toml` 中添加源，并打开你想要的插件：

```toml
[[marketplace.sources]]
name = "My Org Plugins"
git = "https://github.com/my-org/my-org-plugins.git"

# 插件在启用前保持关闭。列出插件名（来自 `grok plugin list`）
# 或完整 ID（`<scope>/<hash>/<name>`）。
[plugins]
enabled = ["gdrive"]
```

若要无人值守安装、无需每人一步操作，还要把插件文件放到 Grok 会自动发现并信任的位置：`~/.grok/plugins/`，或你用设备管理工具管理、并用 `[plugins].paths` 指向的目录。然后用 `[plugins].enabled` 启用它们。

托管工作区也可以不经过插件，直接把技能同步给用户。同步的技能以 `server` 作用域出现，由工作区管理；用户自己同名的技能会遮蔽同步的那份。见 [技能](08-skills.md)。

### 限制可添加的市场

列出人们可以添加的唯一 git 源。任何其他 git URL 都会被拒绝。被认可的条目是 `{ "source": "git", "url": "…" }` 和 `{ "source": "github", "repo": "owner/repo" }`（规范化为 `https://github.com/owner/repo.git`）。可选的 `ref` / `branch` 存在额外固定值上；它不是允许列表身份的一部分。严格列表里的 `local` 条目会被丢弃并给出警告；它们从不放行任何东西。

起限制作用的是键**存在**：空列表（`strict_known_marketplaces = []`）、每条都不支持的列表，或类型错误的键，都会完全锁定，拒绝每一次添加和安装，直到修好。不写这个键，市场就不受限。

在有约束力的严格列表存在时添加**本地路径**会被拒绝（路径永远匹配不上 git-URL 允许列表；失败即封锁），除非**管理员**的 `extraKnownMarketplaces` 固定值点名了那条精确路径。来自用户可写 `~/.grok` 层的固定值不能开这个例外。已有但不通过列表的 git 源会在加载时被丢弃（`Marketplace source blocked by allowlist`）。

```toml
# /etc/grok/requirements.toml  （原生：约束每一个市场）
[[strict_known_marketplaces]]
source = "git"
url = "git@github.enterprise.example:ACME/my-org-plugins.git"

[[strict_known_marketplaces]]
source = "github"
repo = "ACME/more-plugins"
```

同样的列表也适用于 Claude `managed-settings.json`（对已经配置好的 grok 原生源是建议性的）。URL 比较只对**方案和主机**折叠大小写，并恰好剥掉一个尾随 `.git`（`repo.git.git` 是另一个仓库）。用 `grok inspect` 查看已加载的允许列表。

用 `extraKnownMarketplaces` / `extra_known_marketplaces` 从策略预置额外源。第一个固定该名称的层胜出；已配置的源若占用该名称但 URL 不同，不会被覆盖（会记日志）。额外固定值上的 `autoUpdate = false` 会关闭**全局**会话启动时的插件自动更新（grok 没有按市场的等价项）。

```toml
[extra_known_marketplaces.acme]
source = { source = "git", url = "https://github.com/ACME/my-org-plugins.git", ref = "main" }
```

### 限制可运行的 MCP 服务器

Grok 从每一个原生 TOML 策略层以及 Claude `managed-settings.json`（建议性；见上）强制执行 MCP 允许/拒绝列表。`grok inspect` 打印合并后的列表。

每条允许或拒绝条目是以下之一：

| 字段 | 匹配对象 |
| --- | --- |
| `serverUrl` / `server_url` | HTTP/SSE 服务器 URL。主机和路径在两个列表上都遵循 Claude 的 `serverUrl` 规则：`*` 通配符分别匹配主机和路径（`https://*.example.com/*` 不能匹配另一主机上的相似路径）；没有路径的模式（`https://mcp.example.com`，或只有一个裸尾随 `/`）匹配该主机上的每一条路径；带路径的模式只匹配那条路径，因此用 `/mcp/*` 来限定授权。**允许条目**在方案和端口上比 Claude 更严。方案是字面值或裸 `*`（`*` 匹配受支持的远程方案 http 和 https，其他都不匹配）；没有方案的 `*.example.com/*`，或 Claude 那种部分方案 glob（如 `http*://`）永远不匹配，并在启动时记一条警告。端口保持字面（https 上显式的 `:443` 和没有端口是同一目标）；Claude 那种端口 glob（如 `http://localhost:*/*`）永远不匹配，并在启动时记一条警告——请逐个列出端口。**拒绝条目**按主机和路径跨每一种方案和端口匹配：`mcp.untrusted.example/*` 和 `http://mcp.untrusted.example:*/*` 都会在任意方案和端口上封锁该主机，且不警告。 |
| `command` | stdio 可执行文件名，与配置的命令精确匹配（不是 argv 的其余部分）。 |
| `serverCommand` / `server_command` | stdio argv，与 `[command, args…]` 精确匹配。部分数组（非字符串或空）会匹配错命令：在允许列表上它不授予任何东西；在拒绝列表上它会锁死该源（见下）。 |
| `serverName` / `server_name` | 任意传输上的配置名。比较在空格变成 `_` 之后不区分大小写；运行时名称上的 `grok_com_` 前缀会被剥掉。 |

**拒绝优先。** 匹配 `deniedMcpServers` 的服务器即使也匹配 `allowedMcpServers` 也会被封锁。若 `allowedMcpServers` 存在，每一个未列出的服务器都被封锁；存在但为空的列表（`allowed_mcp_servers = []`）会封锁该文件所约束的每一个服务器，因此不要把它当脚手架发出去。只有拒绝列表的文件封锁列出的服务器，其余不管（空拒绝列表无害）。跨层时，服务器必须通过**每一个**受限源。

**配置错误会锁死而不是失败放行。** 类型错误的策略键（该是列表却写成表或字符串）、两种拼写都写了且值不同、每条都不支持的允许列表，或无法强制执行的拒绝条目（未知字段、部分 `serverCommand`、永远匹配不上的 `serverUrl`）会锁死该文件的 MCP 策略：它约束的每一个服务器都会以原因 `locked down by policy (<file>)` 被封锁，直到文件修好。启动日志会点出文件和出错的键。不可用的**允许**条目只是不授予任何东西。

`allowManagedMcpServersOnly = true`（或 `allow_managed_mcp_servers_only`）是锁死：即使允许列表为空，也需要一次正向的允许条目匹配。原生 TOML 在 inspect 中显示为 `enforced`；仅 Claude 的显示为 `advisory`（grok 原生服务器豁免）。

`enableAllProjectMcpServers = false` 会丢掉项目作用域的 MCP，除非该服务器也匹配一条允许条目。

```toml
# /etc/grok/requirements.toml
allow_managed_mcp_servers_only = true
enable_all_project_mcp_servers = false

[[allowed_mcp_servers]]
server_url = "https://*.example.com/*"

[[allowed_mcp_servers]]
command = "npx"

[[allowed_mcp_servers]]
server_command = ["npx", "@corp/mcp"]

[[allowed_mcp_servers]]
server_name = "linear"

[[denied_mcp_servers]]
command = "node"

[[denied_mcp_servers]]
server_url = "https://mcp.untrusted.example/*"
```

这些列表在 Grok 合并用户、项目、插件和导入的 MCP 配置之后生效。被封锁的服务器会从会话中丢掉（记为 `MCP server blocked by managed settings policy`），原因是匹配了 `deniedMcpServers`、不在 `allowedMcpServers` 中、被策略锁死，或项目 MCP 固定值，再加上策略文件路径（inspect/JSON/日志保留完整路径；面向用户的拒绝只显示文件名）。

部署也可以直接把 MCP 服务器发给用户。原生允许列表仍然约束任何配置——托管的或个人的——允许运行什么。

### 关闭会话启动时的插件自动更新

`plugin_auto_update = false` / `pluginAutoUpdate = false` 只收紧。这个全局固定值是 Grok 自己的键，没有 Claude 对应项。固定后，会话启动不会扫描市场或分发每个插件的更新（没有 toast）。手动 `grok plugin update` 仍然可用。Claude 按市场的 `extraKnownMarketplaces.<name>.autoUpdate: false` 固定的是同一个全局开关。

### 要求固定版本

拒绝任何未钉到完整 commit sha 的远程插件安装或更新：

```toml
[marketplace]
require_sha = true
```

也可以设置 `GROK_MARKETPLACE_REQUIRE_SHA=1`。两者都只收紧策略；都不会把它再关掉。在市场的 `plugin-index.json` 中发布 `sha` 值，从该市场安装才能满足规则。直接内嵌在市场仓库里的插件从该仓库的检出复制，因此用同样的方式固定，在 `plugin-index.json` 里写 `sha` 值。

### 关闭插件 UI

要隐藏插件和钩子界面，在 `pager.toml` 中设置：

```toml
disable_plugins = true
```

### 本功能不覆盖的范围

市场分发的是 Grok 内容：技能、命令、agent、钩子和 MCP 服务器配置。它们不会在机器上安装一个程序。技能或 MCP 服务器若要跑辅助二进制（例如自定义登录工具），仍需单独交付该二进制，随你的部署捆绑，或通过设备管理工具推送。

---

## 故障排除

**已安装的插件没有出现。** 插件在启用前是关闭的。检查 `grok plugin list`，然后把插件名或 ID 加到 `[plugins].enabled`，或在插件标签页对它按 `Space`。在插件标签页按 `r` 重新加载，或开一个新会话。

**插件的技能、钩子或 MCP 服务器没有加载。** 它们在插件受信任之前保持不活动。用 `--trust` 重新安装，或把插件放到 `~/.grok/plugins/`（自动信任）。见 [信任与安全](#trust-and-security)。

**市场里的技能或 MCP 服务器缺失。** 用 `grok plugin marketplace update` 刷新源，确认插件已安装并启用；若组织限制来源，检查该市场是否仍被允许（见 [在组织内分发](#distribute-across-an-organization)）。有些 MCP 服务器需要登录，在你认证之前不会出现。

**MCP 服务器已配置但从不启动。** 组织策略可能封锁了它。`grok inspect` 列出 `allowedMcpServers` / `deniedMcpServers`、`mcpManagedServersOnly`、任何被锁死的策略文件，以及每个服务器的来源。拒绝匹配、未授权该服务器的允许列表 / 锁死、被锁死的策略文件，或对项目作用域服务器设置了 `enableAllProjectMcpServers = false`，都会在启动前丢掉它。见 [限制可运行的 MCP 服务器](#restrict-which-mcp-servers-can-run)。

**添加市场被拒绝。** 生效的是 `strictKnownMarketplaces` 列表。只能添加列出的 git / GitHub URL；本地路径添加会被拒绝，除非管理员的 `extraKnownMarketplaces` 固定值点名了那条精确路径。若 `grok inspect` 显示该列表被锁死，说明键存在但是空的、格式错误，或只点了不支持的源，在修好之前什么都加不了。

**安装因未固定而被拒绝。** 你的部署要求固定提交。安装精确提交（`owner/repo@<sha>`），或使用 `plugin-index.json` 发布了 `sha` 值的市场。见 [要求固定版本](#require-pinned-versions)。

**查看到底加载了什么。** 运行 `grok inspect`（加 `--json` 得到机器可读输出），列出每一个发现的插件以及它提供的技能、agent、钩子和 MCP 服务器，每项都标有 `plugin: <name>` 来源。

---

## 参考

### 插件包含什么

插件是一个目录，可以任意组合：

- **技能**：`skills/` 目录，内含 SKILL.md 文件
- **斜杠命令**：`commands/` 目录
- **Agents**：`agents/` 目录
- **钩子**：`hooks/hooks.json` 文件
- **MCP 服务器**：`.mcp.json` 文件
- **LSP 服务器**：`.lsp.json` 文件

可选的 `plugin.json` 清单可以覆盖路径或添加元数据；没有清单时，Grok 从这些标准目录发现组件。例如，`team-tools` 插件可能捆绑一个部署技能、一个代码审查 agent、提交前钩子，以及一个 Linear MCP 服务器，一步安装到位。

技能或命令可以在 SKILL.md 旁边附带**辅助脚本**（例如它调用的 Python 文件）。把脚本放进插件，让技能按相对路径运行；它会随插件复制到机器上。脚本的运行时以及它导入的任何包必须已经存在——插件交付的是文件，不是运行时或原生二进制（见 [本功能不覆盖的范围](#what-this-does-not-cover)）。

### Grok 在哪里查找插件

Grok 按优先级从这些位置发现插件。`.claude/plugins/` 的等价位置也有效；两个插件同名时，优先级更高的胜出：

| 位置 | 作用域 | 信任 |
|----------|-------|-------|
| `_meta.pluginDirs`（`session/new` / `session/load`） | 会话，仅该会话 | 自动信任 |
| `--plugin-dir`（`grok agent … stdio` 标志） | 进程，仅该 agent 进程 | 自动信任 |
| `.grok/plugins/` | 项目，通过版本控制共享 | 需要信任 |
| `~/.grok/plugins/` | 用户，每个项目 | 自动信任 |
| `[plugins].paths`（配置） | 你添加的自定义目录 | 取决于位置 |

`session/new` 和 `session/load` 请求上的 `_meta.pluginDirs` 字段为单个会话加载插件；因为调用方提供目录，这些插件自动受信任，会话结束后不持久化。`--plugin-dir` 是专用 `grok agent … stdio` 进程的进程级等价项，可重复（`grok agent --no-leader --plugin-dir A --plugin-dir B stdio`），在 leader 模式下被忽略，由共享 leader 自己发现插件。

### 插件钩子中的环境变量

插件钩子在标准钩子环境之外再收到两个变量：

| 变量 | 说明 |
|----------|-------------|
| `GROK_PLUGIN_ROOT` | 插件已安装目录的绝对路径。 |
| `GROK_PLUGIN_DATA` | 插件可写数据目录的绝对路径，用于状态、缓存和日志。 |

Grok 设置这些值，并覆盖钩子 `env` 映射中任何同名值（`CLAUDE_PLUGIN_ROOT` 和 `CLAUDE_PLUGIN_DATA` 别名也会设置）。传给钩子的全部变量见 [钩子指南](10-hooks.md)。

### 键盘快捷键

这些按键在插件弹窗的每个标签页都有效：

| 按键 | 动作 |
|-----|--------|
| `Tab` / `Shift+Tab` | 下一 / 上一标签页 |
| `j` / `k` 或方向键 | 移动选中项 |
| `Enter` | 展开或折叠选中项 |
| `/` | 按名称搜索当前标签页 |
| `Esc` | 清除搜索，或关闭弹窗 |
