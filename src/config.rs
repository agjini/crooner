use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::Docker;
use futures_util::stream::StreamExt;
use serde::Deserialize;
use std::error::Error;
use std::sync::Arc;
use tracing::{info, warn};

#[derive(Deserialize, Clone)]
pub struct CronJob {
    pub(crate) name: String,
    pub(crate) at: String,
    container: String,
    command: Vec<String>,
    #[serde(default)]
    output_file: Option<String>,
    #[serde(default)]
    pub(crate) run_on_startup: bool,
}

impl CronJob {
    pub(crate) async fn exec(&self, docker: Arc<Docker>) -> Result<(), Box<dyn Error>> {
        info!(
            container = %self.container,
            command = ?self.command,
            "Executing job in container"
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
            let mut stdout_data = Vec::new();
            let mut stderr_data = Vec::new();

            while let Some(Ok(msg)) = output.next().await {
                match msg {
                    bollard::container::LogOutput::StdOut { message } => {
                        stdout_data.extend_from_slice(&message);
                        print!("{}", String::from_utf8_lossy(&message));
                    }
                    bollard::container::LogOutput::StdErr { message } => {
                        stderr_data.extend_from_slice(&message);
                        eprint!("{}", String::from_utf8_lossy(&message));
                    }
                    _ => {}
                }
            }

            if let Some(output_path) = &self.output_file {
                tokio::fs::write(output_path, &stdout_data).await?;
                info!(path = %output_path, "Output saved to file");
            }

            if !stderr_data.is_empty() {
                warn!("Command completed with errors");
            } else {
                info!("Command completed successfully");
            }
        }

        Ok(())
    }
}

#[derive(Deserialize)]
pub struct Config {
    pub(crate) jobs: Vec<CronJob>,
}
