use bollard::Docker;
use bollard::container::LogOutput;
use bollard::exec::{CreateExecOptions, StartExecResults};
use futures_util::stream::StreamExt;
use serde::Deserialize;
use std::error::Error;
use std::sync::Arc;
use tokio::fs::write;
use tracing::{info, warn};

pub type Result = std::result::Result<(), Box<dyn Error>>;

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

impl CronJob {
    pub async fn exec(&self, docker: Arc<Docker>) -> Result {
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
