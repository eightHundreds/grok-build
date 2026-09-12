# 项目规则（AGENTS.md）

项目规则让你按项目或目录配置 Grok。在仓库中放置 AGENTS.md 文件，即可设置编码约定、构建说明、风格指南，以及 Grok 在该代码库中工作时应遵循的任何其他指令。启动加载需要文件夹信任（`--trust` 或交互式授权）。

---

## 什么是项目规则？

项目规则是 Grok 读取并加入上下文的 Markdown 文件。Grok 在该树中的每一次交互都会遵循其内容。

这是向 Grok 传授项目约定的主要机制，因此不必每个会话都再讲一遍。

---

## 支持的文件名

Grok 在每个目录中按此顺序检查这些文件名：

- `Agents.md`
- `Claude.md`
- `CLAUDE.md`
- `CLAUDE.local.md`
- `AGENT.md`
- `AGENTS.md`

Grok 会加载目录中每一个匹配的文件，因此同时包含 `AGENTS.md` 和 `CLAUDE.md` 的文件夹会贡献两份。在不区分大小写的文件系统上，解析到同一文件的名称（例如 `Agents.md` 和 `AGENTS.md`）会去重并只计一次。支持 `Claude.md`、`CLAUDE.md` 和 `CLAUDE.local.md`，以兼容 Claude Code 工作流。启用 Claude 兼容（默认）时，Grok 还会扫描家目录级的 `~/.claude/` 查找这些文件名，并在每一层目录检查 `.claude/CLAUDE.md` 和 `.claude/CLAUDE.local.md`——这是 Claude Code 用于项目记忆的位置。启用 Cursor 兼容时，家目录级的 `~/.cursor/` 以同样方式扫描。

### 规则目录

除 AGENTS.md 文件外，Grok 还会从仓库根到当前工作目录的每一层（`<dir>`）扫描规则目录中的 `*.md` 文件：

| 位置 | 说明 |
|----------|-------|
| `<dir>/.grok/rules/` | 始终扫描 |
| `<dir>/.claude/rules/` | Claude 兼容（可配置） |
| `<dir>/.cursor/rules/` | Cursor 兼容（可配置） |

Grok 也会扫描家目录级规则，无论从哪里启动。这些根已经按厂商区分，因此规则直接放在 `rules/` 下：

| 位置 | 说明 |
|----------|-------|
| `$GROK_HOME/rules/`（默认 `~/.grok/rules/`） | 始终扫描；适用于所有项目 |
| `~/.claude/rules/` | 由 `compat.claude.rules` 控制 |
| `~/.cursor/rules/` | 由 `compat.cursor.rules` 控制 |

家目录规则先加载，按表中顺序，然后是从仓库根到当前目录的项目文件。每个规则目录内的文件按字母顺序。厂商 `rules` 单元独立于对应的 `agents` 单元，分别控制家目录和项目规则。Claude 的 `agents` 单元控制 `~/.claude/` 下的具名文件以及项目 `<dir>/.claude/CLAUDE*.md`；`Claude.md`、`CLAUDE.md` 和 `CLAUDE.local.md` 这类通用顶层名称仍被识别。见 [配置](05-configuration.md#harness-compatibility)。

---

## 发现如何工作

Grok 按此顺序扫描项目规则：

1. **家目录规则**：`$GROK_HOME`，然后是已启用的 `~/.claude/` 和 `~/.cursor/` 源
2. **仓库规则**：若在 git 仓库内，从仓库根到当前工作目录（含）的每一个目录
3. **仅 CWD**：若不在 git 仓库内，只有当前工作目录

### 示例

给定此项目结构：

```
~/projects/my-app/
  AGENTS.md              # "Use TypeScript. Follow ESLint rules."
  src/
    AGENTS.md            # "Prefer functional components."
    components/
      AGENTS.md          # "Use CSS modules for styling."
```

当 Grok 在 `~/projects/my-app/src/components/` 中运行时，它会加载全部三个文件。指令会累积，因此 Grok 会看到全部内容。

### 更深的文件优先

Grok 把文件从仓库根排到当前工作目录，因此更深目录中的文件在上下文中更靠后，指令冲突时优先。上例中，若根目录写「Use styled-components」，但 `components/AGENTS.md` 写「Use CSS modules」，CSS modules 指令胜出，因为它出现得更晚。

### 自动加载行为

- Grok 在会话开始时自动从仓库根到当前工作目录加载这些文件。
- 当 Grok 在该初始集合之外的目录中读取、列出或编辑文件时，它会检测那里的任何项目指令文件，记下路径，并在它们适用于当前任务时读取。

---

## 项目规则里该写什么

### 编码约定

```markdown
# Coding Standards

- Use TypeScript for all new code
- Prefer functional components with hooks over class components
- Use `const` by default; only use `let` when reassignment is needed
- Maximum line length: 100 characters
```

### 构建与测试说明

```markdown
# Build & Test

- Run `npm test` before committing
- Use `npm run lint` to check code style
- Build with `npm run build` -- ensure no TypeScript errors
- Integration tests: `npm run test:e2e` (requires Docker)
```

### 风格指南

```markdown
# Style Guide

- Follow the Airbnb JavaScript Style Guide
- Use 2-space indentation
- Always use trailing commas in multi-line arrays/objects
- Prefer template literals over string concatenation
```

### PR 与提交要求

```markdown
# Version Control

- Write commit messages in conventional commits format
- Prefix branch names with `feature/`, `fix/`, or `chore/`
- All PRs require at least one approval before merge
- Squash-merge feature branches
```

### 架构说明

```markdown
# Architecture

- API routes go in `src/routes/` with one file per resource
- Business logic goes in `src/services/`
- Database queries go in `src/repositories/`
- Never import from `src/routes/` in `src/services/`
```

---

## 把规则限定到子目录

AGENTS.md 文件的作用域是以其所在文件夹为根的整棵目录树。用它为代码库的不同部分提供不同指令：

```
my-monorepo/
  AGENTS.md                    # Monorepo-wide rules
  packages/
    frontend/
      AGENTS.md                # "Use React. Prefer CSS modules."
    backend/
      AGENTS.md                # "Use Express. Follow REST conventions."
    shared/
      AGENTS.md                # "No framework-specific code in this package."
```

---

## 会话规则标志

要为单个会话添加规则而不编辑文件，传入 `--rules`（别名 `--append-system-prompt`）：

```bash
grok --rules "Always use TypeScript. Prefer functional components."
```

Grok 把这段文字追加到会话的系统提示。用于会话级定制。

要完全替换系统提示，传入 `--system-prompt-override`（别名 `--system-prompt`）。Grok 原样使用该文本，并跳过默认系统提示和 `--rules`。（相比之下，用 `--rules` 传入的文本会包在 `<human_rules>` 块中，追加到默认提示。）

---

## 文件大小

Grok 完整加载每个项目指令文件；没有字符上限，也不会截断。即便如此，请保持指令简洁、聚焦。更短、更具体的规则比长篇更容易被 Grok 遵循，而且加载的每个文件都会消耗上下文。

---

## Gitignore 过滤

被 `.gitignore` 忽略的文件在发现时会被跳过。要把个人覆盖留在共享仓库之外，gitignore 一个已识别的文件名，例如 `CLAUDE.local.md`：

```gitignore
# .gitignore
CLAUDE.local.md
```

作为顶层指令文件，Grok 只发现 [支持的文件名](#supported-file-names) 下列出的已识别文件名——不会发现 `AGENTS.local.md` 或 `notes.md` 这类自定义名称。（在 `.grok/rules/` 这类规则目录内，每一个 `*.md` 文件都会加载，不论名称。）

---

## `.grok/` 项目目录

除 AGENTS.md 文件外，项目根目录的 `.grok/` 还可以包含额外的项目级配置：

| 路径 | 用途 |
|------|---------|
| `.grok/config.toml` | 项目作用域的 MCP 服务器、插件和权限规则（其他设置只从 `~/.grok/config.toml` 加载） |
| `.grok/skills/` | 项目作用域的技能定义 |
| `.grok/plugins/` | 项目作用域的插件 |
| `.grok/agents/` | 项目作用域的 agent 定义 |
| `.grok/hooks/` | 项目作用域的生命周期钩子 |
| `.grok/lsp.json` | LSP 服务器配置 |

这些都是可选的。各指南见对应文档。

---

## 检查已加载的规则

用 `grok inspect` 查看所有已加载的项目指令：

```bash
grok inspect
```

它会显示找到的每个项目指令文件，以及路径和大致 token 数。用它确认 Grok 拾取了你的规则。

---

## 最佳实践

1. **从根目录开始。** 把最重要、全项目的规则放在仓库根的 AGENTS.md。

2. **写具体。** 「Use TypeScript」比「Use modern JavaScript」更好。「提交前运行 `cargo fmt`」比「格式化代码」更好。

3. **保持简短。** 简洁指令比冗长的更容易被遵循。

4. **大型仓库用子目录限定。** 单体仓库的不同部分可能有不同约定。用按目录的 AGENTS.md 适当限定规则。

5. **把规则纳入版本控制。** 把 AGENTS.md 提交到仓库，让整个团队受益。用户特定的覆盖放在 `~/.grok/`（全局规则）。

6. **不要重复文档。** AGENTS.md 应包含可执行的指令，而不是项目 README 的副本。需要时链接到外部文档。

7. **定期复查。** 随着项目演进，更新规则以匹配当前约定。
