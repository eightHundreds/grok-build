# 子 Agent 与 Personas

子 agent 是并行处理任务的独立子会话。每个子 agent 有自己的上下文窗口，因此主 agent 可以把工作（研究、实现、测试和代码审查）委派出去，而不消耗自己的上下文。子 agent 完成后向父会话报告摘要。

子 agent 默认启用。

---

## Agents 与 Personas

Agents 和 Personas 都定制行为，但作用层级不同：

| | **Agents** | **Personas** |
|---|---|---|
| **它们配置什么** | 整个会话：模型、工具、提示模式、系统提示 | 加到子 agent 提示上的行为叠加层 |
| **作用域** | 主会话或子 agent | 仅子 agent |
| **如何设置** | 启动时，或用 agent 定义（`.grok/agents/` 或 `~/.grok/agents/` 中的 `.md` 文件） | 在 `config.toml`（`[subagents.personas]`）或 `.grok/personas/` 下的 `.toml` 文件中；在子 agent 解析时应用 |
| **它们控制什么** | 模型、工具可用性、提示正文、技能 | 语气、输出格式、任务焦点，以及输入/输出契约 |
| **谁编辑** | 你——在 agents 弹窗中创建、删除或开关，或编辑文件 | 你——在配置或文件中定义自定义 Persona；捆绑的 Persona 只读 |
| **示例** | `grok-build`、`explore`、`plan` | `researcher`、`concise` |

Agent 定义会话本身。Persona 塑造子 agent 在会话中的行为。子 agent 始终以一种 agent 类型运行（例如 `general-purpose`），解析时可以在上面叠一层 Persona。

在 agents 弹窗中管理两者。用 `/config-agents`（别名 `/agents`）打开，或用 `/personas` 直接打开 Personas 标签页。弹窗有两个标签页：**Agents** 和 **Personas**。

---

## 禁用子 Agent

用环境变量或配置文件禁用子 agent：

```bash
export GROK_SUBAGENTS=0              # 环境变量
```

```toml
# ~/.grok/config.toml
[subagents]
enabled = false
```

---

## 子 Agent 如何工作

当主 agent 识别出要委派的工作时，它调用 `spawn_subagent` 工具启动子会话。子会话运行时带有：

- 自己的上下文窗口，独立于父会话
- 由其 agent 类型和可选能力模式决定的工具集
- 解析时应用的可选 Persona 指令

子会话完成时，父会话收到其输出——通常是一份摘要。

---

## 内置 Agent 类型

`spawn_subagent` 工具接受 `subagent_type` 参数，选择子会话的角色：

| 类型              | 说明                                          |
| ----------------- | ---------------------------------------------------- |
| `general-purpose` | 默认类型。适用于任何任务的全能力 agent。    |
| `explore`         | 研究 agent。搜索、读取、grep 并运行 shell 命令，但不编辑文件。用于代码库调查。 |
| `plan`            | 规划 agent。探索代码库并产出结构化实现计划；不编辑文件。 |

项目或用户定义的 agent 可以按名称添加新类型或遮蔽这些内置类型。

---

## Personas

Persona 是具名的行为叠加层。其指令作为 `<system-reminder>` 注入子 agent 的对话，塑造语气、输出格式和任务焦点，而不改变子 agent 的 agent 类型、模型或工具。

在 `config.toml` 或 `.toml` 文件中定义 Persona：

```toml
[subagents.personas.researcher]
instructions = "You are a thorough researcher. Always cite specific file paths."
description = "Deep investigator."
```

Grok Build 按优先级从这些位置发现基于文件的 Persona：

- `.grok/personas/*.toml`（项目）
- `~/.grok/personas/*.toml`（用户）
- 捆绑的 Persona 目录（最低优先级）

每个文件定义一个 Persona，文件名（不含扩展名）成为 Persona 名。内联 `config.toml` Persona 优先于文件。只发现 `.toml` 文件。

在 agents 弹窗的 Personas 标签页（`/personas`）中管理。捆绑的 Persona 只读；你定义的 Persona 可编辑。

> **注意：** Grok Build 通过子 agent 解析和角色应用 Persona，而不是通过 `spawn_subagent` 参数。主 agent 生成子会话时不传递 Persona 名。

### Persona 字段

| 字段               | 说明                                                          |
| ------------------- | ------------------------------------------------------------------- |
| `instructions`      | 作为 Persona 层应用的内联指令文本。               |
| `instructions_file` | 指令文件路径，在生成时加载并合并到 `instructions` 之后。 |
| `description`       | 显示在 Persona 目录中的简短摘要。回退到 `instructions` 的第一段。 |
| `inputs` / `outputs`| 声明的输入和输出契约（见下）。                     |
| `model`             | 使用该 Persona 时应用的模型覆盖。                    |
| `reasoning_effort`  | 使用该 Persona 时应用的推理力度。                  |
| `default_isolation` | 默认隔离模式（`none` 或 `worktree`）。                      |

### 输入/输出契约

Persona 可以声明它期望的输入和它产出的输出。父 agent 读取这些，以知道该提供什么上下文、该期望什么产物。这让你可以串联 Persona，让一个 Persona 的输出文件成为下一个 Persona 的输入：

```toml
[[subagents.personas.reviewer.inputs]]
name = "review_file"
io_type = "file"
required = true
description = "Path to the code under review"

[[subagents.personas.reviewer.outputs]]
name = "summary_file"
io_type = "file"
required = false
description = "Path to write review notes"
```

每个字段有 `name`、`io_type`（默认为 `file`）、`required` 标志和 `description`。

### Persona 解析

当 Persona 应用时，Grok Build 按此顺序解析生效的模型和推理力度，优先级从高到低：

1. 显式的生成时覆盖
2. 角色默认
3. Persona 默认
4. 父会话

隔离对前三步遵循同一顺序，但默认是 `none`（无工作树），而不是从父会话继承。

若请求了 Persona 但无法解析——找不到、没有指令，或其 `instructions_file` 不可读——生成失败。

---

## 生成子 Agent

主 agent 调用 `spawn_subagent` 工具。其参数：

| 参数         | 说明                                                       |
| ----------------- | ---------------------------------------------------------------- |
| `prompt`          | 给子 agent 的完整任务提示。                           |
| `description`     | 任务的短标签（3–5 个词）。                          |
| `subagent_type`   | 要启动的 agent 类型。默认为 `general-purpose`。         |
| `background`       | 在后台运行子 agent，并立即返回子 agent ID。默认为 `false`。 |
| `isolation`       | `none`（共享工作区，默认）或 `worktree`（隔离的 git 工作树）。 |
| `resume_from`     | 继续已完成子 agent 的对话。传入其子 agent ID。 |
| `cwd`             | 子 agent 的工作目录。与 `isolation: worktree` 互斥；设置了 `resume_from` 时忽略（恢复的子会话继承其来源的目录）。 |

在后台运行子 agent 时，稍后用 `get_command_or_subagent_output` 取回结果。

### 向活动的子 Agent 发消息

`send_subagent_message` 工具目前仅对根会话可用，并且只能针对该会话拥有的活动子 agent。其可选 `queue` 参数控制投递：

- 省略或 `false` 使用 **Steer**。若子 agent 空闲，消息成为一次受保护的排队回合。若它正在运行，消息在下一个安全点投递进当前回合。
- `true` 使用 **Queue**，保留排队回合行为：消息作为受保护回合等待，而不是进入活动回合。

---

## 能力模式

能力模式不是生成参数。子会话的工具来自其 **agent 类型** 以及任何 **角色 / 定义默认**。`general-purpose` 不受限（`all`）。内置的 `explore` 和 `plan` 类型可以读、搜索并运行 shell 命令，但不能编辑文件。

| 模式         | 读 | 写 | 执行 | 说明                                  |
| ------------ | ---- | ----- | ------- | -------------------------------------------- |
| `read-only`  | 是  | 否    | 否      | 读取、搜索和检查（也包括网络搜索和 LSP）；不能编辑文件或使用 shell。 |
| `read-write` | 是  | 是   | 否      | 读取，外加创建、编辑、删除和移动文件。无 shell。 |
| `execute`    | 是  | 否    | 是     | 读取，外加运行 shell 命令和后台任务。不能编辑文件。 |
| `all`        | 是  | 是   | 是     | 不受限的工具访问。`general-purpose` 的默认。 |

---

## 上下文继承

### resume_from

`resume_from` 参数让新的子 agent 从已完成的子 agent 停下的地方继续，适用于多阶段工作流：

1. 生成一个研究子 agent 调查问题。
2. 再生成第二个子 agent，把 `resume_from` 设为第一个子 agent 的 ID，这样它带着完整研究上下文继续。

新的子 agent 继承来源的转录、工具状态和模型；其系统提示和工具从当前 agent 定义重新渲染。来源必须已完成（未在运行）、属于当前会话，并且使用同一 agent 类型。

### MCP 继承

子 agent 默认继承父会话**已经连接**的 MCP 服务器。这包括本地 stdio/HTTP 服务器以及来自插件的 agent（例如 `my-plugin:reviewer`）。子会话用 `search_tool` / `use_tool` 发现并调用这些工具，方式与父会话相同。

用 agent frontmatter 的 `mcpInheritance` 控制继承：

| 值 | 效果 |
| ----- | ------ |
| `all`（省略时的默认） | 继承每一个父会话已连接的 MCP 服务器 |
| `none` | 不继承任何父会话 MCP 服务器 |
| `named: [server, …]` | 只继承列出的服务器名 |
| `except: [server, …]` | 继承除列出名称外的所有父会话服务器 |

示例：

```yaml
---
name: research-only
description: Read MCP tools but not internal connectors
tools: search_tool, use_tool, Read
mcpInheritance:
  except:
    - internal-tools
---
```

**插件 agent** 以同样方式继承父会话 MCP。为安全起见，它们仍然不能：

- 在 agent frontmatter 中声明自己的 `mcpServers`（忽略并警告）
- 在 agent frontmatter 中声明钩子
- 设置 `permissionMode: bypassPermissions`

插件捆绑的 MCP 服务器（插件 `.mcp.json`）在插件受信任后仍挂到**父会话/会话**上——它们不是仅属于子会话的 frontmatter 声明。见 [插件](09-plugins.md) 和 [MCP 服务器](07-mcp-servers.md)。

---

## 隔离：工作树模式

对会修改文件的任务，用 `isolation: worktree` 在隔离的 git 工作树中运行子 agent。这避免子会话的编辑与父会话冲突：

- 子 agent 在自己的工作树副本中工作。
- 其变更在你合并之前与父会话隔离。
- 子 agent 的结果包含工作树路径。

Grok Build 通过 `x.ai/git/worktree/*` 扩展方法管理工作树，包括把变更合并回主工作目录的 apply 操作。

---

## 配置

### 按类型开关与模型覆盖

禁用特定 agent 类型，或把它们路由到不同模型：

```toml
[subagents.toggle]
explore = true                       # 默认——省略则保持启用
plan = false                         # 禁用 plan 子 agent

[subagents.models]
explore = "grok-4.6"                 # 把 explore 路由到特定模型
```

按类型的模型覆盖对任何父会话都生效。没有覆盖时，子 agent 继承父会话的模型。

### 自定义角色与 Personas

定义带有自己的能力和模型默认值的自定义角色：

```toml
[subagents.roles.researcher]
description = "Deep research agent"
default_capability_mode = "read-only"
model = "grok-4.6"
prompt_file = ".grok/prompts/researcher.md"
```

用行为指令定义自定义 Persona：

```toml
[subagents.personas.concise]
instructions = "Be concise. No filler words."
# instructions_file = ".grok/personas/concise.md"  # 或从文件加载
```

Grok Build 也会从 `.grok/roles/*.toml` 发现角色，从 `.grok/personas/*.toml` 发现 Persona。内联 `config.toml` 定义优先于文件。

---

## 任务窗格（TUI）

Grok Build 在 agent 屏幕的侧栏显示正在运行和已完成的工作：

- 按 `Ctrl+G` 开关任务窗格，其中列出活动的和已完成的子 agent 及后台命令及其状态。
- 按 `Ctrl+T` 开关单独的待办窗格。

要查看可用的 agent 类型和 Persona，用 `Ctrl+P` 打开命令面板并选择 **管理 Agents**（`/config-agents`）。

子 agent 出现在任务窗格顶部，位于可折叠的「Subagents」分组中。

---

## 在 TUI 中查看子 Agent

子 agent 出现在交互式 TUI 的多个位置：

### 回看（父会话对话历史）

生成子 agent 时，会在*父会话*的回看中加入一块紧凑的生命周期块：

- `Subagent running: "do the thing" (Implementer · grok-4.6) · Thinking`
- 或对后台子 agent：`Subagent started: "..."`

运行中，该块显示从子会话回合跟踪器拉取的实时活动后缀（例如「Running: cargo test」、「Compacting」、「Retrying (2/3)」）。圆点按状态动画（或着色）。

在该块上按 **Enter**（或 Ctrl-F）打开子 agent 的完整转录。

对阻塞式子 agent，单条条目在子会话完成时更新圆点颜色。对后台的，会追加一条后续的 `Subagent completed/failed/cancelled in Xs: "..."` 块。

### 任务窗格（Ctrl+G）

如上所述——分组在「Subagents」下，带旋转指示、已用时间，以及快速终止或检查。

### 全屏带框视图（子会话转录）

当你打开一个子 agent（从回看块或任务窗格）时，父视图被替换为带边框的框架，内含子会话的完整转录：

- 框架内的标题栏：状态图标（旋转 / ✓ / ✗）、标签 + 粗体描述 + 模型、可选的「resumed」/「forked」徽章、实时活动 · 已用时间，以及 [✗] 关闭按钮。
- 子会话自己的回看、思考、工具调用和（有限的）输入区在框架内渲染。
- 子 agent 视图主要是观察性的——通常不能像对父会话那样直接向它们发送新的顶层提示。

用 `q`、`Esc` 或点击关闭按钮回到父视图。父会话的回看继续显示子 agent 的状态。

---

## 深度限制

只有顶层会话能生成子 agent。子 agent 不能生成自己的子 agent：最大嵌套深度为一。若子 agent 调用 `spawn_subagent`，该调用会以深度限制错误失败。这保持 agent 树扁平，并防止失控生成。

---

## 何时使用子 Agent

**适合的用例：**

- 研究代码库的同时，父会话继续其他工作
- 父会话实现改动的同时并行跑测试
- 提交之前审查生成的改动
- 委派彼此不依赖的独立任务

**不适合时：**

- 父会话可以直接处理的简单任务
- 需要与用户紧密来回的任务，因为子 agent 自主运行，不适合交互式交流
- 上下文准备成本超过并行收益的任务
