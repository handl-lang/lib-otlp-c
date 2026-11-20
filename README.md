# lib-otlp-c

`handl-plugin-lib-otlp-c` — native OTLP exporter backend for lib-otlp.

Ships both the `handl-packages/otlp` .hnl package and the Rust plugin crate
under `src/`.

## Layout

```text
handl-packages/otlp/   the .hnl package
src/                   the Rust plugin crate
ci.sh                  bupha entry → scripts/hnl-gate.sh
```

## Gate

```bash
HNL=../host-cli/target/debug/hnl scripts/hnl-gate.sh
./ci.sh                # full gate (bupha entry)
```

## Development

```bash
cargo xtask gate        # fmt --check → clippy -D warnings → test → doc
```

`cargo xtask` wraps the repo's `xtask/` crate (`fmt`, `clippy`, `test`, `doc`,
`check`, `gate`). Deps resolve from git, so no sibling checkouts are needed;
a plain clone test-runs as-is.

## License

MIT.
