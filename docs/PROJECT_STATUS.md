# Aevora Client — implementation status

This file distinguishes working repository components from roadmap items. A checked-in feature is not considered verified until its CI test or manual test has passed.

## In the repository

- Native Rust desktop application with its own winit window and wgpu rendering stack.
- Login, friends/presence, place discovery and a custom join pipeline inherited from the upstream client.
- Local saved-account persistence and an add-account flow. This is account storage/switching support, not bulk live-session orchestration.
- A separate Rust-hosted Luau command-line runtime for pure Luau modules.
- Isolated local Luau worker processes for parallel tests, with a per-VM heap limit and execution deadline.
- Windows helper scripts to build and start the desktop executable and run local Luau scripts.

## Important limitations

- The live replication parser is version-sensitive and incomplete. A successful join/transport connection does not guarantee a complete rendered world.
- The Luau runner is not connected to a Roblox DataModel and does not provide \`game\`, \`workspace\`, Roblox services, networking, or executor APIs.
- Local Luau workers are not Roblox accounts and do not join any live game.
- The desktop application does not yet contain an embedded code editor, script browser, script run/stop panel, or per-script output panes.
- There is no supported bulk login/start of parallel live Roblox sessions or headless public-game farming mode.
- The runner's Luau heap cap is not a whole-process OS memory limit or a hardened security boundary.

## Windows quick start

1. Install Rust using [rustup](https://rustup.rs).
2. Run \`BUILD_AEVORA.bat\` from the repository folder.
3. Run \`RUN_AEVORA.bat\` to start the independent desktop client.
4. To run pure Luau locally, use \`RUN_LUAU.bat --script scripts\\examples\\hello.luau\`.
5. To stress-test one script in parallel local workers, use \`RUN_LUAU.bat --script scripts\\examples\\worker-info.luau --instances 8\`.

For Roblox-specific APIs in an experience you own or are authorized to develop, use Roblox Studio's test environment.
