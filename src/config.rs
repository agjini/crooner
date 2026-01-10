use crate::error::Result;
use bollard::container::LogOutput;
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::Docker;
use futures_util::stream::StreamExt;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::write;
use tokio::time::sleep;
use tracing::{error, info, warn};

#[derive(Deserialize, Clone)]
pub struct CronJob {
    pub name: String,
    pub at: String,
    pub container: String,
    pub command: Vec<String>,
    #[serde(default)]
    pub output_file: Option<String>,
    #[serde(default)]
    pub run_on_startup: bool,
}

const MAX_RETRIES: u32 = 5;
const RETRY_DELAY_MS: Duration = Duration::from_secs(2);

impl CronJob {
    pub async fn exec_and_retry(&self, docker: &Arc<Docker>) {
        let mut retry_count = 0;

        while let Err(e) = self.try_exec(docker).await {
            error!(
                job = %self.name,
                error = %e,
                "Error executing job"
            );
            retry_count += 1;
            if retry_count == MAX_RETRIES {
                error!(
                    job = %self.name,
                    "Fails executing job after {} retries",
                        MAX_RETRIES
                );
                break;
            }
            sleep(RETRY_DELAY_MS).await;
        }
    }

    pub async fn exec(&self, docker: &Arc<Docker>) {
        if let Err(e) = self.try_exec(docker).await {
            error!(
                job = %self.name,
                error = %e,
                "Error executing job"
            );
        }
    }

    pub async fn try_exec(&self, docker: &Docker) -> Result<()> {
        info!(
            job = %self.name,
            container = %self.container,
            command = ?self.command,
            "Executing job in container",
        );

        let exec = docker
            .create_exec(
                &self.container,
                CreateExecOptions {
                    cmd: Some(self.command.clone()),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    ..Default::default()
                },
            )
            .await?;

        if let StartExecResults::Attached { mut output, .. } =
            docker.start_exec(&exec.id, None).await?
        {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();

            while let Some(Ok(msg)) = output.next().await {
                match msg {
                    LogOutput::StdOut { message } => {
                        stdout.extend_from_slice(&message);
                    }
                    LogOutput::StdErr { message } => {
                        stderr.extend_from_slice(&message);
                        warn!(job = %self.name, "{}", String::from_utf8_lossy(&message));
                    }
                    _ => {}
                }
            }

            if let Some(output_path) = &self.output_file {
                write(output_path, &stdout).await?;
                info!(job = %self.name, path = %output_path, "Output saved to file");
            }

            if !stderr.is_empty() {
                warn!(job = %self.name, "Command completed with errors");
            } else {
                info!(job = %self.name, "Command completed successfully");
            }
        }

        Ok(())
    }
}

#[derive(Deserialize)]
pub struct Config {
    pub jobs: Vec<CronJob>,
}
