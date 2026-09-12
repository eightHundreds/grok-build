# grok clone

`grok clone` 把 Git 仓库取到 Grove 内容存储，并挂载投影工作树
（macOS 上是 NFS，Linux 上是 FUSE）。每次调用都会读 Grove 配置
（`~/.config/grove/config.toml`）里的 `[clone] enabled`，以及本进程的
`GROK_CLONE` / `GROVE_CLONE`。Grove daemon 不会授权 Clone IPC。

这**不会**为会话 / `-w` 工作树启用 Grove。那些走另一套门闩
（`GROK_WORKTREE_TYPE` 和 `~/.grok/config.toml` 里的 `[cli] grove_worktree`；
见 [配置参考](26-config-reference.md)）。
`GROK_WORKTREE_TYPE` 和 `[cli] grove_worktree` **不会**启用 `grok clone`。

```bash
grok clone <url> [dir] [--branch NAME] [--cone PATH]... [--full-history]
```

## 历史

**默认是对所选分支做 depth-1 引导**（`blob:none` +
`--depth=1`）。只有该分支会被宣传为远程跟踪引用。

需要完整提交历史、标签，或克隆时就要每一个远程分支时，用
`--full-history`（以前的默认）。

depth-1 克隆之后，这些命令**只加深所选分支**：

```bash
git fetch --deepen=N origin
git fetch --unshallow origin
```

取另一个分支需要显式的限深 refspec。普通的
`git fetch origin` 或 `git fetch origin other` 不会通过默认 refspec
拉下该分支的完整历史：

```bash
git fetch --depth=1 origin refs/heads/NAME:refs/remotes/origin/NAME
```

默认浅克隆需要理解 `clone_shallow` RPC 的 Grove daemon。若客户端拒绝，
重启或更新 daemon（或传 `--full-history`）：

```bash
grove daemon --foreground
```

在 macOS 上也可以安装 KeepAlive agent：

```bash
grove doctor --install-agent
```

## 认证

克隆用的 Git 凭据属于 **Grove daemon**，不属于 `grok login`。
这两套世界是分开的：

| 世界 | 覆盖 | 命令 | 存储 |
|-------|--------|----------|-------|
| Grok | 模型和 API | `grok login`、`grok logout` | `~/.grok/auth.json` |
| Grove Git | 这次克隆拉取的远程 | `grove status`、`grove reload-credentials` | daemon 的凭据单元，来自 Grove 配置的 `auth_mode`（`git credential` helper、carrier token 文件，或 `GROVE_AUTH_TOKEN`） |

`grok clone` 从不为 Git 读 `~/.grok/auth.json`。登录 Grok 不会
给 daemon 一份远程凭据，`[clone] enabled = true` 也不会：该旗标是
**产品门闩**，决定 `grok clone` 是否运行，不是 GitHub 授权。

当 Grove 把失败归类为凭据问题时，克隆会打印类别和该由谁处理的命令，
不带远程 URL：

```
Grove Git credentials rejected (unavailable).
Grove Git credentials belong to the Grove daemon, not `grok login`.
Check `grove status` then `grove reload-credentials`.
```

| 类别 | 含义 | 下一步 |
|-------|---------|-----------|
| `unavailable` | daemon 没有可用凭据，或远程拒绝了它 | `grove status` 会点名当前 provider；修好来源后 `grove reload-credentials` |
| `expired-static` | token 过期且此部署不刷新 token | 开新会话，或启用 `GROVE_TOKEN_ROTATION=expected` |
| `carrier-stale` | daemon 等过 carrier 重写，仍拿着被拒绝的 token | 等重写，或 `grove reload-credentials` |
| `other` | 凭据 provider 因其他原因失败 | `grove status`，然后 `grove reload-credentials` |

只有 `expired-static` 和 `carrier-stale` 会在消息里多加一行。
`unavailable` 的建议取决于 daemon 拿着哪个 provider，而克隆从不读那个，
所以它只指向 `grove status` 和 `grove doctor`。

有一种凭据拒绝保持未分类：当 token 看不到私有仓库时，GitHub 的回答
就像仓库不存在。这和公开 URL 打错无法区分，因此克隆会报成缺失仓库，
而不是猜测凭据。

`grove status` 即使没有挂载也会打印 daemon 范围的 auth 块：

```
  auth: mode=auto live=git-delegate health=ok last=- reload=supported
        hint=credentials look healthy
```

`mode` 是配置的 `auth_mode`；`live` 是 daemon 实际持有的 provider。
`mode` 和 `live` 可以不一致——这正是 `grove reload-credentials` 要修的。
例如 `auth_mode = auto` 时，在 token 文件写好之前启动的 daemon 会停在
`git-delegate`，直到单元被重建。

```bash
grove reload-credentials
# grove: credentials reloaded → token-file
```

Reload 从环境和磁盘重建单元，并打印落到的 provider；不打印秘密。
它不能创建登录：在 `auto` 或 `git` 且没有 carrier token 时，先配置
`git credential` 或 `gh auth`，再 reload。

`grove doctor` 用一条 finding 报告同样字段：`auth.ok`、
`auth.degraded`、`auth.unavailable`、`auth.daemon-down` 或 `auth.old-daemon`。

## Daemon

`grok clone` 使用活着的 Grove daemon。若控制套接字挂了，它会把
`grove daemon --foreground` 作为分离进程启动（这样退出或 Ctrl-C `grok`
不会带走 daemon 或其挂载），并等待套接字。

`grove` 二进制从 `PATH` 解析，然后从 `grok` 可执行文件所在目录
（例如 `grok` 旁边的 `~/.grok/bin/grove`）。没有单独安装位置。
macOS 没有 grove 的 PATH 包；从 monorepo 构建：

```bash
cargo build -p grove --release
```

在 Linux 上，克隆在启动 daemon 前需要可用的 FUSE：`/dev/fuse` 必须
存在，且当前用户能打开它，或 PATH 上有 setuid 的 `fusermount3` /
`fusermount` helper（Grove 两条路都能挂）。缺少 FUSE 是硬错误并带
安装命令，而不是挂起。若 daemon 已经在跑则跳过检查，因为那个
daemon 可能持有本进程没有的权限。

Windows 不受支持（没有 ProjFS 后端）。用 `git clone`，或在 macOS
或 Linux 上跑 `grok clone`。
