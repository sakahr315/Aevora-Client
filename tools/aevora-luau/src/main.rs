use mlua::{Lua, Value, Variadic};
use std::{env, fs, path::PathBuf, process::ExitCode};

fn usage() {
    eprintln!("Aevora Luau Runner");
    eprintln!("Usage: aevora-luau --script <file.luau>");
    eprintln!("Runs a Luau file in an offline local-development environment.");
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

fn run(path: PathBuf) -> Result<(), String> {
    if path.extension().and_then(|s| s.to_str()) != Some("luau")
        && path.extension().and_then(|s| s.to_str()) != Some("lua")
    {
        return Err("script must use a .lua or .luau extension".to_owned());
    }

    let source = fs::read_to_string(&path)
        .map_err(|e| format!("could not read {}: {e}", path.display()))?;

    // This runner is intentionally offline. It does not create Roblox globals,
    // network APIs, filesystem APIs, or executor compatibility functions.
    let lua = Lua::new();
    let globals = lua.globals();

    for name in [
        "io", "os", "package", "debug", "require", "dofile", "loadfile",
        "loadstring", "game", "workspace", "HttpGet", "request", "getgenv",
        "getrawmetatable", "hookfunction", "firetouchinterest",
    ] {
        globals.set(name, Value::Nil).map_err(|e| e.to_string())?;
    }

    let print = lua
        .create_function(|_, values: Variadic<Value>| {
            println!("{}", values.iter().map(display_value).collect::<Vec<_>>().join("\t"));
            Ok(())
        })
        .map_err(|e| e.to_string())?;
    globals.set("print", print).map_err(|e| e.to_string())?;

    let warn = lua
        .create_function(|_, values: Variadic<Value>| {
            eprintln!("[WARN] {}", values.iter().map(display_value).collect::<Vec<_>>().join("\t"));
            Ok(())
        })
        .map_err(|e| e.to_string())?;
    globals.set("warn", warn).map_err(|e| e.to_string())?;

    println!("[Aevora] Running {} (offline Luau environment)", path.display());
    lua.load(&source)
        .set_name(path.to_string_lossy())
        .exec()
        .map_err(|e| format!("Luau execution failed: {e}"))?;
    println!("[Aevora] Script completed.");
    Ok(())
}

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let mut script = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--script" | "-s" => script = args.next().map(PathBuf::from),
            "--help" | "-h" => {
                usage();
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("Unknown argument: {other}");
                usage();
                return ExitCode::from(2);
            }
        }
    }

    let Some(path) = script else {
        usage();
        return ExitCode::from(2);
    };

    match run(path) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("[Aevora] {error}");
            ExitCode::FAILURE
        }
    }
}
