//! Bash-mode (`!`) history/prompt must keep the full captured output (1 MiB runner cap).
//! The old last-10-line `... (N lines)\n{tail}` shape is gone.

use super::support::*;
use super::*;
use std::sync::Arc;
use std::time::Duration;

/// Terminal that returns a fixed stdout body and exit 0.
#[derive(Debug)]
struct FixedOutputTerminal {
    output: String,
}

#[async_trait::async_trait]
impl crate::terminal::AsyncTerminalRunner for FixedOutputTerminal {
    async fn run(
        &self,
        _request: crate::terminal::runner::TerminalRunRequest,
    ) -> Result<crate::terminal::runner::TerminalRunResult, crate::terminal::runner::TerminalError>
    {
        Ok(crate::terminal::runner::TerminalRunResult {
            combined_output: self.output.clone(),
            exit_code: Some(0),
            truncated: false,
            signal: None,
            timed_out: false,
        })
    }
}

/// 15 lines used to become `... (15 lines)` plus the last 10. The next turn now sees every line.
#[tokio::test(flavor = "current_thread")]
async fn bash_mode_history_push_keeps_full_captured_output() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let output: String = (1..=15)
                .map(|i| format!("L{i:02}"))
                .collect::<Vec<_>>()
                .join("\n");
            let (gateway_tx, _gateway_rx) =
                tokio::sync::mpsc::unbounded_channel::<xai_acp_lib::AcpClientMessage>();
            let (persistence_tx, persistence_rx) =
                tokio::sync::mpsc::unbounded_channel::<PersistenceMsg>();
            drain_persistence(persistence_rx);
            let (actor, ev) = create_test_actor_with_terminal(
                0,
                256_000,
                85,
                gateway_tx,
                persistence_tx,
                Arc::new(FixedOutputTerminal {
                    output: output.clone(),
                }),
            )
            .await;
            // Drop the event receiver so `flush_replay_actor` fails immediately instead of waiting 5s.
            drop(ev);

            tokio::time::timeout(
                Duration::from_secs(10),
                actor.handle_direct_bash_command(
                    "bash-1",
                    "printf 'L%02d\\n' $(seq 1 15)".to_string(),
                    &[acp::ContentBlock::Text(acp::TextContent::new(
                        "!printf 'L%02d\\n' $(seq 1 15)",
                    ))],
                ),
            )
            .await
            .expect("bash-mode turn timed out")
            .expect("bash-mode turn should complete");

            let conv = actor.chat_state_handle.get_conversation().await;
            let text: String = conv.iter().map(|item| item.text_content()).collect();
            for i in 1..=15 {
                assert!(
                    text.contains(&format!("L{i:02}")),
                    "next-turn history must keep line L{i:02}; got:\n{text}"
                );
            }
            assert!(
                !text.contains("... (15 lines)"),
                "next-turn history must not use the old last-10-line tail; got:\n{text}"
            );
            assert!(
                text.contains("I executed a terminal command:"),
                "history must still wrap the command as a user message; got:\n{text}"
            );
        })
        .await;
}
