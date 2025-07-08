use std::{net::Ipv4Addr, time::Duration};

use anyhow::Context;
use reqwest::{Client, Url};
use tracing::instrument;

use crate::{anyhow_tracing::Tracing, config::Config, state::State};

pub enum IpResult {
    Found(Ipv4Addr),
    MatchesCache,
    NotFound,
}

#[instrument(skip(config, state))]
pub async fn find_public_ip(config: &Config, state: &Option<State>) -> IpResult {
    tracing::trace!("Finding public IP");
    let ip_config = config.get_ip_config();
    let retries = ip_config.get_retries();
    tracing::debug!("Using {retries} retries");
    let timeout = Duration::from_secs(ip_config.get_timeout() as u64);
    tracing::debug!("Using {timeout:?} timeout");
    let client = Client::new();

    let cache_config = config.get_cache_config();
    let ignore_cache = cache_config.get_ignore();

    for url in ip_config.iter().filter_map(|try_url| {
        Url::parse(try_url)
            .with_context(|| format!("failed to parse url from `{try_url}`"))
            .error()
            .ok()
    }) {
        tracing::debug!("Trying {url}");
        if let Some(ip) = try_url(&client, &url, retries, timeout).await {
            if ignore_cache {
                return IpResult::Found(ip);
            } else if let Some(State {
                last_sent_ip: Some(cached_ip),
            }) = state
            {
                if cached_ip == &ip {
                    return IpResult::MatchesCache;
                } else {
                    return IpResult::Found(ip);
                }
            } else {
                return IpResult::Found(ip);
            }
        } else {
            tracing::warn!("Finder `{url}` failed, trying next finder");
        }
    }

    tracing::error!("No more finders to try. Failed to find public IPv4 address.");
    IpResult::NotFound
}

#[instrument(skip(client, url, retries, timeout))]
async fn try_url(client: &Client, url: &Url, retries: u8, timeout: Duration) -> Option<Ipv4Addr> {
    tracing::trace!("Trying a URL");
    for attempt in 0..=retries {
        let final_attempt = attempt == retries;
        if let Ok(response) = client
            .get(url.clone())
            .timeout(timeout)
            .send()
            .await
            .with_context(|| format!("failed on attempt {} for `{url}`", attempt + 1))
            .warn_or_error(final_attempt)
        {
            tracing::debug!("Got a Response from {url} on attempt {}", attempt + 1);
            if let Ok(response) = response
                .error_for_status()
                .context("server responded with an error")
                .warn_or_error(final_attempt)
            {
                tracing::debug!("Server responded with success");
                if let Ok(text) = response
                    .text()
                    .await
                    .context("failed to fetch text from successful response")
                    .warn_or_error(final_attempt)
                {
                    let text = text.trim();
                    tracing::debug!("Server responded with `{text}`");
                    if let Ok(ip) = text.parse() {
                        return Some(ip);
                    }
                }
            }
        }
    }
    None
}
