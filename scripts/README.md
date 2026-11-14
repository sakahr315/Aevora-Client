# Aevora custom scripts

Place your own `.lua` or `.luau` files in this directory. Keep reusable
scripts in subfolders and use descriptive filenames.

## Run a local script

From the repository root:

```powershell
cargo run --manifest-path tools/aevora-luau/Cargo.toml -- --script scripts/examples/hello.luau
```

The first build downloads and compiles the bundled Luau runtime. Later runs
reuse Cargo's build cache.

## Current runtime boundary

The runner is an **offline local-development runtime**, not a Roblox client
executor. It does not provide `game`, `workspace`, Roblox services, HTTP,
filesystem access, or exploit/executor APIs. It is useful for testing pure
Luau logic and small standalone modules.

The current runner removes common operating-system, filesystem, dynamic-loader,
and debug globals from the script environment. It is an early development
sandbox, not a hardened security boundary: do not run untrusted scripts in it,
and do not treat it as safe against scripts designed to exhaust CPU or memory.

## Planned integration

The next step is to expose a script workspace in the Aevora UI with file
browsing, editing, run/stop controls, output, and diagnostics. That UI is not
implemented by this initial runner.
