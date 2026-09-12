# 并行工作：工作树

想让 Grok 做功能，同时你（或另一个 Grok 会话）在同一仓库做别的事？
**Git worktree** 给每个会话独立检出 — 不会互相踩改动，也不用 stash。

## 在工作树里开会话

- **任意位置：** 按 `Ctrl+N`（按两次确认）新建会话，然后选工作树。
- **欢迎屏：** 在 git 仓库里按 `Ctrl+W` 打开「新建工作树」对话框。
- **终端：**

  ```bash
  grok --worktree=my-feature "refactor the auth module"
  ```

  （要用 `=` — 否则提示会被当成工作树名字。）

## 为什么好用

- 可以在同一仓库同时跑两三个 Grok 会话。
- 实验彼此隔离 — 改坏了也不动主检出。
- 做完后像普通 git 分支一样把改动合回去。

**`/fork`** 把当前对话复制到并行会话 —
可以加一句指向任务：`/fork try the async approach`。

同时跑多个 agent？**面板**（`/dashboard` 或 `Ctrl+\`）按状态分组显示
每个会话 — 谁在等输入、谁在干活、谁已经完成。

*深入阅读：`/docs Session Management`*
