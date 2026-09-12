//! Chinese how-to bodies. English sources stay in `docs/user-guide/` and `docs/`.

pub fn content_for(filename: &str) -> Option<&'static str> {
    Some(match filename {
        "01-getting-started.md" => include_str!("../docs/user-guide/zh-CN/01-getting-started.md"),
        "02-authentication.md" => include_str!("../docs/user-guide/zh-CN/02-authentication.md"),
        "03-keyboard-shortcuts.md" => {
            include_str!("../docs/user-guide/zh-CN/03-keyboard-shortcuts.md")
        }
        "04-slash-commands.md" => include_str!("../docs/user-guide/zh-CN/04-slash-commands.md"),
        "05-configuration.md" => include_str!("../docs/user-guide/zh-CN/05-configuration.md"),
        "06-theming.md" => include_str!("../docs/user-guide/zh-CN/06-theming.md"),
        "07-mcp-servers.md" => include_str!("../docs/user-guide/zh-CN/07-mcp-servers.md"),
        "08-skills.md" => include_str!("../docs/user-guide/zh-CN/08-skills.md"),
        "09-plugins.md" => include_str!("../docs/user-guide/zh-CN/09-plugins.md"),
        "10-hooks.md" => include_str!("../docs/user-guide/zh-CN/10-hooks.md"),
        "11-custom-models.md" => include_str!("../docs/user-guide/zh-CN/11-custom-models.md"),
        "12-project-rules.md" => include_str!("../docs/user-guide/zh-CN/12-project-rules.md"),
        "13-memory.md" => include_str!("../docs/user-guide/zh-CN/13-memory.md"),
        "14-headless-mode.md" => include_str!("../docs/user-guide/zh-CN/14-headless-mode.md"),
        "15-agent-mode.md" => include_str!("../docs/user-guide/zh-CN/15-agent-mode.md"),
        "16-subagents.md" => include_str!("../docs/user-guide/zh-CN/16-subagents.md"),
        "17-sessions.md" => include_str!("../docs/user-guide/zh-CN/17-sessions.md"),
        "18-sandbox.md" => include_str!("../docs/user-guide/zh-CN/18-sandbox.md"),
        "19-plan-mode.md" => include_str!("../docs/user-guide/zh-CN/19-plan-mode.md"),
        "20-background-tasks.md" => include_str!("../docs/user-guide/zh-CN/20-background-tasks.md"),
        "21-terminal-support.md" => include_str!("../docs/user-guide/zh-CN/21-terminal-support.md"),
        "22-permissions-and-safety.md" => {
            include_str!("../docs/user-guide/zh-CN/22-permissions-and-safety.md")
        }
        "23-dashboard.md" => include_str!("../docs/user-guide/zh-CN/23-dashboard.md"),
        "24-monitoring-usage.md" => include_str!("../docs/user-guide/zh-CN/24-monitoring-usage.md"),
        "25-status-line.md" => include_str!("../docs/user-guide/zh-CN/25-status-line.md"),
        "26-config-reference.md" => include_str!("../docs/user-guide/zh-CN/26-config-reference.md"),
        "27-grok-clone.md" => include_str!("../docs/user-guide/zh-CN/27-grok-clone.md"),
        "hooks-and-plugins.md" => include_str!("../docs/zh-CN/hooks-and-plugins.md"),
        "custom-hooks.md" => include_str!("../docs/zh-CN/custom-hooks.md"),
        _ => return None,
    })
}
