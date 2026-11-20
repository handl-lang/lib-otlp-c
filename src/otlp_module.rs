use std::collections::HashMap;
use std::sync::Arc;

use handl_ops::{Op, OpContext, OpError, OpRegistry};
use serde_json::Value;

use crate::state::{MetricKind, MetricOptions, OtlpConfig, OtlpState, StartSpanOptions};

/// Op table for the `otlp_c.*` interface — see [`OTLP_OPS`].
///
/// | Op | Args | Description |
/// |----|------|-------------|
/// | `otlp_c.configure` | `(config)` | Set endpoint, service name, headers and resource attributes. |
/// | `otlp_c.start_span` | `(name, options?)` | Start a span and return its id. |
/// | `otlp_c.end_span` | `(id)` | End a span. |
/// | `otlp_c.set_attribute` | `(id, key, value)` | Set a span attribute. |
/// | `otlp_c.add_event` | `(id, name, attrs?)` | Add a named event to a span. |
/// | `otlp_c.record_log` | `(level, message, attrs?)` | Record a log entry. |
/// | `otlp_c.record_metric` | `(name, value, options?)` | Record a numeric metric. |
/// | `otlp_c.flush` | `()` | Export all buffered telemetry. |
/// | `otlp_c.shutdown` | `()` | End all spans, flush and release the exporter. |
///
/// The interface is declared in `otlp.hnl` via `op`/`handl` declarations;
/// the host builds the module and imports this kernel registry via
/// `OpRegistry::import_kernel`. The former `OtlpModule` host wrapper lives
/// in `handl-plugin-lib-stdlib` now.

fn require_string_at(args: &[serde_json::Value], idx: usize) -> Result<&str, OpError> {
    args.get(idx)
        .ok_or(OpError::TypeMismatch(format!(
            "expected {} args, got {}",
            idx + 1,
            args.len(),
        )))?
        .as_str()
        .ok_or(OpError::TypeMismatch(format!(
            "expected {}, got {}",
            "string", "non-string",
        )))
}

fn require_number_at(args: &[serde_json::Value], idx: usize) -> Result<f64, OpError> {
    args.get(idx)
        .ok_or(OpError::TypeMismatch(format!(
            "expected {} args, got {}",
            idx + 1,
            args.len(),
        )))?
        .as_f64()
        .ok_or(OpError::TypeMismatch(format!(
            "expected {}, got {}",
            "number", "non-number",
        )))
}

fn require_object_at(
    args: &[serde_json::Value],
    idx: usize,
) -> Result<&serde_json::Map<String, Value>, OpError> {
    args.get(idx)
        .ok_or(OpError::TypeMismatch(format!(
            "expected {} args, got {}",
            idx + 1,
            args.len(),
        )))?
        .as_object()
        .ok_or(OpError::TypeMismatch(format!(
            "expected {}, got {}",
            "object", "non-object",
        )))
}

fn optional_object_at(
    args: &[serde_json::Value],
    idx: usize,
) -> Option<&serde_json::Map<String, Value>> {
    args.get(idx).and_then(|v| v.as_object())
}

fn get_string<'a>(obj: &'a serde_json::Map<String, Value>, key: &str) -> Option<&'a str> {
    obj.get(key).and_then(|v| v.as_str())
}

fn value_map_to_hashmap(map: &serde_json::Map<String, Value>) -> HashMap<String, Value> {
    map.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
}

fn parse_headers(map: &serde_json::Map<String, Value>) -> Result<HashMap<String, String>, OpError> {
    map.iter()
        .map(|(k, v)| {
            v.as_str()
                .map(|s| (k.clone(), s.to_string()))
                .ok_or(OpError::TypeMismatch(format!(
                    "expected {}, got {}",
                    "string", "non-string",
                )))
        })
        .collect()
}

fn parse_config(args: &[serde_json::Value]) -> Result<OtlpConfig, OpError> {
    let obj = require_object_at(args, 0)?;
    let endpoint = get_string(obj, "endpoint").unwrap_or("").to_string();
    let service_name = get_string(obj, "service_name")
        .unwrap_or("handl")
        .to_string();
    let resource_attributes = obj
        .get("resource_attributes")
        .and_then(|v| v.as_object())
        .map(value_map_to_hashmap)
        .unwrap_or_default();
    let headers = obj
        .get("headers")
        .and_then(|v| v.as_object())
        .map(parse_headers)
        .transpose()?
        .unwrap_or_default();
    Ok(OtlpConfig {
        endpoint,
        service_name,
        resource_attributes,
        headers,
    })
}

fn parse_attributes(obj: &serde_json::Map<String, Value>, key: &str) -> HashMap<String, Value> {
    obj.get(key)
        .and_then(|v| v.as_object())
        .map(value_map_to_hashmap)
        .unwrap_or_default()
}

struct ConfigureFn {
    state: Arc<dyn OtlpState>,
}

impl Op for ConfigureFn {
    fn call(
        &self,
        args: &[serde_json::Value],
        _ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        let config = parse_config(args)?;
        self.state.configure(config).map_err(OpError::from)?;
        Ok(serde_json::Value::from(Value::Null))
    }
}

struct StartSpanFn {
    state: Arc<dyn OtlpState>,
}

impl Op for StartSpanFn {
    fn call(
        &self,
        args: &[serde_json::Value],
        _ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        let name = require_string_at(args, 0)?;
        let mut options = StartSpanOptions::default();
        if let Some(obj) = optional_object_at(args, 1) {
            options.parent = get_string(obj, "parent").map(|s| s.to_string());
            options.attributes = parse_attributes(obj, "attributes");
        }
        let id = self
            .state
            .start_span(name, options)
            .map_err(OpError::from)?;
        Ok(serde_json::Value::from(id))
    }
}

struct EndSpanFn {
    state: Arc<dyn OtlpState>,
}

impl Op for EndSpanFn {
    fn call(
        &self,
        args: &[serde_json::Value],
        _ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        let id = require_string_at(args, 0)?;
        self.state.end_span(id).map_err(OpError::from)?;
        Ok(serde_json::Value::from(Value::Null))
    }
}

struct SetAttributeFn {
    state: Arc<dyn OtlpState>,
}

impl Op for SetAttributeFn {
    fn call(
        &self,
        args: &[serde_json::Value],
        _ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        let id = require_string_at(args, 0)?;
        let key = require_string_at(args, 1)?;
        let value = args
            .get(2)
            .ok_or(OpError::TypeMismatch(format!(
                "expected {} args, got {}",
                3,
                args.len(),
            )))?
            .clone();
        self.state
            .set_attribute(id, key, value)
            .map_err(OpError::from)?;
        Ok(serde_json::Value::from(Value::Null))
    }
}

struct AddEventFn {
    state: Arc<dyn OtlpState>,
}

impl Op for AddEventFn {
    fn call(
        &self,
        args: &[serde_json::Value],
        _ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        let id = require_string_at(args, 0)?;
        let name = require_string_at(args, 1)?;
        let attributes = args
            .get(2)
            .and_then(|v| v.as_object())
            .map(value_map_to_hashmap)
            .unwrap_or_default();
        self.state
            .add_event(id, name, attributes)
            .map_err(OpError::from)?;
        Ok(serde_json::Value::from(Value::Null))
    }
}

struct RecordLogFn {
    state: Arc<dyn OtlpState>,
}

impl Op for RecordLogFn {
    fn call(
        &self,
        args: &[serde_json::Value],
        _ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        let level = require_string_at(args, 0)?;
        let message = require_string_at(args, 1)?;
        let attributes = args
            .get(2)
            .and_then(|v| v.as_object())
            .map(value_map_to_hashmap)
            .unwrap_or_default();
        self.state
            .record_log(level, message, attributes)
            .map_err(OpError::from)?;
        Ok(serde_json::Value::from(Value::Null))
    }
}

struct RecordMetricFn {
    state: Arc<dyn OtlpState>,
}

impl Op for RecordMetricFn {
    fn call(
        &self,
        args: &[serde_json::Value],
        _ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        let name = require_string_at(args, 0)?;
        let value = require_number_at(args, 1)?;
        let mut options = MetricOptions::default();
        if let Some(obj) = optional_object_at(args, 2) {
            options.kind = get_string(obj, "kind")
                .and_then(MetricKind::parse)
                .unwrap_or(MetricKind::Counter);
            options.attributes = parse_attributes(obj, "attributes");
        }
        self.state
            .record_metric(name, value, options)
            .map_err(OpError::from)?;
        Ok(serde_json::Value::from(Value::Null))
    }
}

struct FlushFn {
    state: Arc<dyn OtlpState>,
}

impl Op for FlushFn {
    fn call(
        &self,
        _args: &[serde_json::Value],
        _ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        self.state.flush().map_err(OpError::from)?;
        Ok(serde_json::Value::from(Value::Null))
    }
}

struct ShutdownFn {
    state: Arc<dyn OtlpState>,
}

impl Op for ShutdownFn {
    fn call(
        &self,
        _args: &[serde_json::Value],
        _ctx: &mut dyn OpContext,
    ) -> Result<serde_json::Value, OpError> {
        self.state.shutdown().map_err(OpError::from)?;
        Ok(serde_json::Value::from(Value::Null))
    }
}

// ---------------------------------------------------------------------------
// Op Registry (ADR-0077 Phase 4)
// ---------------------------------------------------------------------------

/// The op names declared in `otlp.hnl` — the OTLP interface.
pub const OTLP_OPS: &[&str] = &[
    "otlp_c.configure",
    "otlp_c.start_span",
    "otlp_c.end_span",
    "otlp_c.set_attribute",
    "otlp_c.add_event",
    "otlp_c.record_log",
    "otlp_c.record_metric",
    "otlp_c.flush",
    "otlp_c.shutdown",
];

/// Register the `otlp` ops into the given [`OpRegistry`], dispatching to
/// the given [`OtlpState`] implementation.
pub fn register_otlp_ops(registry: &mut OpRegistry, state: Arc<dyn OtlpState>) {
    registry.register(
        "otlp_c.configure",
        Arc::new(ConfigureFn {
            state: state.clone(),
        }),
    );
    registry.register(
        "otlp_c.start_span",
        Arc::new(StartSpanFn {
            state: state.clone(),
        }),
    );
    registry.register(
        "otlp_c.end_span",
        Arc::new(EndSpanFn {
            state: state.clone(),
        }),
    );
    registry.register(
        "otlp_c.set_attribute",
        Arc::new(SetAttributeFn {
            state: state.clone(),
        }),
    );
    registry.register(
        "otlp_c.add_event",
        Arc::new(AddEventFn {
            state: state.clone(),
        }),
    );
    registry.register(
        "otlp_c.record_log",
        Arc::new(RecordLogFn {
            state: state.clone(),
        }),
    );
    registry.register(
        "otlp_c.record_metric",
        Arc::new(RecordMetricFn {
            state: state.clone(),
        }),
    );
    registry.register(
        "otlp_c.flush",
        Arc::new(FlushFn {
            state: state.clone(),
        }),
    );
    registry.register("otlp_c.shutdown", Arc::new(ShutdownFn { state }));
}
