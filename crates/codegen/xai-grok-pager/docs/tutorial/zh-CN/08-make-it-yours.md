# 把它变成你的

## 最简单的办法：直接说

Grok 了解自己的能力，也可以配置自己。试试：

- *「给我们的预发库加上 Postgres MCP 服务器」*
- *「换成浅色主题」*
- *「给这个仓库写一份 AGENTS.md」*

如果你更想自己操作，下面每项也都有对应命令。

## 用 AGENTS.md 教 Grok 了解项目

在仓库根目录放一份 `AGENTS.md`，写上构建命令、约定和坑。
Grok 每个会话都会自动读它 — 这是杠杆最高的定制：

```markdown
# My Project
- Run tests with `pnpm test`
- Never edit files under generated/
```

## 用记忆教 Grok 事实

用 `#` 开头写提示（或用 `/remember`）保存给以后的会话：
`# the staging deploy uses eu-west`。

## 外观、按键和扩展

- **`/theme`** — 颜色主题（或 `auto` 跟随系统）；其余都在 **`/settings`**
  （或 `F2`）；想用 vim 就 **`/vim-mode`**。
- **技能**（`/skills`）— 可复用的提示包；用户可调用的技能会自动变成斜杠命令。
- **MCP 服务器**（`/mcps`）以及**插件与钩子**（`/plugins`、`/hooks`）。

先从 `AGENTS.md` 和一个主题开始；需要时再加其余。

*深入阅读：`/docs Project Rules (AGENTS.md)`、`/docs Skills` 或 `/docs MCP Servers`*
