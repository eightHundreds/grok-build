# 从 Claude、Cursor 或 Codex 过来？

别担心 — 你的设置、规则和技能都可以带过来。Grok Build
会读取其他 agent 使用的同一套项目约定，并导入其余部分。

## 自动识别

- **规则与说明** — `AGENTS.md`（Codex/OpenCode 约定）、
  `CLAUDE.md`（包括嵌套文件），以及 `.claude/rules/` 和
  `.cursor/rules/` 下的 `*.md` 规则。
- **技能与自定义命令** — `~/.claude/skills/`、`~/.claude/commands/`、
  `~/.cursor/skills/`，以及项目级对应目录。扁平的命令 `.md`
  文件在这里也会变成斜杠命令。
- **MCP 服务器** — 来自 `~/.claude.json`、`.cursor/mcp.json` 和项目
  `.mcp.json`。
- **钩子** — 来自 `.claude/settings.json`，包括 `Bash` 这类 matcher
  别名，因此大多数钩子可以原样运行。

## 一步导入

**`/import-claude`** 会扫描你的 `~/.claude` 设置 — 权限、环境
变量、MCP 服务器、钩子 — 并显示复选预览；确认后把你选中的
项写入 `.grok` 配置。随时可以再跑一遍。

## 从上次停下的地方继续

**`/resume-claude`**、**`/resume-codex`** 和 **`/resume-cursor`**
技能可以把那些工具里最近的会话接到这里继续。

## 查看发现了什么

在仓库里运行 **`grok inspect`**，可以看到 Grok 找到的每条规则、
技能和 MCP 服务器，并标出来源。每个兼容源都可以在
`[compat.claude]` / `[compat.cursor]` 配置段里开关。

还有一些别处可能没注意到的能力：`/btw` 可以在不打断当前任务的
情况下问一句旁路问题，`/rewind` 可以把对话回退到更早的回合
（文件改动保持原样）。

*深入阅读：`/docs Project Rules (AGENTS.md)`、`/docs Skills` 或 `/docs MCP Servers`*
