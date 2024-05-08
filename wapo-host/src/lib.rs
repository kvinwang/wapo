mod module_loader;
#[cfg(feature = "rocket-stream")]
pub mod rocket_stream;
mod run;
mod runtime;

pub mod service;
pub use runtime::blobs;
pub use runtime::metrics::{Meter, Metrics};
pub use runtime::vm_context::{
    crate_outgoing_request_channel, vm_count, OutgoingRequest, OutgoingRequestReceiver,
    OutgoingRequestSender, ShortId,
};

pub type VmId = [u8; 32];
pub use run::{InstanceConfig, InstanceConfigBuilder, WasmEngine, WasmModule, WasmRun};
pub use wasmtime;

pub use service::IncomingHttpRequest;
pub use wapo_env::OcallError;
