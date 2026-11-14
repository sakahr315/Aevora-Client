use mlua::{Lua, Value, Variadic, VmState};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitCode};
use std::time::{Duration, Instant};

const DEFAULT_TIMEOUT_MS: u64 = 5_000;
const DEFAULT_MEMORY_MB: usize = 64;
const MAX_INSTANCES: usize = 32;
const MAX_TIMEOUT_MS: u64 = 60_000;
const MAX_MEMORY_MB: usize = 1_024;

#[derive(Debug)]
struct Args {
    script: PathBuf,
    instances: usize,
    timeout_ms: u64,
    memory_mb: usize,
    worker_index: Option<usize>,
    worker_count: Option<usize>,
}

fn usage() {
    eprintln!("Aevora Luau Runner");
    eprintln!();
    eprintln!("Usage:");
    eprintln!("  aevora-luau --script <file.luau> [options]");
    eprintln!();
    eprintln!("Options:");
    eprintln!("  -s, --script <path>       Luau script to run");
    eprintln!("  -n, --instances <count>   Run isolated local workers in parallel (1-32)");
    eprintln!("      --timeout-ms <ms>     Per-worker execution deadline (100-60000)");
    eprintln!("      --memory-mb <mb>      Luau heap limit per worker (8-1024)");
    eprintln!("  -h, --help                Show this help");
    eprintln!();
    eprintln!("This is a local Luau test runtime. It does not connect to Roblox servers.");
}

fn parse_value<T: std::str::FromStr>(name: &str, value: Option<String>) -> Result<T, String> {
    value
        .ok_or_else(|| format!("missing value for {name}"))?
        .parse::<T>()
        .map_err(|_| format!("invalid value for {name}"))
}

fn parse_args() -> Result<Option<Args>, String> {
    let mut args = env::args().skip(1);
    let mut script = None;
    let mut instances = 1usize;
    let mut timeout_ms = DEFAULT_TIMEOUT_MS;
    let mut memory_mb = DEFAULT_MEMORY_MB;
    let mut worker_index = None;
    let mut worker_count = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--script" | "-s" => script = args.next().map(PathBuf::from),
            "--instances" | "-n" => {
                instances = parse_value("--instances", args.next())?;
                if !(1..=MAX_INSTANCES).contains(&instances) {
                    return Err(format!("--instances must be between 1 and {MAX_INSTANCES}"));
                }
            }
            "--timeout-ms" => {
                timeout_ms = parse_value("--timeout-ms", args.next())?;
                if !(100..=MAX_TIMEOUT_MS).contains(&timeout_ms) {
                    return Err(format!(
                        "--timeout-ms must be between 100 and {MAX_TIMEOUT_MS}"
                    ));
                }
            }
            "--memory-mb" => {
                memory_mb = parse_value("--memory-mb", args.next())?;
                if !(8..=MAX_MEMORY_MB).contains(&memory_mb) {
                    return Err(format!("--memory-mb must be between 8 and {MAX_MEMORY_MB}"));
                }
            }
            // Internal child-process arguments used by --instances.
            "--worker-index" => worker_index = Some(parse_value("--worker-index", args.next())?),
            "--worker-count" => worker_count = Some(parse_value("--worker-count", args.next())?),
            "--help" | "-h" => {
                usage();
                return Ok(None);
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }

    let script = script.ok_or_else(|| "missing --script <path>".to_owned())?;
    if worker_index.is_some() != worker_count.is_some() {
        return Err("invalid internal worker arguments".to_owned());
    }
    if let (Some(index), Some(count)) = (worker_index, worker_count) {
        if count == 0 || count > MAX_INSTANCES || index == 0 || index > count {
            return Err("invalid internal worker index/count".to_owned());
        }
    }

    Ok(Some(Args {
        script,
        instances,
        timeout_ms,
        memory_mb,
        worker_index,
        worker_count,
    }))
}

fn display_value(value: &Value) -> String {
    match value {
        Value::Nil => "nil".to_owned(),
        Value::Boolean(v) => v.to_string(),
        Value::Integer(v) => v.to_string(),
        Value::Number(v) => v.to_string(),
        Value::String(v) => v.to_string_lossy(),
        other => format!("<{}>", other.type_name()),
    }
}

fn run_local(
    path: &Path,
    worker_index: usize,
    worker_count: usize,
    timeout_ms: u64,
    memory_mb: usize,
) -> Result<(), String> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if extension != "luau" && extension != "lua" {
        return Err("script must use a .lua or .luau extension".to_owned());
    }

    let source = fs::read_to_string(path)
        .map_err(|error| format!("could not read {}: {error}", path.display()))?;
    let lua = Lua::new();
    lua.set_memory_limit(memory_mb * 1024 * 1024)
        .map_err(|error| format!("could not set Luau memory limit: {error}"))?;

    // This runtime is intentionally offline. Do not expose Roblox globals,
    // filesystem/network APIs, dynamic loaders, or executor compatibility APIs.
    let globals = lua.globals();
    for name in [
        "io",
        "os",
        "package",
        "debug",
        "require",
        "dofile",
        "loadfile",
        "loadstring",
        "game",
        "workspace",
        "HttpGet",
        "request",
        "getgenv",
        "getrawmetatable",
        "hookfunction",
        "firetouchinterest",
    ] {
        globals
            .set(name, Value::Nil)
            .map_err(|error| error.to_string())?;
    }

    let prefix = format!("[Aevora][local {worker_index}/{worker_count}]");
    let print_prefix = prefix.clone();
    let print = lua
        .create_function(move |_, values: Variadic<Value>| {
            let message = values
                .iter()
                .map(display_value)
                .collect::<Vec<_>>()
                .join("\t");
            println!("{print_prefix} {message}");
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    globals
        .set("print", print)
        .map_err(|error| error.to_string())?;

    let warn_prefix = prefix.clone();
    let warn = lua
        .create_function(move |_, values: Variadic<Value>| {
            let message = values
                .iter()
                .map(display_value)
                .collect::<Vec<_>>()
                .join("\t");
            eprintln!("{warn_prefix} [WARN] {message}");
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    globals
        .set("warn", warn)
        .map_err(|error| error.to_string())?;

    let aevora = lua.create_table().map_err(|error| error.to_string())?;
    aevora
        .set("instanceIndex", worker_index)
        .map_err(|error| error.to_string())?;
    aevora
        .set("instanceCount", worker_count)
        .map_err(|error| error.to_string())?;
    aevora
        .set("instanceName", format!("local-{worker_index:02}"))
        .map_err(|error| error.to_string())?;
    globals
        .set("Aevora", aevora)
        .map_err(|error| error.to_string())?;

    // Luau sandbox mode makes builtin libraries and the global environment
    // read-only while allowing script-local globals in a separate environment.
    lua.sandbox(true)
        .map_err(|error| format!("could not enable Luau sandbox: {error}"))?;

    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    lua.set_interrupt(move |_| {
        if Instant::now() >= deadline {
            Err(mlua::Error::RuntimeError(format!(
                "execution timed out after {timeout_ms} ms"
            )))
        } else {
            Ok(VmState::Continue)
        }
    });

    println!(
        "{prefix} running {} (offline Luau, {memory_mb} MB heap, {timeout_ms} ms timeout)",
        path.display()
    );
    lua.load(&source)
        .set_name(path.display().to_string())
        .exec()
        .map_err(|error| format!("{prefix} script failed: {error}"))?;
    println!("{prefix} completed");
    Ok(())
}

fn spawn_workers(args: &Args) -> Result<bool, String> {
    let executable =
        env::current_exe().map_err(|error| format!("could not locate Aevora runner: {error}"))?;
    let mut children: Vec<Child> = Vec::with_capacity(args.instances);

    println!(
        "[Aevora] Starting {} isolated local Luau workers for {}",
        args.instances,
        args.script.display()
    );

    for index in 1..=args.instances {
        let mut command = Command::new(&executable);
        command
            .arg("--script")
            .arg(&args.script)
            .arg("--timeout-ms")
            .arg(args.timeout_ms.to_string())
            .arg("--memory-mb")
            .arg(args.memory_mb.to_string())
            .arg("--worker-index")
            .arg(index.to_string())
            .arg("--worker-count")
            .arg(args.instances.to_string());

        match command.spawn() {
            Ok(child) => children.push(child),
            Err(error) => {
                for child in &mut children {
                    let _ = child.kill();
                    let _ = child.wait();
                }
                return Err(format!("could not start local worker {index}: {error}"));
            }
        }
    }

    let mut all_succeeded = true;
    for (offset, child) in children.iter_mut().enumerate() {
        let status = child
            .wait()
            .map_err(|error| format!("could not wait for local worker {}: {error}", offset + 1))?;
        all_succeeded &= status.success();
    }
    Ok(all_succeeded)
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(Some(args)) => args,
        Ok(None) => return ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("[Aevora] {error}");
            usage();
            return ExitCode::from(2);
        }
    };

    if args.worker_index.is_none() && args.instances > 1 {
        return match spawn_workers(&args) {
            Ok(true) => ExitCode::SUCCESS,
            Ok(false) => {
                eprintln!("[Aevora] One or more local workers failed.");
                ExitCode::FAILURE
            }
            Err(error) => {
                eprintln!("[Aevora] {error}");
                ExitCode::FAILURE
            }
        };
    }

    let index = args.worker_index.unwrap_or(1);
    let count = args.worker_count.unwrap_or(1);
    match run_local(&args.script, index, count, args.timeout_ms, args.memory_mb) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temporary_script(contents: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after the Unix epoch")
            .as_nanos();
        let path = env::temp_dir().join(format!(
            "aevora-luau-test-{}-{nonce}.luau",
            std::process::id()
        ));
        fs::write(&path, contents).expect("test script should be written");
        path
    }

    #[test]
    fn exposes_local_worker_metadata() {
        let path = temporary_script(
            r#"
                assert(Aevora.instanceIndex == 2)
                assert(Aevora.instanceCount == 4)
                assert(Aevora.instanceName == "local-02")
            "#,
        );
        let result = run_local(&path, 2, 4, 5_000, 64);
        let _ = fs::remove_file(path);
        assert!(result.is_ok(), "unexpected runner error: {result:?}");
    }

    #[test]
    fn interrupts_an_endless_script_at_the_deadline() {
        let path = temporary_script("while true do end");
        let result = run_local(&path, 1, 1, 100, 64);
        let _ = fs::remove_file(path);
        let error = result.expect_err("an endless loop must time out");
        assert!(
            error.contains("timed out"),
            "expected timeout error, got: {error}"
        );
    }
}
