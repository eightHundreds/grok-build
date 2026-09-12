//! Chinese tutorial pages. English sources stay in `docs/tutorial/`.

pub fn content_for(title: &str) -> Option<&'static str> {
    Some(match title {
        "Coming from Claude, Cursor, or Codex?" => {
            include_str!("../docs/tutorial/zh-CN/01-coming-from-another-tool.md")
        }
        "Your First Prompt" => include_str!("../docs/tutorial/zh-CN/02-first-prompt.md"),
        "Attach Files, Images & Paste" => {
            include_str!("../docs/tutorial/zh-CN/03-attach-and-paste.md")
        }
        "Finding Your Way Around" => include_str!("../docs/tutorial/zh-CN/04-navigation.md"),
        "Slash Commands" => include_str!("../docs/tutorial/zh-CN/05-slash-commands.md"),
        "Parallel Work: Worktrees" => include_str!("../docs/tutorial/zh-CN/06-worktrees.md"),
        "Plan Mode & Permissions" => {
            include_str!("../docs/tutorial/zh-CN/07-plan-and-permissions.md")
        }
        "Make It Yours" => include_str!("../docs/tutorial/zh-CN/08-make-it-yours.md"),
        "Where to Go Next" => include_str!("../docs/tutorial/zh-CN/09-where-next.md"),
        _ => return None,
    })
}

pub fn localized_content(title: &str, english: &'static str) -> &'static str {
    if xai_grok_i18n::locale() == xai_grok_i18n::Locale::ZhCn {
        content_for(title).unwrap_or(english)
    } else {
        english
    }
}
