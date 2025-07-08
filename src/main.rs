use anyhow::{Context, Result};
use cloudflare_ddns::{
    Environment,
    anyhow_tracing::Tracing,
    cloudflare::update_cloudflare,
    config::Config,
    ip_find::{IpResult, find_public_ip},
    state::State,
};
use tokio::{signal::unix::Signal, time::Instant};
use tracing_subscriber::filter::LevelFilter;

#[derive(PartialEq)]
enum LogLevel {
    Default,

    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    fn from_string(source: String) -> Option<Self> {
        match source.to_lowercase().as_str() {
            "trace" => Some(Self::Trace),
            "debug" => Some(Self::Debug),
            "info" => Some(Self::Info),
            "warn" => Some(Self::Warn),
            "error" => Some(Self::Error),
            _ => None,
        }
    }

    fn log_level(&self) -> &'static str {
        match self {
            Self::Default => "INFO",

            Self::Trace => "TRACE",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
        }
    }
}

impl From<&LogLevel> for LevelFilter {
    fn from(value: &LogLevel) -> Self {
        match value {
            LogLevel::Default => Self::INFO,

            LogLevel::Trace => Self::TRACE,
            LogLevel::Debug => Self::DEBUG,
            LogLevel::Info => Self::INFO,
            LogLevel::Warn => Self::WARN,
            LogLevel::Error => Self::ERROR,
        }
    }
}

#[tracing::instrument]
fn setup_tracing() {
    let level = match std::env::var("LOG_LEVEL") {
        Ok(source) => LogLevel::from_string(source).unwrap_or(LogLevel::Default),
        Err(_) => LogLevel::Default,
    };

    tracing_subscriber::fmt().with_max_level(&level).init();

    std::panic::set_hook(Box::new(tracing_panic::panic_hook));

    if level == LogLevel::Default {
        tracing::info!("Using default log level");
    }

    tracing::info!("Log level set to {:?}", level.log_level());
}

#[tracing::instrument]
fn setup_signals() -> Result<(Signal, Signal)> {
    use tokio::signal::unix::{SignalKind, signal};
    let sigterm = signal(SignalKind::terminate()).context("Error creating SIGTERM handler")?;
    let sigint = signal(SignalKind::interrupt()).context("Error creating SIGINT handler")?;

    Ok((sigterm, sigint))
}

#[tokio::main]
async fn main() {
    setup_tracing();

    tracing::trace!("Setting up signals");
    let (mut sigterm, mut sigint) = match setup_signals() {
        Ok(signals) => signals,
        Err(err) => {
            panic!("{err:?}")
        }
    };

    tracing::trace!("Starting main future selection");
    let start = Instant::now();
    let interrupted = tokio::select! {
        result = run_service() => {
            let duration = Instant::now().duration_since(start);
            if let Err(err) = result.with_context(|| format!("Service failed after {}ms", duration.as_millis())) {
                tracing::error!("{err:?}");
                std::process::exit(1)
            }
            false
        },
        _ = sigterm.recv() => {
            tracing::debug!("SIGTERM signal triggered");
            true
        },
        _ = sigint.recv() => {
            tracing::debug!("SIGINT signal triggered");
            true
        },
    };

    tracing::debug!(interrupted = interrupted, "Main process future finished,");

    if interrupted {
        tracing::warn!("Recieved termination signal, shutting down")
    }
    tracing::info!("Service shutting down, goodbye!");
}

#[tracing::instrument]
async fn run_service() -> Result<()> {
    tracing::trace!("Running service");
    let mut env = Environment::initialize()
        .context("failed to initialize Environment")
        .error()?;

    let config: Config = match toml::from_str(env.get_config())
        .context("failed to read config, creating default")
        .warn()
    {
        Ok(config) => config,
        Err(_) => {
            let config = Config::default();
            env.write_config(
                toml::to_string_pretty(&config).expect("failed to serialize default Config"),
            )
            .context("failed to write default config.toml")
            .error()?;
            config
        }
    };

    let mut state: Option<State> = toml::from_str(env.get_state())
        .context("failed to read state, defaulting to `None`")
        .warn()
        .ok();

    if !config.is_active() {
        tracing::info!("Config setting `active` is false, make sure to set `active` to true");
        return Ok(());
    }

    tracing::info!("Searching for public IPv4 address...");
    let ip = match find_public_ip(&config, &state).await {
        IpResult::Found(ip) => ip,
        IpResult::MatchesCache => {
            tracing::info!("IP matched previously cached IP");
            tracing::info!(
                "NOTE: You can ignore the cache using the `ignore` key in the `cache` settings"
            );
            return Ok(());
        }
        IpResult::NotFound => {
            anyhow::bail!("Failed to find public IPv4 address, all provided finders failed");
        }
    };

    tracing::debug!("Found new IPv4: {ip}");

    tracing::info!("Updating Cloudflare DNS Record...");
    update_cloudflare(&config, &mut state, ip).await?;

    tracing::info!("Successfully updated Cloudflare DNS Record...");

    if state.is_some() {
        env.write_state(toml::to_string_pretty(&state).expect("failed to serialize State"))
            .context("failed to write default config.toml")
            .error()?;
    }

    Ok(())
}
