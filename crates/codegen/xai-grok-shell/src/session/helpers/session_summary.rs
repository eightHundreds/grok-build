//! Session title generation via LLM tool call.

use xai_grok_sampler::SamplerConfig;
use xai_grok_sampling_types::ApiBackend;

use crate::sampling::{
    Client as OaiCompatClient, ConversationItem, ConversationRequest, ConversationToolChoice,
    SamplingClient, ToolSpec,
};
use crate::session::helpers::chat::floor_char_boundary;

/// Upper bound on the user text that feeds title generation.
/// Titles only need the opening, and this keeps the request well under the model prompt limit.
const TITLE_SOURCE_MAX_BYTES: usize = 8_000;

/// Real-user turn counts at which the auto title is refreshed from the whole conversation, then frozen.
/// Used when `[session] title_refresh_turns` is unset.
/// Refreshing at a couple of early turns lets the title catch up to the real topic without churning enough to make sessions hard to recognize.
/// A manual `/rename` always wins and stops refreshes.
pub(crate) const TITLE_REFRESH_TURNS: [usize; 2] = [3, 6];

/// Built-in system prompt for first-prompt title generation ([`generate_session_summary`]).
/// Used when `[session] title_prompt` is unset.
pub(crate) const DEFAULT_TITLE_PROMPT: &str = r#"You are tasked with generating the session title. The user is asking almost always software engineering related questions on their codebase.
We describe the session title below
# Session Title
A short and distinctive 5-10 word descriptive title for the session. Super info dense, no filler.

You will be given the user query below encapsulated in <user_query></user_query>.

Just generate the session_title and nothing else"#;

/// Built-in instruction for whole-conversation title refresh.
/// Used when `[session] title_prompt` is unset.
pub(crate) const DEFAULT_TITLE_REFRESH_INSTRUCTION: &str = "Generate a session title for the conversation above. It should be a short and \
distinctive 5-10 word descriptive title capturing what this session is actually about \
(the main task or topic), based on the WHOLE conversation — not just the first message. \
Super info dense, no filler. User-role messages wrapped in reminder tags like this one \
are injected context, not the user.\n\n\
Output ONLY the title: plain text, no quotes, no labels, no markdown. Do NOT call any \
tools — respond with plain text only.";

/// Normalize configured refresh turns: drop zeros, sort, dedup.
/// `None` means the built-in [`TITLE_REFRESH_TURNS`]. An explicit empty list means no refresh checkpoints (frozen after the first title).
pub(crate) fn resolve_title_refresh_turns(configured: Option<&[u32]>) -> Vec<usize> {
    match configured {
        None => TITLE_REFRESH_TURNS.to_vec(),
        Some(turns) => {
            let mut out: Vec<usize> = turns
                .iter()
                .copied()
                .filter(|&t| t > 0)
                .map(|t| t as usize)
                .collect();
            out.sort_unstable();
            out.dedup();
            out
        }
    }
}

/// Treat blank / whitespace-only prompt overrides as unset.
pub(crate) fn resolve_title_prompt(configured: Option<&str>) -> Option<&str> {
    configured.map(str::trim).filter(|s| !s.is_empty())
}

/// System prompt for first-prompt title generation.
pub(crate) fn title_generation_system_prompt(configured: Option<&str>) -> &str {
    resolve_title_prompt(configured).unwrap_or(DEFAULT_TITLE_PROMPT)
}

/// Number of `refresh_turns` checkpoints reached at `turns` real-user turns, i.e. the checkpoint index to advance to.
/// This catches up past any checkpoints a burst of turns jumped over.
/// Equal to `refresh_turns.len()` means the title is frozen.
pub(crate) fn checkpoints_reached(turns: usize, refresh_turns: &[usize]) -> usize {
    refresh_turns.iter().filter(|&&t| turns >= t).count()
}

/// Hard byte cap guarding runaway title output; the instruction already targets 5-10 words.
/// Applied on a char boundary, so a multibyte title is capped a little shorter, which is fine for a safety bound.
const TITLE_MAX_BYTES: usize = 80;

/// An explicit sampler route a backend pins its titles to, instead of the configured default.
#[derive(Clone)]
pub struct DirectSessionTitleRoute {
    base_url: String,
    model: String,
    api_key: String,
}

impl DirectSessionTitleRoute {
    pub fn new(
        base_url: impl Into<String>,
        model: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
        DirectSessionTitleRoute {
            base_url: base_url.into(),
            model: model.into(),
            api_key: api_key.into(),
        }
    }
}

/// Builds the title client for a daemon pinned to a direct Grok model endpoint. The route is
/// authoritative: its credential never falls through to the configured public endpoints.
pub fn build_direct_session_title_client(
    direct: DirectSessionTitleRoute,
    client_version: Option<String>,
) -> crate::sampling::Result<(SamplingClient, String)> {
    let sampling_config = direct_session_title_sampling_config(direct, client_version);
    let model = sampling_config.model.clone();
    let client = SamplingClient::new(sampling_config)?;
    Ok((client, model))
}

fn direct_session_title_sampling_config(
    direct: DirectSessionTitleRoute,
    client_version: Option<String>,
) -> SamplerConfig {
    SamplerConfig {
        api_key: Some(direct.api_key),
        base_url: direct.base_url,
        model: direct.model,
        api_backend: ApiBackend::Responses,
        context_window: 200_000,
        client_version,
        ..SamplerConfig::default()
    }
}

/// Durable title-refresh checkpoint watermark under `{session_dir}/`: the number of refresh-turn checkpoints already consumed.
/// Only a committed value is persisted, so an aborted refresh still retries.
pub(crate) const TITLE_REFRESH_WATERMARK_FILE: &str = "title_refresh_idx";

/// Load the persisted checkpoint index (unclamped).
/// `None` when the session has no watermark yet (fresh, pre-feature, or feature-was-off).
/// Clamp with [`clamp_title_refresh_idx`] against the configured checkpoint count so a stale larger value still means "frozen".
pub(crate) fn load_title_refresh_watermark(session_dir: &std::path::Path) -> Option<usize> {
    std::fs::read_to_string(session_dir.join(TITLE_REFRESH_WATERMARK_FILE))
        .ok()
        .and_then(|s| s.trim().parse::<usize>().ok())
}

/// Clamp a watermark to the number of configured checkpoints so a stale larger value still means "frozen".
pub(crate) fn clamp_title_refresh_idx(idx: usize, checkpoint_count: usize) -> usize {
    idx.min(checkpoint_count)
}

/// The checkpoint index a session starts at on spawn.
/// A managed session (has a watermark) uses it; the watermark is authoritative and durable across compaction.
/// An unmanaged session is *adopted* as open (`0`) only when the feature is enabled and it is brand new (no turns); otherwise it freezes.
pub(crate) fn initial_title_refresh_idx(
    watermark: Option<usize>,
    enabled: bool,
    turns: usize,
    checkpoint_count: usize,
) -> usize {
    match watermark {
        Some(idx) => idx,
        None if enabled && turns == 0 => 0,
        None => checkpoint_count,
    }
}

/// Resolve refresh turns from a `[session]` config table.
pub(crate) fn title_refresh_turns_from_session(
    session: &crate::agent::config::SessionConfig,
) -> Vec<usize> {
    resolve_title_refresh_turns(session.title_refresh_turns.as_deref())
}

/// Resolve a title-prompt override from a `[session]` config table.
pub(crate) fn title_prompt_from_session(
    session: &crate::agent::config::SessionConfig,
) -> Option<String> {
    resolve_title_prompt(session.title_prompt.as_deref()).map(str::to_owned)
}

/// Refresh turns from the process-effective config (dormant-session paths with no actor).
pub(crate) fn title_refresh_turns_from_effective_config() -> Vec<usize> {
    crate::config::load_effective_config()
        .ok()
        .and_then(|raw| crate::agent::config::Config::new_from_toml_cfg(&raw).ok())
        .map(|c| title_refresh_turns_from_session(&c.session))
        .unwrap_or_else(|| TITLE_REFRESH_TURNS.to_vec())
}

/// Title-prompt override from the process-effective config (persistence first-prompt generation).
pub(crate) fn title_prompt_from_effective_config() -> Option<String> {
    crate::config::load_effective_config()
        .ok()
        .and_then(|raw| crate::agent::config::Config::new_from_toml_cfg(&raw).ok())
        .and_then(|c| title_prompt_from_session(&c.session))
}

/// Persist the checkpoint index after a completed attempt.
/// It is best-effort and written atomically (temp sibling, then rename).
/// A crash mid-write therefore can't leave a partial/empty file that would load as `0` and reopen the refresh window.
pub(crate) fn save_title_refresh_watermark(session_dir: &std::path::Path, idx: usize) {
    if !session_dir.is_dir() {
        return;
    }
    let path = session_dir.join(TITLE_REFRESH_WATERMARK_FILE);
    if let Err(e) = crate::session::storage::write_bytes_atomic(&path, idx.to_string().as_bytes()) {
        tracing::warn!(error = %e, path = %path.display(), "failed to persist title refresh watermark");
    }
}

#[derive(serde::Deserialize)]
struct SessionTitle {
    session_title: String,
}

/// Remove `<system-reminder>…</system-reminder>` blocks from `text`.
/// They are system-injected context (e.g. the `/goal` setup reminder), not the user's words, so they must not drive the session title.
fn strip_system_reminder_blocks(text: &str) -> String {
    const OPEN: &str = "<system-reminder>";
    const CLOSE: &str = "</system-reminder>";
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find(OPEN) {
        let Some(before) = rest.get(..start) else {
            break;
        };
        out.push_str(before);
        let Some(after_open) = rest.get(start + OPEN.len()..) else {
            return out.trim().to_string();
        };
        // An unterminated reminder drops the remainder; it is system text
        let Some(end) = after_open.find(CLOSE) else {
            return out.trim().to_string();
        };
        let Some(next) = after_open.get(end + CLOSE.len()..) else {
            return out.trim().to_string();
        };
        rest = next;
    }
    out.push_str(rest);
    out.trim().to_string()
}

/// Text the session title is derived from: strip system reminders and skill XML markup, then cap to the first few KB.
/// Stripping runs before the cap so a leading reminder larger than the cap is still removed.
/// Callers that retain a prompt for later titling keep this, not the raw text.
pub fn title_source_text(user_message: &str) -> String {
    let without_reminders = strip_system_reminder_blocks(user_message);
    let base = if without_reminders.is_empty() {
        user_message
    } else {
        &without_reminders
    };
    let mut display =
        xai_grok_tools::implementations::skills::skill::extract_skill_display_text(base)
            .unwrap_or_else(|| base.to_string());
    display.truncate(floor_char_boundary(&display, TITLE_SOURCE_MAX_BYTES));
    display
}

/// The deterministic first-ten-words fallback shared by every initial-title path.
pub fn title_fallback_from_user_text(user_message: &str) -> String {
    let text = title_source_text(user_message);
    let s = text
        .split_whitespace()
        .take(10)
        .collect::<Vec<_>>()
        .join(" ");
    if s.is_empty() {
        "New session".to_string()
    } else {
        s
    }
}

/// Generate the initial session title from the first user message, for the fast first-prompt path ([`crate::session::summary::SummaryGenerator`]).
/// The title is later refreshed from the whole conversation at the configured refresh turns (default [`TITLE_REFRESH_TURNS`]), then frozen.
/// `title_prompt` overrides [`DEFAULT_TITLE_PROMPT`] when set.
pub async fn generate_session_summary(
    user_message: String,
    client: OaiCompatClient,
    model: &str,
    title_prompt: Option<&str>,
) -> String {
    let clean_message = title_source_text(&user_message);
    let request = ConversationRequest::from_items(vec![
        ConversationItem::system(title_generation_system_prompt(title_prompt)),
        ConversationItem::user(format!(
            r#"<user_query>
{}
</user_query>"#,
            clean_message
        )),
    ])
    .with_model(model)
    .with_tools(vec![ToolSpec {
        name: "session_title".to_owned(),
        description: Some("Generate the session_title which we use for the user_message".to_owned()),
        parameters: serde_json::json!({
            "type": "object",
            "required": ["session_title"],
            "properties": {
                "session_title": {
                    "type": "string",
                    "description": "Final session title, just 5-10 word descriptive title for the session. Super info dense, no filler."
                }
            },
            "additionalProperties": false
        }),
    }])
    .with_max_output_tokens(100)
    .with_temperature(1.0)
    .with_tool_choice(ConversationToolChoice::Function("session_title".to_owned()));

    match client.conversation_collect(request).await {
        Ok(response) => {
            if let Some(a) = response.assistant()
                && let Some(tool_call) = a.tool_calls.first()
                && let Ok(result) = serde_json::from_str::<SessionTitle>(&tool_call.arguments)
            {
                return result.session_title;
            }
            tracing::debug!(
                model = %model,
                "session title generation: response did not contain a session_title tool call"
            );
        }
        Err(e) => {
            tracing::warn!(
                model = %model,
                error = %e,
                "session title generation failed, falling back to truncated user text"
            );
        }
    }
    title_fallback_from_user_text(&clean_message)
}

/// Instruction turn appended to a conversation snapshot to refresh the auto title.
/// Like the recap / turn-summary side-calls, all directions live in one reminder-wrapped turn.
/// The model sees the whole conversation, so the title reflects the real topic rather than a possibly-useless first prompt.
/// `prompt` overrides [`DEFAULT_TITLE_REFRESH_INSTRUCTION`] when set.
pub(crate) fn title_refresh_instruction(tag: &str, prompt: Option<&str>) -> String {
    let body = resolve_title_prompt(prompt).unwrap_or(DEFAULT_TITLE_REFRESH_INSTRUCTION);
    format!("<{tag}>{body}</{tag}>")
}

/// Clean a refreshed title into a one-line string.
/// Applies the recap normalization (whitespace collapse, stray label/quote stripping) plus the [`TITLE_MAX_BYTES`] cap.
pub(crate) fn clean_title_text(raw: &str) -> String {
    let mut out = crate::session::helpers::session_recap::clean_recap_text(raw);
    if out.len() > TITLE_MAX_BYTES {
        let cut = floor_char_boundary(&out, TITLE_MAX_BYTES);
        out.truncate(cut);
        out = out.trim_end().to_string();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_TITLE_PROMPT, DEFAULT_TITLE_REFRESH_INSTRUCTION, DirectSessionTitleRoute,
        TITLE_SOURCE_MAX_BYTES, clean_title_text, direct_session_title_sampling_config,
        strip_system_reminder_blocks, title_fallback_from_user_text,
        title_generation_system_prompt, title_refresh_instruction, title_source_text,
    };

    #[test]
    fn direct_title_route_builds_its_own_sampler_config() {
        let config = direct_session_title_sampling_config(
            DirectSessionTitleRoute::new("http://127.0.0.1:4242/v1", "local-model", "direct-key"),
            Some("test-version".to_owned()),
        );

        assert_eq!("http://127.0.0.1:4242/v1", config.base_url);
        assert_eq!("local-model", config.model);
        assert_eq!(Some("direct-key"), config.api_key.as_deref());
    }

    #[test]
    fn checkpoints_reached_counts_and_catches_up() {
        use super::{TITLE_REFRESH_TURNS, checkpoints_reached};
        let turns = TITLE_REFRESH_TURNS.as_slice();
        assert_eq!(checkpoints_reached(0, turns), 0);
        assert_eq!(checkpoints_reached(2, turns), 0);
        assert_eq!(checkpoints_reached(3, turns), 1);
        assert_eq!(checkpoints_reached(5, turns), 1);
        assert_eq!(checkpoints_reached(6, turns), 2);
        // A burst past the last checkpoint catches up to frozen, no overshoot.
        assert_eq!(checkpoints_reached(50, turns), TITLE_REFRESH_TURNS.len());
    }

    #[test]
    fn resolve_title_refresh_turns_defaults_and_normalizes() {
        use super::{TITLE_REFRESH_TURNS, resolve_title_refresh_turns};
        assert_eq!(
            resolve_title_refresh_turns(None),
            TITLE_REFRESH_TURNS.to_vec()
        );
        assert_eq!(resolve_title_refresh_turns(Some(&[3, 6])), vec![3, 6]);
        assert_eq!(resolve_title_refresh_turns(Some(&[6, 3, 3, 0])), vec![3, 6]);
        assert_eq!(resolve_title_refresh_turns(Some(&[])), Vec::<usize>::new());
        assert_eq!(resolve_title_refresh_turns(Some(&[1, 4, 8])), vec![1, 4, 8]);
    }

    #[test]
    fn checkpoints_reached_honors_custom_turns() {
        use super::checkpoints_reached;
        let turns = [1usize, 4, 8];
        assert_eq!(checkpoints_reached(0, &turns), 0);
        assert_eq!(checkpoints_reached(1, &turns), 1);
        assert_eq!(checkpoints_reached(4, &turns), 2);
        assert_eq!(checkpoints_reached(8, &turns), 3);
        assert_eq!(checkpoints_reached(20, &turns), 3);
        assert_eq!(checkpoints_reached(5, &[]), 0);
    }

    /// The freeze watermark round-trips (durable across a shortened conversation, e.g. compaction).
    #[test]
    fn title_refresh_watermark_round_trips_and_clamps() {
        use super::{
            TITLE_REFRESH_TURNS, TITLE_REFRESH_WATERMARK_FILE, clamp_title_refresh_idx,
            load_title_refresh_watermark, save_title_refresh_watermark,
        };
        let dir = tempfile::TempDir::new().unwrap();
        assert_eq!(
            load_title_refresh_watermark(dir.path()),
            None,
            "missing → None"
        );
        save_title_refresh_watermark(dir.path(), 1);
        assert_eq!(load_title_refresh_watermark(dir.path()), Some(1));
        std::fs::write(dir.path().join(TITLE_REFRESH_WATERMARK_FILE), "99").unwrap();
        assert_eq!(
            clamp_title_refresh_idx(
                load_title_refresh_watermark(dir.path()).unwrap(),
                TITLE_REFRESH_TURNS.len()
            ),
            TITLE_REFRESH_TURNS.len(),
            "stale-large watermark reads as frozen"
        );
    }

    #[test]
    fn initial_title_refresh_idx_adopts_only_fresh_enabled_sessions() {
        use super::{TITLE_REFRESH_TURNS, initial_title_refresh_idx};
        let frozen = TITLE_REFRESH_TURNS.len();
        // Managed: watermark is authoritative regardless of enabled/turns.
        assert_eq!(initial_title_refresh_idx(Some(1), true, 9, frozen), 1);
        assert_eq!(
            initial_title_refresh_idx(Some(frozen), true, 0, frozen),
            frozen
        );
        // Unmanaged, enabled, and brand new: adopt open
        assert_eq!(initial_title_refresh_idx(None, true, 0, frozen), 0);
        // Unmanaged but already has turns (pre-feature, even if compacted): frozen
        assert_eq!(initial_title_refresh_idx(None, true, 5, frozen), frozen);
        // Unmanaged with the feature off: frozen even when brand new
        assert_eq!(initial_title_refresh_idx(None, false, 0, frozen), frozen);
        // Empty configured turns: freeze immediately (checkpoint count 0).
        assert_eq!(initial_title_refresh_idx(None, true, 0, 0), 0);
        assert_eq!(initial_title_refresh_idx(None, true, 5, 0), 0);
    }

    #[test]
    fn title_refresh_instruction_wraps_tag_and_asks_for_whole_conversation() {
        let text = title_refresh_instruction("system-reminder", None);
        assert!(text.starts_with("<system-reminder>"));
        assert!(text.ends_with("</system-reminder>"));
        assert!(text.contains("WHOLE conversation"));
        assert!(text.contains(DEFAULT_TITLE_REFRESH_INSTRUCTION));
    }

    #[test]
    fn title_refresh_instruction_uses_custom_prompt() {
        let text = title_refresh_instruction("system-reminder", Some("  Name this session.  "));
        assert_eq!(
            text,
            "<system-reminder>Name this session.</system-reminder>"
        );
    }

    #[test]
    fn title_generation_system_prompt_defaults_and_overrides() {
        assert_eq!(title_generation_system_prompt(None), DEFAULT_TITLE_PROMPT);
        assert_eq!(
            title_generation_system_prompt(Some("   ")),
            DEFAULT_TITLE_PROMPT
        );
        assert_eq!(
            title_generation_system_prompt(Some("  Custom title rules.  ")),
            "Custom title rules."
        );
    }

    #[test]
    fn title_refresh_turns_from_session_honors_config() {
        use super::{
            TITLE_REFRESH_TURNS, title_prompt_from_session, title_refresh_turns_from_session,
        };
        let defaults = crate::agent::config::SessionConfig::default();
        assert_eq!(
            title_refresh_turns_from_session(&defaults),
            TITLE_REFRESH_TURNS.to_vec()
        );
        assert_eq!(title_prompt_from_session(&defaults), None);

        let custom = crate::agent::config::SessionConfig {
            title_refresh_turns: Some(vec![2, 5, 0]),
            title_prompt: Some("  Keep it punchy.  ".into()),
            ..Default::default()
        };
        assert_eq!(title_refresh_turns_from_session(&custom), vec![2, 5]);
        assert_eq!(
            title_prompt_from_session(&custom).as_deref(),
            Some("Keep it punchy.")
        );
    }

    #[test]
    fn clean_title_normalizes_and_caps() {
        // Collapses whitespace and strips surrounding quotes.
        assert_eq!(
            clean_title_text("\"Fix the auth  bug\""),
            "Fix the auth bug"
        );
        let capped = clean_title_text(&"word ".repeat(50));
        assert!(capped.len() <= super::TITLE_MAX_BYTES);
    }

    #[test]
    fn title_source_text_caps_oversized_input() {
        let big = "word ".repeat(10_000);
        let out = title_source_text(&big);
        assert!(!out.is_empty() && out.len() <= TITLE_SOURCE_MAX_BYTES);
    }

    #[test]
    fn title_source_text_cap_is_utf8_safe() {
        // 3-byte chars straddle the byte cap; must truncate on a boundary, not panic.
        let big = "あ".repeat(10_000);
        let out = title_source_text(&big);
        assert!(!out.is_empty() && out.len() <= TITLE_SOURCE_MAX_BYTES);
    }

    #[test]
    fn title_source_text_strips_leading_reminder_larger_than_cap() {
        // A leading reminder bigger than the cap must still be stripped, so the title derives from the objective rather than reminder text
        let reminder = "x".repeat(TITLE_SOURCE_MAX_BYTES * 2);
        let input =
            format!("<system-reminder>\n{reminder}\n</system-reminder>\n\nbuild a mario game");
        let out = title_source_text(&input);
        assert_eq!(out, "build a mario game");
    }

    #[test]
    fn strip_removes_goal_setup_reminder_leaving_objective() {
        let input = "<system-reminder>\nA goal has been set: do stuff\nlots of rules\nStart \
                     now.\n</system-reminder>\n\nbuild a mario platformer game";
        assert_eq!(
            strip_system_reminder_blocks(input),
            "build a mario platformer game"
        );
    }

    #[test]
    fn strip_handles_unterminated_reminder() {
        assert_eq!(
            strip_system_reminder_blocks("<system-reminder>\nrules with no close tag"),
            ""
        );
    }

    #[test]
    fn strip_no_reminder_is_identity() {
        assert_eq!(
            strip_system_reminder_blocks("fix the auth bug"),
            "fix the auth bug"
        );
    }

    /// Regression: a `/goal <objective>` first turn must title off the objective, not the injected `<system-reminder>` setup block.
    #[test]
    fn fallback_titles_off_goal_objective_not_reminder() {
        let input = "<system-reminder>\nA goal has been set: do stuff\nStart \
                     now.\n</system-reminder>\n\nbuild a mario platformer game in html";
        assert_eq!(
            title_fallback_from_user_text(input),
            "build a mario platformer game in html"
        );
    }

    #[test]
    fn fallback_trims_to_words() {
        assert_eq!(
            title_fallback_from_user_text(
                "one two three four five six seven eight nine ten eleven"
            ),
            "one two three four five six seven eight nine ten"
        );
    }

    #[test]
    fn fallback_new_session_when_whitespace_only() {
        assert_eq!(title_fallback_from_user_text("   \n\t"), "New session");
    }

    #[test]
    fn fallback_strips_skill_xml_with_args() {
        let input = "<command-name>implement</command-name>\n\
                      <command-message>/implement</command-message>\n\
                      <command-args>fix the rendering bug</command-args>";
        assert_eq!(
            title_fallback_from_user_text(input),
            "/implement fix the rendering bug",
        );
    }

    #[test]
    fn fallback_strips_skill_xml_no_args() {
        let input = "<command-name>deploy</command-name>\n\
                      <command-message>/deploy</command-message>";
        assert_eq!(title_fallback_from_user_text(input), "/deploy");
    }

    #[test]
    fn fallback_plain_text_unaffected() {
        assert_eq!(
            title_fallback_from_user_text("fix the auth bug in login.rs"),
            "fix the auth bug in login.rs",
        );
    }
}
