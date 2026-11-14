# Aevora custom scripts

Place your own \`.lua\` or \`.luau\` files in this directory. Keep reusable
scripts in subfolders and use descriptive filenames.

## Run a local script

From the repository root:

\`\`\`powershell
cargo run --manifest-path tools/aevora-luau/Cargo.toml -- --script scripts/examples/hello.luau
\`\`\`

Run the same pure-Luau test in parallel isolated processes:

\`\`\`powershell
cargo run --manifest-path tools/aevora-luau/Cargo.toml -- --script scripts/examples/worker-info.luau --instances 8
\`\`\`

The first build downloads and compiles the bundled Luau runtime. Later runs
reuse Cargo's build cache.

## Runtime boundary

This is an offline local-development runtime, not a Roblox client executor.
It does not provide \`game\`, \`workspace\`, Roblox services, HTTP, filesystem
access, or exploit/executor APIs. It is useful for pure Luau logic and small
standalone modules. Scripts that depend on Roblox APIs must be tested in
Roblox Studio against an experience you own or are authorized to develop.

Each local worker is a separate process with its own Luau state. The runner
applies a per-VM memory limit and execution deadline. These limits are not a
hardened operating-system sandbox; do not run untrusted scripts.
