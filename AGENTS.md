# lib-otlp-c

`handl-plugin-lib-otlp-c` — native OTLP exporter backend for
`lib-otlp`: the Rust plugin crate holding the `otlp_c.*` op impls
(spans, logs, metrics, flush/shutdown), plus the shared
`handl-packages/otlp` .hnl surface.

## Layout

| path | role |
| --- | --- |
| `src/lib.rs` | crate root — exports `otlp_op_registry`, `OtlpError`, `OtlpResult` |
| `src/otlp_module.rs` | `OTLP_OPS` / `otlp_op_impls` / `register_otlp_ops` |
| `src/{error,id,state,stub}.rs` | error type, id gen, stateful conn, stub backend |
| `src/otlp_module/tests.rs` | module tests |
| `handl-packages/otlp/` | the shared `otlp.*` .hnl contract package |
| `atoms.yaml`, `Cargo.toml`, `xtask/` | atoms / manifest / `cargo xtask` gate |
| `pkg.hnl`, `scripts/hnl-gate.sh`, `workflows/gate.yaml` | manifest / hnl gate / bupha entry |

## Contract

- `lib-otlp` is the `.hnl` contract; this crate registers ops as
  `otlp_c.*` — the `.hnl` `otlp.*` layer calls
  `ffi.call("otlp_c.*", …)`. Ops are pure `handl_ops::Op`s behind
  `otlp_op_registry` — a kernel `OpRegistry` the host imports via
  `OpRegistry::import_kernel`.
- The former `OtlpModule` host wrapper lives in
  `handl-plugin-lib-stdlib`; `lib-otlp-std` delegates to this crate's
  shared `otlp_op_registry` builder.
- Deps resolve from git — a plain clone test-runs as-is.

## Verification

```bash
cargo xtask gate        # fmt --check → clippy -D warnings → test → doc
HNL=../host-cli/target/debug/hnl scripts/hnl-gate.sh
```

## License

MIT.

<!-- myco:agent-awareness -->

## Agent awareness

- Session start / any work discussion: run `myco status --prompt`
  before proposing next steps — know the board first.
- User describes new work → offer to capture it (`myco todo add`,
  `issue new`, `proposal new`). Don't wait to be asked.
- Picking up a task: `myco resume <ID>` is the briefing.
- Wrapping up: `myco handoff <ID>` anything resumable; keep statuses
  honest (claimed → in-progress → blocked/done).

<!-- /myco:agent-awareness -->
