use serde::Serialize;

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionState {
    Connected,
    Connecting,
    Disconnected,
    Disconnecting,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Inbound,
    Outbound,
    Unknown,
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct PeerCount {
    #[serde(with = "serde_utils::quoted_u64")]
    pub disconnected: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    pub connecting: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    pub connected: u64,
    #[serde(with = "serde_utils::quoted_u64")]
    pub disconnecting: u64,
}
