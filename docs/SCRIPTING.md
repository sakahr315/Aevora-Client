# Aevora scripting workspace

## Available now

- A standalone Rust-hosted Luau runner under `tools/aevora-luau`.
- Custom `.lua` and `.luau` files under `scripts/`.
- Console output through `print()` and `warn()`.
- An offline Luau sandbox with the global environment and builtin libraries made read-only.
- Per-worker Luau heap limits and execution deadlines.
- Parallel local test workers. Every worker is a separate operating-system process with an independent Luau state.
- A small `Aevora` metadata table available to test scripts:
  - `Aevora.instanceIndex`
  - `Aevora.instanceCount`
  - `Aevora.instanceName`

## Build and run

Requires a Rust toolchain.

List scripts recursively:

```powershell
cargo run --manifest-path tools/aevora-luau/Cargo.toml -- --list
```

Run one script:

```powershell
cargo run --manifest-path tools/aevora-luau/Cargo.toml -- --script scripts/examples/hello.luau
```

Run the same script in eight isolated local workers:

```powershell
cargo run --manifest-path tools/aevora-luau/Cargo.toml -- --script scripts/examples/worker-info.luau --instances 8
```

Tune each worker's limits:

```powershell
cargo run --manifest-path tools/aevora-luau/Cargo.toml -- --script scripts/examples/hello.luau --instances 4 --timeout-ms 3000 --memory-mb 96
```

The runner accepts 1-32 workers, a 100-60000 ms execution deadline, and an 8-1024 MB Luau heap limit per worker. The process manager starts all workers concurrently and returns a failure exit code if any worker fails.

## Important boundaries

These workers are **local Luau test processes**, not Roblox accounts or live game sessions. They do not log in to Roblox, join public servers, load a Roblox DataModel, or provide `game`, `workspace`, Roblox services, HTTP, filesystem access, or executor APIs. A Roblox script that depends on those APIs will not run in this standalone environment.

Use Roblox Studio's own test/play environment to exercise scripts against an experience you own or are authorized to develop. Aevora's local runner is for pure Luau modules, logic tests, and parallel script stress tests.

The sandbox and quotas reduce accidental runaway execution, but this is not a hardened operating-system security boundary. Do not run scripts from untrusted sources.
 
## Desktop UI

Open **Settings → Scripts** in Aevora to refresh the local file list, select a script, open it in the system text editor, run it, stop it, and inspect its output. The UI launches the standalone local runner and polls its process without blocking the render/event loop. The editor itself is external (for example, Notepad on Windows), not an embedded code editor.

## Still not implemented

- Embedded code editor with syntax highlighting and multi-file tabs.
- Roblox Studio integration or a full Roblox DataModel implementation.
- Resource accounting for the full OS process (the Luau heap limit applies to the VM).
