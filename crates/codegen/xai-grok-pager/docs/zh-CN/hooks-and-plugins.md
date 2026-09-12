# Hooks 与插件指南

Grok Build 支持 **hooks**（事件驱动的 shell 命令）和 **plugins**（skills、agents、hooks 与 MCP 服务器的打包）。两者都通过统一的模态界面管理。

## 打开模态框

| 方式 | 打开的标签 |
|--------|-------------|
| `Ctrl+L` | Plugins（任意窗格；**非 VS Code 家族** — 在 VS Code / Cursor / Windsurf / Zed 上请用 `/plugins`） |
| `/plugins` | Plugins（任意终端） |
| `/hooks` | Hooks |

## 标签

模态框有三个标签：**Hooks**、**Plugins** 和 **Marketplace**。用 `Tab` / `→`（向前）或 `Shift+Tab` / `←`（向后）切换。

---

## Hooks 标签

Hooks 是在 `session_start`、`post_tool_use`、`notification` 等事件上自动运行的 shell 命令（或 HTTP 调用）。如何自己编写见 [创建自定义 Hooks](custom-hooks.md)。

Hooks 按来源分组：
- **全局 hooks** — 来自 `~/.grok/hooks/`
- **项目 hooks** — 来自仓库中的 `.grok/hooks/`
- **插件 hooks** — 随已安装插件打包
- **自定义 hooks** — 通过路径手动添加

每个 hook 会显示：
- 触发的 **Event**（例如 `session_start`、`post_tool_use`）
- 运行的 **Command** 或 **URL**
- **Timeout** 时长
- **Status** — 已启用或 `[disabled]`

### 快捷键（Hooks 标签）

| 键 | 操作 |
|-----|--------|
| `l` | 重新加载全部 hooks |
| `a` | 从路径添加 hook |
| `r` | 移除所选 hook |
| `e` | 启用 / 禁用所选 hook |
| `Space` | 展开 / 折叠分组 |

---

## Plugins 标签

Plugins 是包含 skills、agents、hooks 和 MCP 服务器配置任意组合的目录。

每个插件展开后会显示：
- **Name** 和 **version**
- **Scope** — `user`、`project`、`cli`，或 marketplace 来源名
- **Skills** — 名称或数量
- **Agents** — 名称或数量
- **Hooks** — 数量
- **MCP servers** — 数量（未信任时为 "blocked"）
- **Description**
- **Conflicts** — 若有则显示 ⚠ 警告

插件 hooks 会自动收到 `GROK_PLUGIN_ROOT` 和 `GROK_PLUGIN_DATA` 环境变量（见 [插件指南](../user-guide/09-plugins.md#environment-variables-in-plugin-hooks)）。

### 快捷键（Plugins 标签）

| 键 | 操作 |
|-----|--------|
| `r` | 重新加载全部插件 |
| `i` | 从路径安装插件 |
| `e` | 启用 / 禁用所选插件 |
| `Space` | 展开 / 折叠插件详情 |
| `/` | 按名称搜索插件 |

---

## Marketplace 标签

浏览并从已配置的 marketplace 来源安装插件。

来源从以下位置加载：
1. **config.toml** — `[[marketplace.sources]]` 条目
2. **settings.json** — 来自 `~/.grok/settings.json` 或 `~/.claude/settings.json` 的 `extraKnownMarketplaces`

每个来源会列出其插件，并显示：
- **Name** 和 **version**
- **Description**
- **Install status** — `[installed]`、`[installed • update: v1 → v2]`，或未安装

### 快捷键（Marketplace 标签）

| 键 | 操作 |
|-----|--------|
| `i` | 安装所选插件 |
| `d` | 卸载所选插件 |
| `r` | 刷新 marketplace 来源（重新 clone/pull git 仓库） |
| `u` | 更新所有已安装的 marketplace 插件 |
| `Space` | 展开 / 折叠来源或插件 |
| `/` | 按名称搜索插件 |

### 添加 Marketplace 来源

在 Marketplace 标签按 `a`（或运行 `grok plugin marketplace add <source>`），
传入 git URL、GitHub 简写（`owner/repo`），或本地目录路径
（`/absolute`、`~/dir` 或 `./relative`）。本地路径会存为 `path`
来源——方便从已有 checkout 开发 marketplace。

来源会写入 `~/.grok/config.toml`：

```toml
[[marketplace.sources]]
name = "My Team Plugins"
git = "https://github.com/my-org/plugins.git"

[[marketplace.sources]]
name = "Local Dev"
path = "~/dev/my-plugins"
```

或写在 `~/.grok/settings.json` / `~/.claude/settings.json`：

```json
{
  "extraKnownMarketplaces": {
    "my-marketplace": {
      "source": { "source": "git", "url": "git@github.com:my-org/plugins.git" },
      "autoUpdate": true
    }
  }
}
```

---

## 通用键盘快捷键

这些在所有标签上都有效：

| 键 | 操作 |
|-----|--------|
| `Tab` / `→` | 下一个标签 |
| `Shift+Tab` / `←` | 上一个标签 |
| `j` / `↓` | 选择下移 |
| `k` / `↑` | 选择上移 |
| `Space` | 切换展开 / 折叠 |
| `/` | 开始搜索（Plugins 与 Marketplace） |
| `Backspace` | 删除搜索字符，或重新进入搜索 |
| `Esc` | 清除搜索，或关闭模态框 |
| `q` | 关闭模态框 |

## 确认与错误

某些操作（例如卸载插件）可能会要求确认：
- 按 `y` 确认
- 按 `Esc` 或任意其他键取消

错误以消息覆盖层显示 — 按任意键关闭。

操作进行中时，模态框会显示 "Processing..." 并在完成前屏蔽输入。

## 另见

- [创建自定义 Hooks](custom-hooks.md) — 编写自己的 hooks 与脚本的逐步指南
- [Hooks 用户指南](user-guide/10-hooks.md) — 事件、matcher、信任模型
- [Hook 示例](../../../xai-grok-hooks/examples/README.md) — 可直接使用的示例 hooks
- [插件用户指南](user-guide/09-plugins.md) — 安装、信任与 marketplace
