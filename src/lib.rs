pub mod anyhow_tracing;
pub mod cloudflare;
pub mod config;
pub mod ip_find;
pub mod state;

use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, Write},
    path::PathBuf,
};

use anyhow::{Context, Result};
use anyhow_tracing::Tracing;
use tracing::instrument;

#[derive(Debug)]
pub struct Environment {
    config: File,
    state: File,

    config_text: String,
    state_text: String,
}

impl Environment {
    #[instrument]
    pub fn initialize() -> Result<Self> {
        tracing::trace!("Initializing Environment struct");

        let config_dir = PathBuf::from(
            std::env::var("CONFIGURATION_DIRECTORY")
                .context("failed to read config directory path")
                .debug()
                .debug_success("Succesfully read CONFIGURATION_DIRECTORY env var")
                .error()?,
        );

        let state_dir = PathBuf::from(
            std::env::var("STATE_DIRECTORY")
                .context("failed to read state directory path")
                .debug()
                .debug_success("Succesfully read STATE_DIRECTORY env var")
                .error()?,
        );

        let mut config = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(config_dir.join("config.toml"))
            .with_context(|| {
                format!(
                    "error opening config.toml at `{:?}`",
                    config_dir.join("config.toml")
                )
            })
            .debug()
            .debug_success("Succesfully opened config.toml")
            .error()?;

        let mut state = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(state_dir.join("state.toml"))
            .with_context(|| {
                format!(
                    "error opening state.toml at `{:?}`",
                    state_dir.join("state.toml")
                )
            })
            .debug()
            .debug_success("Succesfully opened state.toml")
            .error()?;

        let config_text = read_file(&mut config)
            .context("failed to read config.toml to text")
            .debug()
            .debug_success("Succesfully read config.toml to text")
            .error()?;
        let state_text = read_file(&mut state)
            .context("failed to read state.toml to text")
            .debug()
            .debug_success("Succesfully read state.toml to text")
            .error()?;

        Ok(Self {
            config,
            state,
            config_text,
            state_text,
        })
    }

    pub fn get_config(&self) -> &str {
        &self.config_text
    }

    pub fn get_state(&self) -> &str {
        &self.state_text
    }

    #[instrument]
    pub fn write_config(&mut self, content: String) -> Result<()> {
        tracing::trace!("Writing to config.toml");

        self.config
            .set_len(0)
            .context("failed to truncate config.toml by setting len to 0")
            .debug()
            .debug_success("Succesfully truncated config.toml")
            .error()?;
        self.config
            .write_all(content.as_bytes())
            .context("failed to write content to config.toml")
            .debug()
            .debug_success("Succesfully wrote content to config.toml")
            .error()?;
        self.config_text = content;
        Ok(())
    }

    #[instrument]
    pub fn write_state(&mut self, content: String) -> Result<()> {
        tracing::trace!("Writing to state.toml");

        self.state
            .set_len(0)
            .context("failed to truncate config.toml by setting len to 0")
            .debug()
            .debug_success("Succesfully truncated state.toml")
            .error()?;
        self.state
            .write_all(content.as_bytes())
            .context("failed to write content to state.toml")
            .debug()
            .debug_success("Succesfully wrote content to state.toml")
            .error()?;
        self.state_text = content;
        Ok(())
    }
}

#[instrument(skip(file))]
fn read_file(file: &mut File) -> Result<String> {
    tracing::trace!("Reading file to string");
    let mut content = String::new();
    file.read_to_string(&mut content)
        .context("failed to read file to string")
        .debug()
        .debug_success("Successfully read file to string")
        .error()?;
    file.rewind()
        .context("failed to rewind file after reading to string")
        .debug()
        .debug_success("Successfully rewound file")
        .error()?;
    Ok(content)
}
