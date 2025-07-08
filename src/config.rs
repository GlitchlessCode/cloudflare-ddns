use std::net::Ipv4Addr;

use anyhow::Context;
use cloudflare::{
    endpoints::dns::dns::{
        DnsContent, DnsRecord, ListDnsRecords, ListDnsRecordsParams, UpdateDnsRecord,
        UpdateDnsRecordParams,
    },
    framework::auth::Credentials,
};
use serde::{Deserialize, Serialize};

use crate::anyhow_tracing::Tracing;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    active: bool,
    #[serde(rename = "ip-find")]
    ip_find: IpFindConfig,
    cloudflare: CloudflareConfig,
    #[serde(default)]
    cache: CacheConfig,
}

impl Config {
    pub fn is_active(&self) -> bool {
        self.active
    }

    pub(crate) fn get_ip_config(&self) -> &IpFindConfig {
        &self.ip_find
    }

    pub(crate) fn get_cloudflare_config(&self) -> &CloudflareConfig {
        &self.cloudflare
    }

    pub(crate) fn get_cache_config(&self) -> &CacheConfig {
        &self.cache
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub(crate) struct IpFindConfig {
    finders: Vec<String>,
    retries: Option<u8>,
    timeout: Option<u8>,
}

impl IpFindConfig {
    pub(crate) fn iter(&self) -> std::slice::Iter<'_, String> {
        self.finders.iter()
    }

    pub(crate) fn get_retries(&self) -> u8 {
        self.retries
            .context("`ip-find` config key `retries` is `None`, defaulting to 0")
            .debug()
            .unwrap_or(0)
    }

    pub(crate) fn get_timeout(&self) -> u8 {
        self.timeout
            .context("`ip-find` config key `timeout` is `None`, defaulting to 1")
            .debug()
            .unwrap_or(1)
    }
}

#[derive(Serialize, Deserialize, Default)]
pub(crate) struct CloudflareConfig {
    #[serde(rename = "api-key")]
    api_key: String,
    #[serde(rename = "zone-identifier")]
    zone_id: String,
    #[serde(rename = "dns-record-name")]
    record_name: String,
}

impl CloudflareConfig {
    pub(crate) fn get_creds(&self) -> Credentials {
        Credentials::UserAuthToken {
            token: self.api_key.clone(),
        }
    }

    pub(crate) fn get_list_request(&self) -> ListDnsRecords {
        ListDnsRecords {
            zone_identifier: &self.zone_id,
            params: ListDnsRecordsParams {
                name: Some(self.record_name.clone()),
                ..Default::default()
            },
        }
    }

    pub(crate) fn get_update_request<'a>(
        &'a self,
        record: &'a DnsRecord,
        ip: Ipv4Addr,
    ) -> UpdateDnsRecord<'a> {
        UpdateDnsRecord {
            zone_identifier: &self.zone_id,
            identifier: &record.id,
            params: UpdateDnsRecordParams {
                ttl: Some(record.ttl),
                proxied: Some(record.proxied),
                name: &record.name,
                content: DnsContent::A { content: ip },
            },
        }
    }

    pub(crate) fn get_record_name(&self) -> &str {
        &self.record_name
    }
}

impl std::fmt::Debug for CloudflareConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CloudflareConfig {{ REDACTED }}")
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub(crate) struct CacheConfig {
    ignore: Option<bool>,
    persist: Option<bool>,
}

impl CacheConfig {
    pub(crate) fn get_ignore(&self) -> bool {
        self.ignore
            .context("`cache` config key `ignore` is `None`, defaulting to false")
            .debug()
            .unwrap_or(false)
    }

    pub(crate) fn get_persist(&self) -> bool {
        self.persist
            .context("`cache` config key `persist` is `None`, defaulting to true")
            .debug()
            .unwrap_or(true)
    }
}
