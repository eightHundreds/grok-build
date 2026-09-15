//! `!` bash-mode history/prompt must keep the captured run output, not a last-N tail.
use super::support::*;
use super::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Terminal that returns more than the old 10-line history tail and records the run-request cap.
struct ManyLineTerminal {
    output_byte_limit: AtomicUsize,
}

#[async_trait::async_trait]
impl crate::terminal::AsyncTerminalRunner for ManyLineTerminal {
    async fn run(
        &self,
        request: crate::terminal::runner::TerminalRunRequest,
    ) -> Result<crate::terminal::runner::TerminalRunResult, crate::terminal::runner::TerminalError>
    {
        self.output_byte_limit
            .store(request.output_byte_limit, Ordering::SeqCst);
        let combined_output = (1..=15)
            .map(|i| format!("LINE-{i:02}"))
            .collect::<Vec<_>>()
            .join("\n");
        Ok(crate::terminal::runner::TerminalRunResult {
            combined_output,
            exit_code: Some(0),
            truncated: false,
            signal: None,
            timed_out: false,
        })
    }
}

/// History and `output_for_prompt` must include the first captured line, not `... (15 lines)` + last 10.
#[tokio::test(flavor = "current_thread")]
async fn bash_mode_history_keeps_full_captured_output() {
    let local = tokio::task::LocalSet::new();
    local
        .run_until(async {
            let (gateway_tx, _gateway_rx) =
                tokio::sync::mpsc::unbounded_channel::<xai_acp_lib::AcpClientMessage>();
            let (persistence_tx, persistence_rx) =
                tokio::sync::mpsc::unbounded_channel::<PersistenceMsg>();
            drop(persistence_rx);
            let terminal = Arc::new(ManyLineTerminal {
                output_byte_limit: AtomicUsize::new(0),
            });
            let (actor, mut ev) = create_test_actor_with_terminal(
                0,
                256_000,
                85,
                gateway_tx,
                persistence_tx,
                terminal.clone(),
            )
            .await;

            tokio::task::spawn_local(async move {
                while let Some(event) = ev.recv().await {
                    if let SessionEvent::FlushReplay { respond_to } = event
                        && let Some(tx) = respond_to
                    {
                        let _ = tx.send(());
                    }
                }
            });

            let result = actor
                .handle_direct_bash_command(
                    "bash-full-history",
                    "seq 1 15".to_string(),
                    &[acp::ContentBlock::Text(acp::TextContent::new("!seq 1 15"))],
                )
                .await;
            assert!(result.is_ok(), "bash-mode turn should complete: {result:?}");

            assert_eq!(
                terminal.output_byte_limit.load(Ordering::SeqCst),
                1_048_576,
                "bash mode must keep the 1 MiB capture cap"
            );

            let conversation = actor.chat_state_handle.get_conversation().await;
            let history = conversation
                .iter()
                .map(|item| item.text_content())
                .collect::<Vec<_>>()
                .join("\n");
            assert!(
                history.contains("LINE-01"),
                "history must keep the first captured line; got:\n{history}"
            );
            assert!(
                history.contains("LINE-15"),
                "history must keep the last captured line; got:\n{history}"
            );
            assert!(
                !history.contains("... (15 lines)"),
                "history must not use the old last-10 tail marker; got:\n{history}"
            );
        })
        .await;
}
