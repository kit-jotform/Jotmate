use anyhow::Result;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, ChildStderr, ChildStdout};
use tokio::sync::mpsc;

use crate::tui::events::AppEvent;

/// Streams stdout and stderr from a child process to the AppEvent channel.
/// Sends `CommandFinished` when the process exits.
pub async fn stream_command_output(
    mut child: Child,
    tx: mpsc::Sender<AppEvent>,
) -> Result<()> {
    let stdout: ChildStdout = child.stdout.take().expect("stdout not piped");
    let stderr: ChildStderr = child.stderr.take().expect("stderr not piped");

    let tx_out = tx.clone();
    let tx_err = tx.clone();

    let stdout_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stdout).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if tx_out.send(AppEvent::OutputLine(line)).await.is_err() {
                break;
            }
        }
    });

    let stderr_task = tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if tx_err.send(AppEvent::OutputLine(line)).await.is_err() {
                break;
            }
        }
    });

    let status = child.wait().await?;
    let _ = stdout_task.await;
    let _ = stderr_task.await;

    let code = status.code().unwrap_or(1);
    let _ = tx.send(AppEvent::CommandFinished(code)).await;

    Ok(())
}
