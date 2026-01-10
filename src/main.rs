use crate::config::Config;
use crate::error::Result;
use bollard::Docker;
use chrono::Utc;
use cron_tab::AsyncCron;
use signal::unix::{signal, SignalKind};
use std::fs;
use std::sync::Arc;
use tokio::signal;
use tracing::{info, Level};
use tracing_subscriber::EnvFilter;

mod config;
mod error;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(Level::INFO.into()))
        .init();

    let docker = Arc::new(Docker::connect_with_local_defaults()?);
    info!("Connected to Docker");

    let config_content = fs::read_to_string("config.toml")?;
    let config = toml::from_str::<Config>(&config_content)?;

    info!(count = config.jobs.len(), "Loaded jobs");

    run_at_startup(&docker, &config).await;

    let mut cron = AsyncCron::new(Utc);

    schedule_jobs(&mut cron, &docker, config).await?;

    info!("Starting cron scheduler");
    cron.start().await;

    info!("Crooner is running. Press Ctrl+C to stop.");

    register_shutdown_hooks().await?;

    info!("Stopping cron scheduler");
    cron.stop().await;
    info!("Goodbye!");
    Ok(())
}

async fn schedule_jobs(
    cron: &mut AsyncCron<Utc>,
    docker: &Arc<Docker>,
    config: Config,
) -> Result<()> {
    for job in config.jobs {
        let at = job.at.clone();
        let docker = Arc::clone(docker);

        info!(job = %job.name, cron = %at, "Scheduling job");

        cron.add_fn(&at, move || {
            let job = job.clone();
            let docker = Arc::clone(&docker);
            async move {
                job.exec(&docker).await;
            }
        })
        .await?;
    }
    Ok(())
}

async fn run_at_startup(docker: &Arc<Docker>, config: &Config) {
    let startup_jobs: Vec<_> = config.jobs.iter().filter(|j| j.run_on_startup).collect();

    if !startup_jobs.is_empty() {
        info!(count = startup_jobs.len(), "Running jobs on startup");
        for job in startup_jobs {
            info!(job = %job.name, "Running startup job");
            job.exec_and_retry(&Arc::clone(docker)).await;
        }
        info!("Startup jobs completed");
    }
}

async fn register_shutdown_hooks() -> Result<()> {
    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;
    tokio::select! {
        _ = sigint.recv() => {
            info!("Received SIGINT (Ctrl+C)");
        }
        _ = sigterm.recv() => {
            info!("Received SIGTERM");
        }
    }
    Ok(())
}
