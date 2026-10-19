//! `otlp-c` — native OTLP backend for `lib-otlp`.
//!
//! `lib-otlp` is the `.hnl` contract (`otlp.*` conn-as-value over `ffi.call`);
//! this crate holds the native impls (moved from `lib-otlp` pre-split).
//! Registers ops as `otlp_c.*` — the `.hnl` `otlp.*` layer calls
//! `ffi.call("otlp_c.*", …)`.

mod error;
mod id;
mod otlp_module;
mod state;
mod stub;

pub use otlp_module::{OTLP_OPS, OtlpModule, register_otlp_ops};
