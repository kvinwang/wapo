use scale::{Decode, Encode};
use wapod_crypto_types::query::{RootOrCertificate, Signature};

pub type QuerySignature = Signature<RootOrCertificate>;

pub type Address = [u8; 32];
pub type Bytes32 = [u8; 32];

#[derive(Debug, Encode, Decode)]
pub struct AppMetrics {
    pub address: Address,
    pub session: Bytes32,
    pub running_time_ms: u64,
    pub gas_consumed: u64,
    pub network_ingress: u64,
    pub network_egress: u64,
    pub storage_read: u64,
    pub storage_write: u64,
    pub starts: u64,
}

#[derive(Debug, Encode, Decode)]
pub struct AppsMetrics {
    pub session: Bytes32,
    pub nonce: Bytes32,
    pub apps: Vec<AppMetrics>,
}
