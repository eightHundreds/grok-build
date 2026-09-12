# 后台任务与监控

Grok 可以跑长生命周期进程而不阻塞对话。本文介绍后台命令、`/loop` 命令、`monitor` 工具和调度器。

---

## 后台命令

在 `run_terminal_command` 工具上设 `background: true`，即可在后台跑命令。它立刻返回任务 ID；用 `get_command_or_subagent_output` 取输出。

### 工作原理

1. agent 以 `background: true` 调用 `run_terminal_command`。
2. 命令在后台启动。
3. agent 收到供之后引用的 `task_id`。
4. 命令完成后，对话里会出现通知。

### 获取输出

用 `get_command_or_subagent_output` 检查后台命令或子 agent。把 `task_ids` 作为列表传入（一个 id 就是单元素数组；最多 20 个）：

- 省略 `timeout_ms`，或传 `0`，得到非阻塞快照。
- 正数 `timeout_ms` 会等待完成。多个 id 会等到**全部**完成。

正数 `timeout_ms` 会被钳到 **1 小时**（`3600000` ms）。传输截止更短的宿主请设 `GROK_MAX_WAIT_BLOCK_MS`（纯毫秒；无法解析的值保持默认）。

若等待返回时子进程仍在跑，请放过它：不要杀掉或让它停止。完成后会自动唤醒父进程。只有在你需要另一份快照时才再轮询。

### 杀掉后台任务

用 `kill_command_or_subagent(task_id)` 终止正在运行的后台任务或子 agent。该工具对 shell 进程先发 SIGTERM，再发 SIGKILL，对子 agent 发送 Cancel 和 Shutdown。若任务被杀掉或已经退出，则报告成功。

### 常见用途

- **开发服务器**：启动开发服务器并继续写代码
- **测试套件**：在修问题的同时于后台跑测试
- **构建过程**：开始构建，稍后查看结果
- **长编译**：开始编译并继续做其他任务

---

## 把正在运行的任务送到后台

在交互式 TUI 里，按 `Ctrl+B` 把正在运行的前台命令送到后台。这是唯一的后台快捷键，不过在命令进行中发送新消息也会把该命令送到后台，而不是杀掉它。在这些情况下这样做：

- 命令比预期更久。
- 你想在命令跑着的时候再问 agent 别的事。
- 进程已经开始后你才意识到它会跑很久。

任务会继续跑，完成后你会收到通知。

---

## /loop 命令

`/loop` 按固定间隔反复跑一条提示。适合轮询任务、定期检查和持续监控。

### 语法

```
/loop [interval] <prompt>
```

间隔格式支持：

| 格式   | 示例    | 说明                   |
| ------ | ------- | ---------------------- |
| `Ns`   | `60s`   | 每 N 秒（最少 60）     |
| `Nm`   | `5m`    | 每 N 分钟              |
| `Nh`   | `2h`    | 每 N 小时              |
| `Nd`   | `1d`    | 每 N 天                |

### 示例

```
/loop 5m Check if the test suite passes and report any failures
/loop 2h Summarize new commits since the last check
/loop 60s Check if the dev server at localhost:3000 is responding
```

### 行为

- 创建时提示立刻触发一次，然后按指定间隔重复
- 每次触发都在分离的后台子 agent 里跑，而不是作为你对话里的一个回合。触发看不到对话，因此存下来的提示必须能独立成立；只有结果会回来
- 周期性任务 7 天后自动过期
- 同时最多 50 个已调度任务可以处于活动状态

---

## monitor 工具

`monitor` 工具从长跑脚本流式接收事件。每一行输出都会变成对话里的一条通知。`monitor` 是 `/loop` 的流式对应物：用 `/loop` 做定期检查，用 `monitor` 做实时事件流。

### 工作原理

1. 你提供一条 shell 命令（`command`）和出现在每条通知里的简短 `description`。
2. Grok 把命令的 stdout 和 stderr 合并到一个输出文件。
3. 该文件里每一行新内容都会变成投递到对话的通知。
4. 监控一直跑到命令退出或你停掉它。

### 脚本指南

- **管道里始终用 `grep --line-buffered`。** 没有它，管道缓冲会把事件推迟数分钟。
- **在轮询循环里处理瞬时失败**（`curl ... || true`）。一次失败的请求不应停掉监控。
- **使用有选择的过滤器。** 每一行都会变成消息，所以永远不要管道原始日志。
- **让轮询间隔匹配来源。** 远程 API 用 30 秒或更长以尊重限速，本地检查用 0.5 到 1 秒。
- **stdout 和 stderr 都会产生事件。** 把你不想当成事件的输出重定向掉——例如追加 `2>/dev/null`——或过滤掉。

### 示例

```bash
# Watch for errors in a log file
tail -f /var/log/app.log | grep --line-buffered "ERROR"

# Monitor file changes in a directory
inotifywait -m --format '%e %f' /watched/dir

# Poll GitHub for new PR comments
last=$(date -u +%Y-%m-%dT%H:%M:%SZ)
while true; do
  now=$(date -u +%Y-%m-%dT%H:%M:%SZ)
  gh api "repos/owner/repo/issues/123/comments?since=$last" \
    --jq '.[] | "\(.user.login): \(.body)"'
  last=$now; sleep 30
done
```

### 持久监控

对应该在会话生命周期内一直跑的监控设 `persistent: true`：

- PR 监控
- 日志跟踪
- CI 状态监视

用 `kill_command_or_subagent(task_id)` 停止持久监控。

### 流量控制

若监控产生过多事件，Grok 会自动停掉它。发生这种情况时，用更紧的过滤器重启监控。优先用 `grep --line-buffered`、`awk`，或只发出你关心的事件的包装脚本。

---

## 调度器

调度器提供创建周期性任务的更底层 API。`/loop` 是调度器上的便捷包装。

### scheduler_create

创建已调度任务：

| 参数              | 说明                                                     |
| ----------------- | -------------------------------------------------------- |
| `interval`        | 多久跑一次：`"5m"`、`"2h"`、`"1d"`、`"60s"`             |
| `prompt`          | 每次触发要执行的提示文本                                 |
| `fire_immediately`| 创建时除间隔外立刻触发（默认：`false`）                  |
| `recurring`       | 重复（默认：`true`）或只触发一次（`false`）              |
| `durable`         | 跨会话持久化（默认：`false`）                            |

每次触发都在分离的后台子 agent 里跑；没有选项把它当成对话里的一个回合。

### scheduler_list

列出所有活动的已调度任务及其 ID、提示、间隔和下次触发时间。

### scheduler_delete

按 ID 取消已调度任务。若找到并移除了任务则返回成功。

---

## 任务窗格

在交互式 TUI 里，按 `Ctrl+G` 切换任务窗格。该窗格在同一视图中列出：

- 正在运行的子 agent 及其进度
- 活动的后台任务及其状态
- Monitor 和 `/loop` 任务，各自带实时行数徽章
- 每条条目的任务 ID

要改为切换提示队列，按 `Ctrl+;`。

---

## 仍在运行状态行

每当后台工作仍在跑而 agent 看起来空闲时——回合之间，或回合阻塞在用户可打断的等待上——提示框上方会出现一条持久状态行：

```
◎ 1 command · 2 monitors · 1 loop · 1 subagent still running
```

它统计正在运行的后台命令、监控、已调度的 `/loop` 任务和后台子 agent，并在每一个结束时实时更新。其中任何一个都可以唤醒 agent 开新回合（命令和子 agent 在完成时，监控在事件时，loop 在定时器时），因此这条提示会一直待到什么都不剩。运行计数只活在这条状态行上：完成会作为单独的 "Task completed" 芯片落到 transcript 里，"Worked for" 标记保持朴素——transcript 从不重复或重述运行计数。

当一个回合在等后台工作（阻塞在 `get_command_or_subagent_output`）时，状态行会加上立刻接管输入的提示：

```
◎ 1 command still running · send a message to interrupt
```

当 agent 在等某个没有实时计数的东西（sleep，或已经结束的工作）时，同样的提示显示为 `◎ waiting · send a message to interrupt`。发送消息会打断等待并立刻跑你的消息。整个过程中 transcript 保持惯常形状：回合结束时一条 "Worked for" 标记。当完成唤醒 agent 并且它回复时，那条回复有自己的 "Worked for" 标记；agent 静默回答的唤醒在 transcript 里不留痕迹——除非失败，这时即使是静默唤醒也会出现 "Turn failed" 行，因此一条常驻指令绝不会在看不见的情况下停止执行。

---

## 用例与模式

### 开发服务器 + 编码

在后台启动开发服务器并继续写代码：

```
Start the dev server with `npm run dev` in the background, then implement the login form.
```

agent 以 `background: true` 跑开发服务器并继续写代码。服务器启动时你会看到通知。

### 持续测试监控

```
/loop 5m Run the test suite and report any new failures since the last run
```

每 5 分钟，agent 跑测试并只报告新的失败。

### 日志监控

用 `monitor` 监视特定事件：

```
Monitor the application log for ERROR and WARN entries. Use:
tail -f /var/log/app.log | grep --line-buffered -E "ERROR|WARN"
```

每条错误或警告都会作为通知出现在对话里。

### 监视 CI 流水线

```
/loop 2m Check the status of the GitHub Actions run for this PR. Report when it completes.
```

---

## 最佳实践

- **一次性长命令用 `background`**（构建、测试套件、启动服务器）
- **定期检查用 `/loop`**（CI 状态、测试运行、健康检查）
- **实时事件流用 `monitor`**（跟踪日志、监视文件）
- **延迟的一次性任务用带 `recurring: false` 的 `scheduler_create`**
- **把监控过滤器收紧**——优先用 `grep --line-buffered` 而不是原始日志流
- **不要在普通命令里用 sleep 循环来轮询**——改用带 `timeout_ms` 的 `get_command_or_subagent_output`
- **设置合理的轮询间隔**——远程 API 用 30s+ 以避免限速，本地检查可以更短
