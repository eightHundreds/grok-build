# 接下来去哪

你已经够用了。想再深入时：

## 内置帮助

- **`/help`** 或 **`Ctrl+P`** — 全部命令、快捷键和技能，可搜索。
- **`/docs`** — TUI 内的完整使用指南（`/docs web` 打开在线文档）。
  覆盖会话、无头模式、子 agent、沙箱、记忆等等。
- **直接问 Grok** — 它能读自己的用户指南并自己配置。
  试试：「我怎么在 CI 里跑你？」或「加一个 GitHub 的 MCP 服务器」。

## 好习惯

- 会话会自动保存。用 `grok -c` 恢复最近一次，或用 `/resume`（`Ctrl+R`）挑选。
- 长会话变慢了？`/compact` 腾出上下文；`/context` 显示用量去向。
- 任何事都可以自动化：`grok -p "summarize new TODOs" --output-format json`
  以无头模式运行 — 适合脚本和 CI。
- 用 `grok update` 保持最新；用 `/release-notes` 看变更。
- 哪里不对劲？`/feedback` 直接发给团队。

## 再打开这份教程

随时输入 **`/tutorial`**。

去构建点什么吧。
