//! This crate provides some instrumentation to write wapo programs. It is built on top of the
//! wapo ocalls.

#![deny(missing_docs)]

pub use res_id::ResourceId;
pub use wapo_env as env;
pub use wapo_macro::main;

pub use env::spawn;
pub use env::tasks as task;
pub use env::ocall_funcs_guest as ocall;

pub mod channel;
pub mod hyper_rt;
pub mod net;
pub mod time;
pub mod logger;

mod res_id;
