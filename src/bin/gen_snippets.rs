use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let check_only = args.iter().any(|a| a == "--check");

    // Ensure the main binary is built
    let mut build = Command::new("cargo");
    build.arg("build");
    run(&mut build);

    let bin = PathBuf::from("target")
        .join("debug")
        .join(if cfg!(windows) { "mica.exe" } else { "mica" });

    let pretty_adt =
        run_capture(Command::new(&bin).args(["--ast", "--pretty", "examples/adt.mica"]));
    let check_nx =
        run_capture(Command::new(&bin).args(["--check", "examples/adt_match_nonexhaustive.mica"]));
    let lower_methods = run_capture(Command::new(&bin).args(["--lower", "examples/methods.mica"]));
    let typed_ir = run_capture(Command::new(&bin).args(["--ir", "examples/methods.mica"]));
    let llvm_methods = run_capture(Command::new(&bin).args(["--llvm", "examples/methods.mica"]));
    let lower_spawn =
        run_capture(Command::new(&bin).args(["--lower", "examples/spawn_await.mica"]));
    let trace_native_entry = run_capture(Command::new(&bin).args([
        "--run",
        "--trace-json",
        "-",
        "examples/native_entry.mica",
    ]));

    let content = format!(
        "# CLI Snippets\n\nThis page shows short outputs from the CLI for selected examples.\n\n## Pretty AST (`--ast --pretty`)\n\nCommand: `cargo run --bin mica -- --ast --pretty examples/adt.mica`\n\n```\n{}\n```\n\n## Exhaustiveness Check (`--check`)\n\nCommand: `cargo run --bin mica -- --check examples/adt_match_nonexhaustive.mica`\n\n```\n{}\n```\n\n## Lowered HIR (`--lower`)\n\nCommand: `cargo run --bin mica -- --lower examples/methods.mica`\n\n```\n{}\n```\n\n## Typed IR (`--ir`)\n\nCommand: `cargo run --bin mica -- --ir examples/methods.mica`\n\n```\n{}\n```\n\n## LLVM Scaffold (`--llvm`)\n\nCommand: `cargo run --bin mica -- --llvm examples/methods.mica`\n\n```\n{}\n```\n\nCommand: `cargo run --bin mica -- --lower examples/spawn_await.mica`\n\n```\n{}\n```\n\n## Runtime Trace (`--run --trace-json`)\n\nCommand: `cargo run --bin mica -- --run --trace-json - examples/native_entry.mica`\n\n```\n{}\n```\n\n",
        pretty_adt.trim_end(),
        check_nx.trim_end(),
        lower_methods.trim_end(),
        typed_ir.trim_end(),
        llvm_methods.trim_end(),
        lower_spawn.trim_end(),
        trace_native_entry.trim_end()
    );

    let path = PathBuf::from("docs/snippets.md");
    if check_only {
        let existing = fs::read_to_string(&path).expect("read snippets.md");
        if normalize(&existing) != normalize(&content) {
            eprintln!("snippets out of date. Run: cargo run --bin gen_snippets");
            std::process::exit(1);
        }
    } else {
        fs::create_dir_all(path.parent().unwrap()).ok();
        fs::write(&path, content).expect("write snippets.md");
        println!("updated {}", path.display());
    }
}

fn run(cmd: &mut Command) {
    let status = cmd.status().expect("spawn");
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
}

fn run_capture(cmd: &mut Command) -> String {
    let out = cmd.output().expect("spawn");
    if !out.status.success() {
        eprintln!("command failed: {:?}", cmd);
        std::process::exit(out.status.code().unwrap_or(1));
    }
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn normalize(s: &str) -> String {
    s.replace("\r\n", "\n")
}
