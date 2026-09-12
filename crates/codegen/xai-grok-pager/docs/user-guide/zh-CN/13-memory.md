# 跨会话记忆

记忆让 Grok 回忆先前会话中的事实、决策和模式。Grok 为你保存的信息建立索引并自动搜索，因此新会话可以复用相关上下文。

---

## 什么是记忆？

没有记忆时，每个 Grok 会话都从零开始：模型对先前会话一无所知。启用记忆后，Grok 可以：

- 回忆你以前解释过的项目约定。
- 复用已经奏效的调试步骤。
- 把架构决策带到后续会话。
- 避免再问它已经有答案的问题。

记忆是实验性的，默认关闭。

---

## 启用记忆

### 环境变量

```bash
export GROK_MEMORY=1
grok
```

### 配置文件（持久）

```toml
# ~/.grok/config.toml
[memory]
enabled = true
```

### 强制禁用

即使 TOML 或远程设置启用了记忆，也要为本进程禁用：

```bash
export GROK_MEMORY=0
```

### 会话中途开关

会话进行中开关记忆，无需重启：

```
/memory on
/memory off
```

该开关是会话作用域的——不会持久化到 `config.toml`。关掉会移除对记忆工具的访问，但磁盘上的现有文件保留。打开会重新初始化记忆存储并注册记忆工具。

也可以在 `/memory` 弹窗内按 `t` 切换。

### 优先级

1. 隐藏的已弃用兼容标志（若提供）
2. `GROK_MEMORY` 环境变量：`1`/`true` 启用，`0`/`false` 禁用
3. 生效 TOML 中的 `[memory]` 段
4. 托管远程设置
5. 默认：关闭

---

## 记忆如何存储

记忆以 Markdown 文件存储在 `~/.grok/memory/` 下：

| 位置 | 作用域 | 说明 |
|----------|-------|-------------|
| `~/.grok/memory/MEMORY.md` | 全局 | 适用于你所有项目的事实 |
| `~/.grok/memory/<project-slug>-<hash8>/MEMORY.md` | 工作区 | 项目特定的约定和上下文 |
| `~/.grok/memory/<project-slug>-<hash8>/sessions/` | 会话 | 按会话的摘要和日志 |

Grok 给每个工作区目录加上仓库身份的短哈希后缀。身份在目录是带 `origin` 远程的 Git 仓库时是 `org/repo` 形式的 `origin` 远程，否则是目录路径。同一仓库的克隆和工作树共享 `origin` 远程，因此也共享同一个记忆目录。

SQLite 索引支持跨所有记忆文件搜索：
- **FTS5** 提供默认的全文搜索，用于关键词匹配。
- **vec0** 在配置了嵌入模型时加入向量搜索，用于语义相似。

---

## 自动保存

会话结束时，Grok 把结构化元数据摘要保存到该会话的每日日志。摘要包含：

- 消息计数（用户、助手和工具结果）。
- 主题：会话中前几条实质性用户提示，最多五条。
- 会话日期和时间（UTC）。

Grok 从对话元数据构建摘要，不调用 LLM，也不增加延迟。琐碎会话会被跳过——实质性提示少于三条，或用户文本少于 50 字节。

摘要不记录工具使用、文件路径或 shell 命令。会话 ID 构成日志文件名的一部分。要关闭自动保存，设置 `session.save_on_end = false`。要更丰富地捕获决策、模式和推理，使用 `/flush`。

---

## 用 /flush 保存丰富知识

要更丰富地捕获——决策、模式、调试工作流、API 发现——在 TUI 中使用 `/flush`：

```
/flush
```

这会触发对当前会话最重要内容的 LLM 生成摘要，并写入带日期的会话日志。摘要会被索引，可在未来会话中搜索。

想保留重要上下文时使用 `/flush`：
- 压缩之前（压缩会丢弃旧的对话回合）
- 一次高效调试会话结束时
- 发现重要模式或约定之后

---

## 使用记忆

### 记住

让 Grok 记住某事，它会把笔记追加到 `MEMORY.md` 文件——项目特定条目用工作区文件，跨项目偏好用全局 `~/.grok/memory/MEMORY.md`：

```
> remember to always open PR links after pushing
```

Grok 把条目作为持久陈述记录在有组织的标题下，例如 `## Preferences`、`## Project Context` 或 `## Debugging`。文件监视器在下一次记忆搜索时重新索引该变更，因此新条目在当前会话内即可搜索。

也可以用 `/remember` 命令直接保存笔记：

```
/remember always open PR links after pushing
```

不带文本运行 `/remember` 进入记住模式，你输入的下一行成为笔记。无论哪种方式，Grok 都会打开审阅面板显示笔记（可用 `Tab` 切换可选的改写版本）；只有你确认后才会写入。保存时，Grok 显示 `Memory saved to ~/.grok/memory/MEMORY.md`。

### 忘记

让 Grok 忘记某事，它会查找并移除匹配的条目：

```
> forget the snake_case convention
```

忘记是尽力而为：模型搜索记忆并移除匹配的条目。要保证删除，直接编辑 `~/.grok/memory/` 下的文件并自己删掉条目。要定位文件，打开 `/memory` 浏览器并按 `y` 复制其路径。

### 回忆

询问 Grok 还记得什么：

```
> what do you remember?
```

Grok 跨所有记忆文件搜索，并按来源分组总结它所知道的：全局偏好、项目特定知识和会话历史。用 `/memory` 浏览原始文件。

### 直接编辑

你可以直接编辑 `~/.grok/memory/` 下的记忆文件。文件监视器在下一次记忆搜索时重新索引你的变更。用 `/flush` 立即保存当前会话，用 `/dream` 把会话日志整理成有组织的主题。

---

## 用 /memory 浏览记忆

`/memory` 命令打开显示所有记忆文件的弹窗：

```
/memory
```

文件按作用域分组：
- **全局** —— 跨项目记忆（`MEMORY.md`）。
- **工作区** —— 项目特定记忆（`MEMORY.md`）。
- **会话** —— 按会话的摘要，按时间倒序。

弹窗使用分栏布局：左侧文件列表，右侧只读内容预览。预览随你在列表中移动而更新。

### 键盘快捷键

| 按键 | 动作 |
|-----|--------|
| `↑`/`↓` 或 `j`/`k` | 在文件列表中移动 |
| `PgUp`/`PgDn` | 跳 10 条 |
| `/` | 过滤文件列表 |
| `y` | 把选中文件的路径复制到剪贴板 |
| `x` | 删除选中的会话文件（再按一次 `x` 确认） |
| `t` | 开关记忆 |
| `Ctrl+F` | 开关全屏 |
| `Esc` | 关闭弹窗，或退出过滤模式 |

预览窗格是只读的。用鼠标滚轮或拖动滚动条滚动。只能删除会话文件，不能删除全局或工作区的 `MEMORY.md`。

当记忆弹窗的内容区不足 80 列时，弹窗隐藏预览窗格，只显示文件列表。

也可以从命令面板打开 `/memory`。

---

## 记忆通知

用 `/remember` 保存笔记时，Grok 在回看中确认：

```
Memory saved to ~/.grok/memory/MEMORY.md
```

后台保存——flush、dream 和会话结束——静默运行，不往回看发消息。随时用 `/memory` 浏览 Grok 已存储的内容。

---

## 用 /dream 做 Dream 整理

`/dream` 命令把零散的记忆片段整理成有组织的主题：

```
/dream
```

Dream 把各个会话日志和记忆条目重组为连贯、去重的知识库，从而随时间降低噪声并提高搜索质量。`/dream` 需要启用记忆。

### 自动 Dream

Dream 也会自动运行。默认情况下，Grok 在启动时以及会话中定期检查整理门闩，并在经过足够时间和积累足够会话后运行一次 Dream：

```toml
[memory.dream]
enabled = true     # 运行自动整理（默认：true）
min_hours = 24     # 两次整理之间的最少小时数
min_sessions = 5   # 自上次整理以来的最少会话数
check_interval_secs = 3600 # 也按小时检查门闩
```

---

## 记忆如何影响提示

### 首回合注入

每个会话的第一回合，Grok 自动搜索与当前项目相关的记忆并作为上下文注入。这意味着 Grok 无需提醒就能带着先前会话的知识开始。

首回合注入可以配置：

```toml
[memory.initial_injection]
enabled = true     # 启用或禁用首回合注入
min_score = 0.9    # 首回合注入的分数阈值
```

### 压缩之后

自动压缩之后也会搜索记忆，以恢复可能被丢弃的相关上下文。

---

## 记忆搜索

Grok 自动搜索记忆，你也可以在聊天中手动触发搜索：

```
Search memory for "auth middleware patterns"
Read my workspace MEMORY.md
```

模型可以使用两个记忆工具：
- `memory_search` —— 跨所有记忆搜索
- `memory_get` —— 按路径读取特定记忆文件

### 搜索评分

默认嵌入模型未设置，因此记忆从纯全文模式开始。若配置了嵌入模型，搜索会把向量相似度（权重 `0.7`）与 BM25 文本相似度（权重 `0.3`）组合。结果按最低分数阈值过滤（默认：`0.7`）。

### 来源权重

每个记忆来源都有一个应用到其分数的权重乘数。所有来源默认 `1.0`，可以在 `[memory.search.source_weights]` 下调整任意一项：

| 来源 | 权重 | 说明 |
|--------|--------|-------------|
| `workspace` | 1.0 | 项目特定记忆 |
| `session` | 1.0 | 会话日志 |
| `global` | 1.0 | 跨项目记忆 |

### 时间衰减

会话记忆会随时间衰减，从而优先最近的会话：

```toml
[memory.search.temporal_decay]
enabled = true           # 启用基于时间的衰减
half_life_days = 30.0    # 经过这么多天后分数减半
```

只有会话块会衰减。全局和工作区记忆豁免，因为它们包含精心整理的长期知识。

### MMR（Maximal Marginal Relevance）

MMR 重排会惩罚冗余结果，以提高多样性：

```toml
[memory.search.mmr]
enabled = true           # 启用多样性重排
lambda = 0.7             # 0.0 = 最大多样性，1.0 = 纯相关性
```

---

## CLI 命令

`grok memory` 命令从 shell 管理记忆。它有一个子命令 `clear`：

```bash
# 清除工作区记忆（MEMORY.md、sessions/ 和 index.sqlite）。这是默认作用域。
grok memory clear

# 同一作用域，显式写出
grok memory clear --workspace

# 清除全局 MEMORY.md
grok memory clear --global

# 同时清除工作区和全局记忆
grok memory clear --all

# 跳过确认提示（-y 是短形式）
grok memory clear --yes
```

要从 shell 编辑记忆，直接在编辑器中打开文件——例如 `$EDITOR ~/.grok/memory/MEMORY.md`。

---

## 配置参考

### 核心设置（`[memory]`）

| 键 | 默认 | 说明 |
|-----|---------|-------------|
| `enabled` | `false` | 启用记忆 |
| `session.save_on_end` | `true` | 会话结束时写入元数据摘要 |
| `watcher.enabled` | `true` | 监视 `~/.grok/memory/` 的外部编辑并重新索引 |

### 索引设置（`[memory.index]`）

| 键 | 默认 | 说明 |
|-----|---------|-------------|
| `max_chunk_chars` | `1600` | 最大块大小（字符） |
| `chunk_overlap_chars` | `320` | 块之间的字符重叠 |

### 嵌入设置（`[memory.embedding]`）

| 键 | 默认 | 说明 |
|-----|---------|-------------|
| `provider` | `"api"` | 嵌入提供商（目前为 `"api"`） |
| `model` | 未设置 | 嵌入模型名。未设置或 `""` 使用纯全文检索。 |
| `dimensions` | `1024` | 嵌入向量维度 |

### 搜索设置（`[memory.search]`）

| 键 | 默认 | 说明 |
|-----|---------|-------------|
| `max_results` | `6` | 最大搜索结果数 |
| `min_score` | `0.7` | 最低相关分数 |
| `vector_weight` | `0.7` | 向量相似度权重 |
| `text_weight` | `0.3` | BM25 文本相似度权重 |

### 初始注入设置（`[memory.initial_injection]`）

| 键 | 默认 | 说明 |
|-----|---------|-------------|
| `enabled` | `true` | 启用首回合记忆注入 |
| `min_score` | `0.9` | 首回合结果的分数阈值 |

### Dream 设置（`[memory.dream]`）

| 键 | 默认 | 说明 |
|-----|---------|-------------|
| `enabled` | `true` | 启用自动 Dream 整理 |
| `min_hours` | `24` | 两次整理之间的最少小时数 |
| `min_sessions` | `5` | 自上次整理以来的最少会话数 |
| `stale_lock_secs` | `3600` | 过期整理锁被回收前的秒数 |
| `check_interval_secs` | `3600` | 定期 Dream 门闩检查间隔（秒）。设为 `0` 禁用定期检查。 |

### Flush 设置（`[compaction.memory_flush]`）

Flush 在 `[compaction]` 下配置，不在 `[memory]` 下，因为它是压缩行为。

| 键 | 默认 | 说明 |
|-----|---------|-------------|
| `enabled` | `true` | 启用压缩前的记忆 flush |
| `soft_threshold_tokens` | `4000` | 触发 flush 的、压缩阈值之前的 token 余量 |
| `max_flush_write_chars` | `8000` | flush 可写入记忆的最大字符数 |
| `flush_model` | 未设置 | flush 回合使用的模型。未设置或 `""` 时，Grok 使用会话的主模型。 |
| `idle_timeout_secs` | `300` | 后台 flush 前的空闲秒数。设为 `0` 禁用空闲 flush。 |
| `semantic_dedup_threshold` | 未设置 | 对 flush 内容去重的余弦相似度阈值。未设置时默认为 `0.92`。 |

### 修剪设置（`[compaction.pruning]`）

修剪在 `[compaction]` 下配置，不在 `[memory]` 下，因为它是压缩行为。

| 键 | 默认 | 说明 |
|-----|---------|-------------|
| `enabled` | `true` | 启用工具结果修剪 |
| `keep_last_n_turns` | `3` | 最近多少个回合的工具结果永不修剪 |
| `soft_trim_threshold` | `4000` | 超过该字符阈值时，旧工具结果会被软修剪 |
| `soft_trim_head` | `1500` | 软修剪结果开头保留的字符数 |
| `soft_trim_tail` | `1500` | 软修剪结果末尾保留的字符数 |
| `hard_clear_age_turns` | `10` | 超过该回合年龄后，工具结果被占位符替换 |

---

## 记忆过期

当会话记忆变旧时，Grok 会在搜索结果中给它附上过期说明。更旧的结果会得到更强的提醒：在依赖之前先核实当前状态。这些说明帮你发现可能不再准确的已存事实。全局和工作区记忆从不带过期说明，因为它们保存的是精心整理的长期知识。

---

## 文件监视器

默认情况下，Grok 监视 `~/.grok/memory/` 的外部文件变更。若你直接编辑记忆文件（例如在编辑器中），变更会在下一次记忆搜索时自动拾取：

- 创建或修改的文件会重新索引。
- 删除的文件会从索引中移除过期块。

```toml
[memory.watcher]
enabled = true    # 默认
```

---

## 故障排除

### 记忆不工作

1. 确认记忆已启用：检查 `grok inspect` 输出。
2. 检查 `GROK_MEMORY` 或生效 TOML 中的 `[memory] enabled`。
3. 检查是否有 `GROK_MEMORY=0` 或已弃用的兼容标志覆盖了配置。

### 记忆没有出现在会话中

记忆在第一回合注入。若你在启用记忆之前就开了会话，用 `/new` 开一个新会话。

### 查看记忆文件

在 TUI 中用 `/memory` 浏览所有记忆文件并预览。也可以直接访问：

```bash
ls ~/.grok/memory/
cat ~/.grok/memory/MEMORY.md
$EDITOR ~/.grok/memory/MEMORY.md
```

### 调试日志

```bash
RUST_LOG=debug GROK_LOG_FILE=/tmp/grok.log grok
grep "memory" /tmp/grok.log
```
