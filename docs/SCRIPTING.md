# Aevora scripting roadmap

## Available now

- A standalone Luau runner under `tools/aevora-luau`.
- Custom `.lua` and `.luau` files under `scripts/`.
- Console output through `print()` and warnings through `warn()`.
- An offline runtime with common OS, filesystem, dynamic-loader, and debug
  globals removed.
- A checked-in example at `scripts/examples/hello.luau`.

## Build and run

Requires the Rust toolchain used by the main project.

```powershell
cargo run --manifest-path tools/aevora-luau/Cargo.toml -- --help
cargo run --manifest-path tools/aevora-luau/Cargo.toml -- --script scripts/examples/hello.luau
```

## Not implemented yet

- Embedded editor and script browser in the desktop UI.
- Script persistence/settings UI.
- Run/stop controls and per-script output panes.
- Instruction and memory quotas.
- Roblox DataModel bindings and Studio integration.
- Live-game script injection or executor compatibility.

The runtime is deliberately separate from the live client networking code.
Future integration should preserve that separation and only expose documented,
least-privilege APIs for local test worlds or experiences the developer owns.
