use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

#[derive(Default, Debug)]
pub struct Meter {
    pub gas_comsumed: AtomicU64,
    pub net_egress: AtomicU64,
    pub net_ingress: AtomicU64,
    pub storage_read: AtomicU64,
    pub storage_written: AtomicU64,
    /// Whether the metering is stopped. Used to signal the epoch checker to stop the VM.
    pub stopped: AtomicBool,
}

impl Meter {
    pub fn set_gas_comsumed(&self, gas: u64) {
        self.gas_comsumed.store(gas, Ordering::Relaxed);
    }

    pub fn record_gas(&self, gas: u64) {
        self.gas_comsumed.fetch_add(gas, Ordering::Relaxed);
    }

    pub fn record_net_egress(&self, bytes: u64) {
        self.net_egress.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn record_net_ingress(&self, bytes: u64) {
        self.net_ingress.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn record_storage_read(&self, bytes: u64) {
        self.storage_read.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn record_storage_written(&self, bytes: u64) {
        self.storage_written.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn record_tcp_connect_start(&self) {
        self.record_net_egress(512);
    }
    pub fn record_tls_connect_start(&self) {
        self.record_net_egress(4096);
    }
    pub fn record_tcp_connect_done(&self) {
        self.record_net_ingress(512);
    }
    pub fn record_tls_connect_done(&self) {
        self.record_net_ingress(4096);
    }
    pub fn record_tcp_shutdown(&self) {
        self.record_net_egress(128);
    }
    pub fn stop(&self) {
        self.stopped.store(true, Ordering::Relaxed)
    }
    pub fn stopped(&self) -> bool {
        self.stopped.load(Ordering::Relaxed)
    }
}
