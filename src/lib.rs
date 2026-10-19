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
pub use state::{MetricKind, MetricOptions, OtlpConfig, OtlpState, StartSpanOptions};
pub use error::{OtlpError, OtlpResult};
pub use stub::{OtlpEvent, RecordedEvent, RecordedSpan, StubOtlpState};

/// Build an `OpRegistry` with the `otlp_c.*` ops bound to `state`.
pub fn otlp_op_registry(state: std::sync::Arc<dyn state::OtlpState>) -> handl_plugin_lib_handl::OpRegistry {
    let mut registry = handl_plugin_lib_handl::OpRegistry::new();
    register_otlp_ops(&mut registry, state);
    registry
}

