# Fork changes

This repository is a personal fork of [xai-org/grok-build](https://github.com/xai-org/grok-build). Upstream syncs land as `Synced from monorepo`. Everything below is a change made **after** the fork, relative to that snapshot.

Functional changes first; packaging / distribution after.

## Functional

### Configurable session titles

Upstream hardcodes auto-title refresh at real-user turns **3** and **6**, and bakes in both title prompts. This fork reads them from `[session]` in `config.toml`.

| Key | Type | When unset | Notes |
| --- | --- | --- | --- |
| `session.title_refresh_turns` | `integer[]` | `[3, 6]` | Turns that regenerate the title from the whole conversation, then freeze. Zeros are dropped; the rest are sorted and deduped. An **empty** list means: title once after the first prompt, never refresh. |
| `session.title_prompt` | `string` | built-in prompt | Used for the first-prompt title **and** later whole-conversation refreshes. Blank / whitespace is treated as unset. |

Still unchanged from upstream:

- A title is generated right after the first user prompt.
- Manual `/rename` always wins; automatic generation will not overwrite it.
- `/rename --auto` hands the title back to automatic generation.
- Remote kill-switch `features.title_refresh` still applies.

Example (`~/.grok/config.toml`):

```toml
[session]
title_refresh_turns = [2, 5, 10]
title_prompt = "Name this coding session in 5-10 dense words. Output only the title."
```

Disable later refreshes (keep only the first auto title):

```toml
[session]
title_refresh_turns = []
```

Details: [config reference](crates/codegen/xai-grok-pager/docs/user-guide/26-config-reference.md), [sessions](crates/codegen/xai-grok-pager/docs/user-guide/17-sessions.md).  
Landed in [#1](https://github.com/eightHundreds/grok-build/pull/1).

### TUI dictionary (zh-CN)

User-visible shortcut-bar labels, cheatsheet section headers, and action descriptions are looked up from a compiled-in dictionary at **paint time**. Upstream English msgids stay in source.

| Switch | When | Effect |
| --- | --- | --- |
| `GROK_UI_LANG` | compile (`release.yml` sets `zh`) | Default locale baked into the binary |
| `GROK_LANG` | runtime | Overrides the compile-time default (`en`, `zh`, `zh-CN`) |

Unset / `en` keeps English (tests stay stable). Missing keys fall back to English. Model prompts and tool protocol strings are **not** in the catalog.

Files: `crates/codegen/xai-grok-i18n/`, `locales/zh-CN.toml`. Call sites: shortcuts bar paint, shortcuts cheatsheet display/search, command-palette title/rows/footer (English labels stay in `default_palette_entries()`), privacy banner, welcome menu / directory trust / consent chrome, login copy, settings modal labels/descriptions/categories (paint only; `settings/defs.rs` untouched), slash-dropdown descriptions (not `/home` names), remaining modal titles, extensions tabs, composer placeholders, `Tip:` prefix, toasts, permission-dialog titles/buttons, and how-to / tutorial pages (Chinese markdown picked by language; English doc tree unchanged).

### Worked-for footer

The turn `Worked for` line also shows output token rate and TTFT when those values are known.

## Versioning

Do **not** bump the three-digit crate semver ahead of, or independently from, upstream. `crates/codegen/xai-grok-version/Cargo.toml` stays whatever the last `Synced from monorepo` snapshot shipped (currently `1.0.32`).

Each published cut is that same X.Y.Z plus a fork suffix:

| Published | Meaning |
| --- | --- |
| `1.0.24-fork.1` | first fork cut of upstream 1.0.24（Fork 一） |
| `1.0.24-fork.2` | second cut of the same upstream（Fork 二） |
| `1.0.32-fork.1` | first cut after syncing upstream 1.0.32（计数重置） |

`GITHUB_RUN_NUMBER` is **not** used as the patch number (that produced `1.0.3` / `1.0.5` and would eventually pass upstream). Resolver: `.github/scripts/resolve-fork-version.sh`.

## Distribution (not product behavior)

These do not change the TUI/agent loop. They keep this fork off the official `x.ai/cli` / GCS channel.

- Installers and the compiled auto-updater fetch **this** repo’s GitHub Releases (`eightHundreds/grok-build`), never official CDN URLs.
- `.github/workflows/release.yml` builds `xai-grok-pager` on every push and publishes from `main` / `v*` / `workflow_dispatch`.
- Release binaries use the official **packaging** path: `--profile release-dist` (thin LTO + `codegen-units=1`), extract a debug sidecar, then `strip` the file users download. Debug files stay in Actions artifacts (14 days), not the GitHub Release. Linux/macOS stay on the GNU/Apple targets in `rust-toolchain.toml` (not official musl).

Landed in [#2](https://github.com/eightHundreds/grok-build/pull/2); slimming is on top of that workflow.

## What this fork does not change

- Model routing, tools, sandbox, MCP, and the rest of the agent runtime stay upstream behavior.
- Official changelog: [x.ai/build/changelog](https://x.ai/build/changelog).
- `SOURCE_REV` is still the monorepo commit this tree was synced from.
