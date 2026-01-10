use cron_tab::CronError;
use std::fmt::{Display, Formatter};

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    Docker(String),
    Io(String),
    Toml(String),
    Cron(String),
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let reason = match self {
            AppError::Docker(s) | AppError::Io(s) | AppError::Toml(s) | AppError::Cron(s) => s,
        };
        f.write_fmt(format_args!("{:?} reason : {}", self, reason))
    }
}

impl From<toml::de::Error> for AppError {
    fn from(value: toml::de::Error) -> Self {
        Self::Toml(value.to_string())
    }
}

impl From<bollard::errors::Error> for AppError {
    fn from(value: bollard::errors::Error) -> Self {
        Self::Docker(value.to_string())
    }
}

impl From<tokio::io::Error> for AppError {
    fn from(value: tokio::io::Error) -> Self {
        Self::Io(value.to_string())
    }
}

impl From<CronError> for AppError {
    fn from(value: CronError) -> Self {
        Self::Cron(value.to_string())
    }
}
