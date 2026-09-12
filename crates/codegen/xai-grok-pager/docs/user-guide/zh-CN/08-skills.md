# 技能

技能是可复用的提示包，用特定任务的说明扩展 Grok。它们让你把可重复的流程记录一次，而不必每个会话重新解释。启动发现会跳过不受信任文件夹中的项目技能和命令。

---

## 什么是技能？

技能是包含 `SKILL.md` 文件的目录。其 markdown 正文告诉 Grok 如何处理某一类任务：逐步说明、约定和工具使用模式。

把技能用于对 AGENTS.md 太具体、又太长不便重打的可重复流程。Grok 仅在技能适用于当前任务时激活它。

---

## 技能位置

Grok 按优先级从这些目录发现技能：

| 位置 | 范围 | 优先级 | 备注 |
|----------|-------|----------|-------|
| `./.grok/skills/`、`./.grok/commands/` | 本地（CWD） | 最高 | 当前目录技能 / 遗留命令 markdown |
| `<repo_root>/.grok/skills/`、`…/commands/` | 仓库 | 中 | 整个仓库共享 |
| `~/.grok/skills/`、`~/.grok/commands/` | 用户 | 最低 | 适用于所有项目的个人技能 |
| `~/.claude/skills/`、`~/.claude/commands/` | 用户 | 最低 | Claude Code 兼容（可配置） |
| `./.claude/skills/`、`./.claude/commands/` | 本地 / 仓库 | 高 | 项目 Claude 技能和遗留自定义斜杠命令 |
| `~/.cursor/skills/` | 用户 | 最低 | Cursor 兼容（可配置） |
| `./.cursor/skills/` | 本地 / 仓库 | 高 | 项目 Cursor 技能（启用 cursor 兼容技能时） |

Grok 按名称去重技能——更高优先级的位置覆盖更低的。Grok 也会在每一档扫描 `.agents/skills/`（以及 `commands/`）（与 `.grok/` 并列），并走遍工作目录到仓库根之间的每个目录。

`commands/` 目录下扁平的 `*.md` 文件会变成用户可调用的斜杠命令（文件名主干 = 命令名），与 Claude Code 的遗留自定义命令布局一致。

技能和命令发现**不**使用 `.gitignore`。已知技能根（`.grok/`、`.agents/`、`.claude/`、`.cursor/`）下的路径只要在磁盘上存在就会加载——团队常常把 `.claude/**` 当作仅本地配置忽略，同时仍期望 `/frontend` 这类项目命令能用。要隐藏技能，在配置里用 `[skills] ignore`（不是仓库忽略规则）。

Grok 默认扫描 Claude 和 Cursor 技能目录。要停止扫描某个厂商，在 `~/.grok/config.toml` 的 `[compat.cursor]` 或 `[compat.claude]` 下把它的 `skills` 单元设为 `false`，或把 `GROK_CURSOR_SKILLS_ENABLED` 或 `GROK_CLAUDE_SKILLS_ENABLED` 环境变量设为 `false`。详见 [配置](05-configuration.md#harness-compatibility)。无论这些设置如何，Grok 始终过滤掉已知的厂商自带默认技能（例如 Cursor 的 `shell`、`canvas` 和 `statusline`）。

### 额外技能目录

通过 `~/.grok/config.toml` 中的 `[skills]` 添加目录、排除路径或禁用单个技能：

```toml
[skills]
paths = ["~/my-team-skills"]          # 额外要扫描的目录
ignore = ["~/my-team-skills/wip"]     # 要排除的路径（完全隐藏）
disabled = ["wip-skill"]              # 保持列出但不活动的技能名
```

`paths` 中的每一项是一个 `SKILL.md` 文件，或 Grok 递归走访的目录。`ignore` 完全隐藏技能；`disabled` 把它留在列表中，但排除出系统提示和调用。`paths` 和 `ignore` 接受文件系统路径并支持 `~` 展开；`disabled` 接受技能名。

---

## 创建技能

### 目录结构

每个技能住在自己的目录里，带一个 `SKILL.md` 文件：

```
~/.grok/skills/
  commit/
    SKILL.md
  review-pr/
    SKILL.md
  deploy/
    SKILL.md
```

### SKILL.md 格式

技能文件有 YAML frontmatter，后面是 markdown 说明：

```markdown
---
name: commit
description: Create well-formatted git commits following conventional commit standards. Use when the user wants to commit changes or asks for /commit.
---

# Git Commit Skill

Review staged changes and create a commit with a clear, conventional message.

## Steps

1. Run `git diff --staged` to see changes
2. Summarize what changed and why
3. Create commit message following conventional commits format
4. Run `git commit -m "..."` with the message
```

### 核心 Frontmatter 字段

| 字段 | 说明 |
|-------|-------------|
| `name` | 技能标识。使用小写字母、数字和连字符，最多 64 个字符。Grok 把空格和下划线规范成连字符。若省略 `name`，Grok 使用技能的目录名。 |
| `description` | 技能做什么以及何时使用。Grok 读这个来决定是否调用该技能。若省略，Grok 使用正文第一段。 |

写具体的 `description`。它决定 Grok 何时自动调用该技能。点名触发短语和用例。

### 可选 Frontmatter 字段

多词 frontmatter 键使用 kebab-case（`model` 这类单词键按原样写）。

| 字段 | 说明 |
|-------|-------------|
| `when-to-use` | 自动调用的触发短语，与 `description` 分开保存。 |
| `allowed-tools` | 技能使用的工具，YAML 列表或逗号/空格分隔的字符串。 |
| `argument-hint` | 斜杠命令自动补全中显示的提示文本（例如 `commit message`）。 |
| `user-invocable` | 是否可以作为斜杠命令运行该技能。默认为 `true`；设为 `false` 可从斜杠命令中隐藏。（要阻止模型调用技能，改为设置 `disable-model-invocation`。） |
| `disable-model-invocation` | 为 `true` 时，只有你的斜杠命令运行该技能——模型不能自动调用。默认为 `false`。 |
| `model` | 运行该技能的模型覆盖。 |
| `effort` | 推理力度覆盖。 |
| `license` | 许可证标识（例如 `Apache-2.0`）。 |
| `compatibility` | 环境要求（例如 `Requires git, docker, jq`）。 |
| `metadata` | 任意字符串键值对。Grok 提升 `metadata.author` 和 `metadata.short-description` 用于显示。 |

---

## 用 /create-skill 创建技能

`/create-skill` 命令带你交互式地构建新技能。Grok 询问你想要什么，起草文件，并写到磁盘。

### 工作方式

运行 `/create-skill` 时，Grok 会：

1. **收集需求。** Grok 询问技能名、要保存到的范围，以及你想记录的工作流描述。名称使用小写字母、数字和连字符（2–64 个字符，以字母或数字开头和结尾）。

2. **起草描述。** Grok 写一条 `description`，说明技能做什么、触发它的短语，以及斜杠命令名。你批准或编辑草稿后再继续。

3. **创建技能目录。** Grok 创建 `<scope>/.grok/skills/<name>/` 目录，技能需要时还会加上 `scripts/` 或 `references/` 子目录。

4. **写入 SKILL.md。** Grok 写入 frontmatter（`name` 和 `description`）以及说明的 markdown 正文，连同任何配套文件。

5. **验证并确认。** Grok 回读文件，确认写入正确，并告诉你如何运行该技能。

### 选择范围

Grok 询问把技能存到哪里：

- **项目**（`<repo_root>/.grok/skills/<name>/`）——仅在本仓库可用，并可通过版本控制与队友共享。在 git 仓库内 Grok 推荐此范围。
- **用户**（`~/.grok/skills/<name>/`）——在你所有项目中可用。

要把技能分发给整个团队或组织，把它打进插件并通过市场发布。见 [创建你自己的市场](09-plugins.md#create-your-own-marketplace) 和 [在组织内分发](09-plugins.md#distribute-across-an-organization)。

新技能会在几秒内出现在斜杠菜单里，因为磁盘上的文件变化时 Grok 会重新加载技能。

---

## 使用技能

### 按名称运行技能

每个技能都是以技能命名的斜杠命令。输入其名称即可运行：

```
/commit              # 运行 "commit" 技能
/review-pr           # 运行 "review-pr" 技能
```

运行技能会把它的说明加载进对话，并指导模型遵循它们。要传参数，把它们打在名称后面：

```
/commit fix the build
```

要浏览你的技能，输入 `/` 打开斜杠命令菜单。Grok 列出每个内置命令和技能，并随你输入过滤。要从命令行列出技能，运行 `grok inspect`（见 [查看技能详情](#viewing-skill-details)）。

### 限定名

当技能名与另一技能或内置命令冲突时，Grok 保持**两者**都可调用。内置保留裸名（`/login`、`/compact` 等）。技能以范围前缀的限定名公布——`local:`、`repo:`、`user:` 或插件名：

```
/local:commit        # 来自 ./.grok/skills/ 的 "commit" 技能
/user:commit         # 来自 ~/.grok/skills/ 的 "commit" 技能
/acme:login          # 名为 "login" 的插件技能（内置 /login 不变）
```

在斜杠菜单中输入 `/login` 会显示两行，右侧带 **built-in** 或 **skill · plugin-name** 徽章，方便区分。若想让技能占用裸 `/name`，重命名技能（或其目录）。

`grok inspect` 会给冲突技能打上 `[collides with /login → /acme:login]`。

### 自动调用

Grok 在识别到相关任务时可以自行调用技能。Grok 把你的提示与技能的 `description` 和 `when-to-use` 字段匹配，因此两者都要写成描述触发情境。

例如，若技能的描述写着 "Use when the user wants to commit changes"，那么说「提交我的更改」可以自动触发该技能。要要求显式斜杠命令并阻止自动调用，在 frontmatter 中设 `disable-model-invocation: true`。

---

## 查看技能详情

运行 `grok inspect` 可查看 Grok 发现的每个技能，以及其余配置：

```bash
grok inspect          # 人类可读摘要
grok inspect --json   # 机器可读报告
```

在人类可读输出中，Skills 节列出每个技能的名称及其来源——`project`、`user`、`bundled`、`config`（`[skills].paths` 条目）、`server`（从托管工作区技能商店同步的技能）或 `plugin: <name>`。Grok 会给通过 `[skills].disabled` 禁用或来自已禁用厂商表面的技能打上 `[disabled]`。

报告遵守你的 `[skills]` 配置，方式与实时会话相同：来自 `paths` 的技能会列出，`ignore` 前缀下的技能会隐藏，`disabled` 中点名的技能保持列出但标为 `[disabled]`。

`--json` 报告包含每个技能的完整细节：其 `name`、`description`、`source`（带 SKILL.md 文件路径）和 `userInvocable` 旗标。裸斜杠名有争议的技能——被内置命令或另一技能争用——还会包含 `collidesWith`（有争议的名称）和 `invocableAs`（要输入的限定命令）。

---

## 捆绑与插件技能

Grok 把平台技能与你的个人技能分开分发。捆绑技能缓存在 `~/.grok/bundled/skills/` 下；Grok 从不把它们写入 `~/.grok/skills/`。同名的本地、仓库或用户技能覆盖捆绑副本。`grok inspect` 按实际来源给每个定义打标。（同名的插件技能不会覆盖原生技能；它以限定的 `plugin:name` 形式保持可用。）

技能也可以来自插件。安装包含技能的插件后，它们会与你的用户和项目技能一起出现。`grok inspect` 把每个插件提供的技能以来源 `plugin: <name>` 打标。

安装提供技能的插件详见 [插件指南](09-plugins.md)。

---

## 最佳实践

1. **写具体的描述。** 描述驱动自动调用。"Create git commits" 太含糊；"Create well-formatted git commits following conventional commit standards. Use when the user wants to commit changes or asks for /commit." 更好。

2. **包含具体步骤。** 技能在给 Grok 清晰、有序的流程时效果最好。

3. **按名称引用工具。** 当技能依赖特定工具（例如 `run_terminal_command` 或 `search_replace`）时，点名它们，让模型知道该用什么。

4. **保持技能聚焦。** 每个工作流写一个技能。「deploy」技能和「rollback」技能比单个「deploy-and-rollback」技能更好。

5. **把项目技能纳入版本控制。** 把 `.grok/skills/` 提交到仓库，让整个团队受益。`~/.grok/skills/` 中的用户技能保持个人且不共享。

6. **通过运行来测试。** 调用 `/name` 并确认技能有效，再依赖自动调用。
