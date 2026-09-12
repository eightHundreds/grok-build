# Agent 模式（ACP）与 IDE 集成

Agent 模式把 Grok 作为长驻服务器运行，客户端通过 [ACP](https://agentclientprotocol.com)（JSON-RPC）与它通信。从 IDE、SDK、评测框架和自定义应用使用。若只要一次性提示、打印后退出，改用 `grok -p`（[无头模式](14-headless-mode.md)）。

---

## 自动化与 SDK

对脚本、CI、评测和 agent 服务器，先从始终批准开始，这样工具运行时不会弹出交互式权限提示。拒绝规则和钩子仍然生效。

```bash
# stdio（本地进程 / 许多 SDK）
grok agent --always-approve stdio

# WebSocket 服务器
grok agent --always-approve serve --bind 127.0.0.1:2419 --secret <token>
```

也可以在 `session/new` 上按会话设置始终批准：

```json
{
  "cwd": "/path/to/project",
  "mcpServers": [],
  "_meta": { "yoloMode": true }
}
```

交互式 TUI 用户通常保留默认的询问模式（或使用 auto）。见 [权限与安全](22-permissions-and-safety.md)。

---

## 什么是 ACP？

[Agent Client Protocol（ACP）](https://agentclientprotocol.com) 定义客户端如何通过 JSON-RPC 与编码 agent 通信。在 Grok 中它覆盖：

- 会话（创建、加载、恢复）
- 提示与流式回复
- 工具调用更新
- 推理 / 思考流
- 会话不是始终批准时的权限提示

---

## stdio 传输

stdio 是常见的本地集成路径。agent 在 stdin 和 stdout 上讲 JSON-RPC：

```bash
grok agent --always-approve stdio
```

典型客户端：IDE 扩展（Zed、Neovim、Emacs）、自定义工具，以及 ACP SDK。

### 选项

Agent 选项适用于每一种传输（`stdio`、`serve`、`headless`、`leader`）。它们放在 `agent` 之后、模式名之前。模式特定标志放在模式之后（例如 `serve --bind`）。

```bash
grok agent --always-approve --model grok-4.6 stdio
grok agent --always-approve serve --bind 127.0.0.1:2419 --secret <token>
```

| 标志 | 说明 |
| ---- | ----------- |
| `-m, --model <MODEL>` | 模型 ID（例如 `grok-4.6`）。 |
| `--always-approve` | 不弹出交互式工具权限提示。别名：`--yolo`。 |
| `--reauth` | 在 agent 启动前认证。 |
| `--agent-profile <PATH>` | 从文件加载 agent 配置档。 |
| `--leader` / `--no-leader` | 连接到共享 leader 进程，或强制使用本地 agent。当请求非 `off` 的沙箱配置档时，leader 模式会被拒绝，以便工具留在进程内（见 [沙箱模式](18-sandbox.md)）。 |

---

## 服务器模式

```bash
grok agent --always-approve serve --bind 127.0.0.1:2419 --secret <token>
```

客户端通过 WebSocket 连接，并用密钥令牌认证。若省略 `--secret`，agent 会在启动时打印生成的令牌，或设置 `GROK_AGENT_SECRET`。进程在客户端重连之间保持状态。权限与其他入口一致；见 [权限与安全](22-permissions-and-safety.md)。

这是你自己运行的服务器——Grok 托管的云沙箱不运行 `grok agent serve`。

---

## WebSocket 中继

要在互联网上到达 agent，把 agent 连到中继，并把浏览器指向同一中继：

```bash
grok agent --always-approve headless --grok-ws-url wss://your-relay.example.com/ws
```

---

## ACP 协议基础

通信遵循 JSON-RPC 2.0 格式。典型会话生命周期：

1. **初始化** —— 客户端发送带能力的 `initialize`
2. **创建会话** —— 客户端发送带工作目录的 `session/new`
3. **发送提示** —— 客户端发送带用户消息的 `session/prompt`
4. **接收更新** —— agent 发送带流式内容的 `session/update` 通知
5. **处理权限** —— agent 可能请求工具执行批准（或根据权限模式允许或拒绝）

### 架构

```
+------------------------------------------+
|           ACP Client                     |
|  (IDE, Editor, Custom Application)       |
+-------------------+----------------------+
                    | JSON-RPC over stdio
+-------------------v----------------------+
|           grok agent stdio               |
|                                          |
|  +---------+  +---------+  +---------+   |
|  | Session |  |  Tools  |  |   MCP   |   |
|  | Manager |  | Registry|  | Servers |   |
|  +---------+  +---------+  +---------+   |
+------------------------------------------+
```

---

## 流式更新

ACP 流式发送结构化事件。每条 `session/update` 通知带一个 `sessionUpdate` 字段，标识更新类型：

| `sessionUpdate` 值 | 说明                                            |
| --------------------- | ----------------------------------------------------- |
| `agent_message_chunk` | agent 响应文本的一块。                 |
| `agent_thought_chunk` | agent 内部推理的一块。            |
| `tool_call`           | 一次新的工具调用（title、kind、status、input）。   |
| `tool_call_update`    | 进行中工具调用的状态或结果更新。 |
| `plan`                | agent 的执行计划。                           |

每次更新都点明类型，因此客户端可以为推理、工具调用和响应文本渲染不同面板。

---

## 扩展方法

在基础 ACP 协议之外，Grok 在 `x.ai/` 前缀下定义扩展方法，用于 SpaceXAI 特定功能。它们覆盖：

| 类别                   | 前缀               | 示例                                         |
| -------------------------- | -------------------- | ------------------------------------------------ |
| **文件系统**             | `x.ai/fs/*`          | `list`、`exists`、`read_file`、`write_file`      |
| **Git**                    | `x.ai/git/*`         | `status`、`stage`、`commit`、`diffs`、`discard`  |
| **Git 工作树**           | `x.ai/git/worktree/*`| `create`、`remove`、`apply`、`list`、`gc`        |
| **搜索**                 | `x.ai/search/*`      | `fuzzy/open`、`fuzzy/change`、`content`          |
| **终端**               | `x.ai/terminal/*`    | `create`、`kill`、`output`、`wait_for_exit`      |
| **会话管理**     | `x.ai/session/*`     | `fork`、`resolve_local_for_worktree_resume`      |
| **对话与历史** | `x.ai/*`             | `prompt_history`、`rewind/*`、`compact_conversation` |
| **认证**         | `x.ai/auth/*`        | `get_url`、`submit_code`                         |
| **反馈与遥测**   | `x.ai/*`             | `feedback`、`telemetry/*`                        |

这里的表格展示各类别的代表性方法。`x.ai/*` 集合是 SpaceXAI 特定的，可能随版本扩展，因此把它视为非穷尽，并从 agent 的 `initialize` 响应发现可用方法。

### 通知（agent 到客户端）

agent 向客户端发送推送通知，用于实时更新：

| 通知               | 说明                          |
| -------------------------- | ------------------------------------ |
| `x.ai/search/fuzzy/status` | 模糊搜索结果更新          |
| `x.ai/git/worktree/status` | 工作树创建进度           |
| `x.ai/fs_notify`           | 文件系统变更通知       |
| `x.ai/fs/index`            | 完整文件索引更新               |
| `x.ai/fs/index/delta`      | 增量文件索引更新        |
| `x.ai/session_notification`| 会话特定更新（diff 审阅、重试状态、自动压缩） |
| `x.ai/session/update`      | 会话更新（工具调用、内容） |

---

## 会话配置选项

`session/new` 和 `session/load` 的响应包含类型化的 `configOptions` 列表（标准 ACP，不是 `x.ai/` 扩展）。用 `session/set_config_option` 更改实时选项。

| `configId` | 类别 | 效果 |
|------------|----------|--------|
| `model` | `model` | 切换会话模型（`allowed_models`、聊天网关路由）。值必须是字符串 id。 |
| `reasoning_effort` | `thought_level` | 在不更换模型的情况下对当前模型应用力度（不改写提示，不经过 `allowed_models` 门闩）。值必须是字符串 id（`minimal`、`low`、`medium`、`high`、`xhigh`）。当模型未宣传 `supportsReasoningEffort` 时丢弃并警告。 |

```json
{
  "sessionId": "…",
  "configId": "reasoning_effort",
  "value": { "value": "high" }
}
```

响应是**完整、已更新**的选项列表。`config_option_update` 会话通知会把它镜像给每一个已订阅的客户端。在 leader 模式下，代理会窥探 `configId: model`，以便每个客户端的 `default_model` 保持同步。布尔值会被拒绝；暴露布尔选项尚未实现。

---

## 会话 `_meta` 选项

`session/new` 上的可选字段：

| 字段 | 说明 |
| ----- | ----------- |
| `rules` | 追加到系统提示的额外规则。 |
| `systemPromptOverride` | 替换系统提示。 |
| `agentProfile` | agent 配置档名称或 JSON 对象。 |
| `yoloMode` | 为 `true` 时，本会话始终批准。 |
| `autoMode` | 为 `true` 时，本会话使用 auto 权限模式。已开启始终批准时被覆盖。 |

```json
{
  "cwd": "/path/to/project",
  "mcpServers": [],
  "_meta": { "yoloMode": true }
}
```

---

## ACP SDK

多种语言提供官方 SDK 库：

| 语言   | 包                                                                                  |
| ---------- | ---------------------------------------------------------------------------------------- |
| TypeScript | [`@agentclientprotocol/sdk`](https://www.npmjs.com/package/@agentclientprotocol/sdk)     |
| Rust       | [`agent-client-protocol`](https://crates.io/crates/agent-client-protocol)                |
| Python     | [`agent-client-protocol-python`](https://github.com/PsiACE/agent-client-protocol-python) |
| Go         | [`acp-go-sdk`](https://github.com/coder/acp-go-sdk)                                     |
| Kotlin     | [`acp`](https://github.com/agentclientprotocol/kotlin-sdk)                               |

---

## 兼容客户端

| 客户端                                                   | 状态      |
| -------------------------------------------------------- | ----------- |
| [Zed](https://zed.dev/docs/ai/external-agents)           | 已支持   |
| [Neovim](https://neovim.io)（CodeCompanion、avante.nvim） | 已支持   |
| [Emacs](https://github.com/xenodium/agent-shell)         | 已支持   |
| [marimo notebook](https://github.com/marimo-team/marimo) | 已支持   |
| JetBrains                                                | 即将推出 |

---

## 集成示例：TypeScript ACP 客户端

```typescript
import { spawn, ChildProcess } from "child_process";
import * as readline from "readline";

class GrokACPChat {
  private proc!: ChildProcess;
  private sessionId!: string;
  private rl!: readline.Interface;

  constructor(private cwd = ".") {}

  async init() {
    this.proc = spawn("grok", ["agent", "--always-approve", "stdio"]);
    this.rl = readline.createInterface({ input: this.proc.stdout! });

    await this.request("initialize", {
      protocolVersion: 1,
      clientCapabilities: {
        fs: { readTextFile: true, writeTextFile: true },
        terminal: true,
      },
    });

    const { sessionId } = await this.request("session/new", {
      cwd: this.cwd,
      mcpServers: [],
      _meta: { yoloMode: true },
    });
    this.sessionId = sessionId;
    return this;
  }

  private async request(method: string, params: any): Promise<any> {
    return new Promise((resolve) => {
      const msg = JSON.stringify({ jsonrpc: "2.0", id: 1, method, params });
      this.proc.stdin!.write(msg + "\n");

      this.rl.once("line", (line) => {
        resolve(JSON.parse(line).result || {});
      });
    });
  }

  async *streamPrompt(text: string) {
    const msg = JSON.stringify({
      jsonrpc: "2.0",
      id: 1,
      method: "session/prompt",
      params: {
        sessionId: this.sessionId,
        prompt: [{ type: "text", text }],
      },
    });
    this.proc.stdin!.write(msg + "\n");

    for await (const line of this.rl) {
      const data = JSON.parse(line);

      if (data.method === "session/update") {
        const update = data.params.update;
        yield update; // { sessionUpdate, content, title, ... }
      } else if (data.result) {
        break; // Final response
      }
    }
  }
}

// Usage
const client = await new GrokACPChat(".").init();

for await (const update of client.streamPrompt("List the files in this project")) {
  switch (update.sessionUpdate) {
    case "agent_message_chunk":
      process.stdout.write(update.content?.text || "");
      break;
    case "agent_thought_chunk":
      console.log(`\n[Thinking: ${update.content?.text}]`);
      break;
    case "tool_call":
      console.log(`\n[Tool: ${update.title}]`);
      break;
  }
}
```

---

## 资源

- [ACP 规范](https://agentclientprotocol.com/protocol/prompt-turn)
- [协议介绍](https://agentclientprotocol.com/overview/introduction)
