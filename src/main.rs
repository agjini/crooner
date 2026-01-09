use crate::config::Config;
use bollard::Docker;
use chrono::Utc;
use cron_tab::AsyncCron;
use signal::unix::SignalKind;
use std::error::Error;
use std::fs;
use std::sync::Arc;
use tokio::signal;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

mod config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    let mut cron = AsyncCron::new(Utc);

    let docker = Arc::new(Docker::connect_with_local_defaults()?);
    info!("Connected to Docker");

    let config_content = fs::read_to_string("config.toml")?;
    let config = toml::from_str::<Config>(&config_content)?;

    info!(count = config.jobs.len(), "Loaded jobs");

    run_at_startup(&docker, &config).await;

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
) -> Result<(), Box<dyn Error>> {
    for job in config.jobs {
        let at = job.at.clone();
        let docker = Arc::clone(docker);

        info!(job = %job.name, cron = %at, "Scheduling job");

        cron.add_fn(&at, move || {
            let job = job.clone();
            let docker = Arc::clone(&docker);
            async move {
                if let Err(e) = job.exec(docker).await {
                    error!(
                        job = %job.name,
                        error = %e,
                        "Error executing job"
                    );
                }
            }
        })
        .await?;
    }
    Ok(())
}

async fn run_at_startup(docker: &Arc<Docker>, config: &Config) {
    let startup_jobs: Vec<_> = config
        .jobs
        .iter()
        .filter(|job| job.run_on_startup)
        .collect();

    if !startup_jobs.is_empty() {
        info!(count = startup_jobs.len(), "Running jobs on startup");
        for job in startup_jobs {
            info!(job = %job.name, "Running startup job");
            if let Err(e) = job.exec(Arc::clone(docker)).await {
                error!(
                    job = %job.name,
                    error = %e,
                    "Error executing startup job"
                );
            }
        }
        info!("Startup jobs completed");
    }
}

async fn register_shutdown_hooks() -> Result<(), Box<dyn Error>> {
    let mut sigint = signal::unix::signal(SignalKind::interrupt())?;
    let mut sigterm = signal::unix::signal(SignalKind::terminate())?;
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
