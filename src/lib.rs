//! `otlp-c` — native OTLP backend for `lib-otlp`.
//!
//! `lib-otlp` is the `.hnl` contract (`otlp.*` conn-as-value over `ffi.call`);
//! this crate holds the native impls (moved from `lib-otlp` pre-split).
//! Registers ops as `otlp_c.*` — the `.hnl` `otlp.*` layer calls
//! `ffi.call("otlp_c.*", …)`.
//!
//! The ops are pure [`handl_ops::Op`]s behind [`otlp_op_registry`] — a
//! kernel [`handl_ops::OpRegistry`] the host imports via
//! `OpRegistry::import_kernel`. The former `OtlpModule` host wrapper lives
//! in `handl-plugin-lib-stdlib`.

mod error;
pub mod id;
mod otlp_module;
mod state;
mod stub;

pub use error::{OtlpError, OtlpResult};
pub use otlp_module::{OTLP_OPS, register_otlp_ops};
pub use state::{MetricKind, MetricOptions, OtlpConfig, OtlpState, StartSpanOptions};
pub use stub::{OtlpEvent, RecordedEvent, RecordedSpan, StubOtlpState};

/// Build a [`handl_ops::OpRegistry`] with the `otlp_c.*` ops bound to
/// `state`.
pub fn otlp_op_registry(state: std::sync::Arc<dyn state::OtlpState>) -> handl_ops::OpRegistry {
    let mut registry = handl_ops::OpRegistry::new();
    register_otlp_ops(&mut registry, state);
    registry
}

/// The handl contract for `otlp_c.*` ops.
pub const OTLP_C_HNL: &str = include_str!("../handl-packages/otlp/otlp.hnl");
