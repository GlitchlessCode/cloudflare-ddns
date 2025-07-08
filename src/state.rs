use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    pub(crate) last_sent_ip: Option<Ipv4Addr>,
}
