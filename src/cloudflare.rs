use std::net::Ipv4Addr;

use anyhow::{Context, Result};
use cloudflare::{
    endpoints::dns::dns::DnsContent,
    framework::{Environment, client::async_api::Client},
};
use tracing::instrument;

use crate::{anyhow_tracing::Tracing, config::Config, state::State};

#[instrument(skip(config, state))]
pub async fn update_cloudflare(
    config: &Config,
    state: &mut Option<State>,
    ip: Ipv4Addr,
) -> Result<()> {
    tracing::trace!("Updating Cloudflare");
    let cf_config = config.get_cloudflare_config();
    let cache_config = config.get_cache_config();

    let client = Client::new(
        cf_config.get_creds(),
        Default::default(),
        Environment::Production,
    )
    .context("failed to create new `cloudflare` client")
    .debug()
    .debug_success("Successfully created new `cloudflare` client")
    .error()?;

    let response = client
        .request(&cf_config.get_list_request())
        .await
        .context("failed to request DNS record list from Cloudflare")
        .debug()
        .debug_success("Successfully got DNS record list from Cloudflare")
        .error()?;

    let mut records = response
        .result
        .iter()
        .filter(|record| matches!(record.content, DnsContent::A { .. }));

    let record = if let Some(record) = records.next() {
        if records.next().is_some() {
            anyhow::bail!(
                "multiple A records retrieved for {}, case is ambiguous",
                cf_config.get_record_name()
            )
        }

        record
    } else {
        anyhow::bail!(
            "failed to find any A records for {}",
            cf_config.get_record_name()
        );
    };

    client
        .request(&cf_config.get_update_request(record, ip))
        .await
        .context("failed to update DNS record on Cloudflare")
        .debug()
        .debug_success("Successfully updated DNS record on Cloudflare")
        .error()?;

    if cache_config.get_persist() {
        if let Some(State { last_sent_ip, .. }) = state {
            *last_sent_ip = Some(ip);
        } else {
            *state = Some(State {
                last_sent_ip: Some(ip),
            });
        }
    }

    Ok(())
}
