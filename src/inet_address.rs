use derive_setters::Setters;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Setters)]
pub struct InetAddress {
    pub host: String,
    pub port: u16,
}

impl InetAddress {
    pub fn new(host: String, port: u16) -> InetAddress {
        InetAddress { host, port }
    }
}

impl Display for InetAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.host, self.port)
    }
}
