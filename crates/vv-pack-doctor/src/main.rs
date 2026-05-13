//! Pack Doctor CLI.
//!
//! Usage:
//!   vv-pack-doctor <pack_root> [--json <path>] [--html <path>]

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use vv_pack_doctor::{output, run};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    let parsed = match parse_args(&args) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e);
            eprintln!("Usage: vv-pack-doctor <pack_root> [--json <path>] [--html <path>]");
            return ExitCode::from(2);
        }
    };

    let report = match run(&parsed.pack_root) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Pack Doctor failed: {}", e);
            return ExitCode::from(2);
        }
    };

    if let Some(json_path) = &parsed.json {
        if let Err(e) = write_file(json_path, &output::json::render(&report)) {
            eprintln!("Cannot write JSON report: {}", e);
            return ExitCode::from(2);
        }
        println!("Wrote JSON report: {}", json_path.display());
    }

    if let Some(html_path) = &parsed.html {
        if let Err(e) = write_file(html_path, &output::html::render(&report)) {
            eprintln!("Cannot write HTML report: {}", e);
            return ExitCode::from(2);
        }
        println!("Wrote HTML report: {}", html_path.display());
    }

    // Always print a short text summary so the script wrapper is useful too.
    println!(
        "Pack Doctor: {} errors, {} warnings, score {}/100",
        report.summary.errors, report.summary.warnings, report.health_score
    );
    for err in &report.errors {
        print_diagnostic("ERROR", err);
    }
    for warn in &report.warnings {
        print_diagnostic("WARN ", warn);
    }

    if report.ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

struct Args {
    pack_root: PathBuf,
    json: Option<PathBuf>,
    html: Option<PathBuf>,
}

fn parse_args(args: &[String]) -> Result<Args, String> {
    let mut iter = args.iter().skip(1).peekable();

    // First positional (if present and not a flag) is the pack root.
    let pack_root = match iter.peek() {
        Some(s) if !s.starts_with("--") => PathBuf::from(iter.next().unwrap()),
        _ => PathBuf::from("assets/packs/core"),
    };

    let mut json = None;
    let mut html = None;
    while let Some(flag) = iter.next() {
        match flag.as_str() {
            "--json" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--json requires a path".to_string())?;
                json = Some(PathBuf::from(value));
            }
            "--html" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "--html requires a path".to_string())?;
                html = Some(PathBuf::from(value));
            }
            other => return Err(format!("unknown argument: {}", other)),
        }
    }
    Ok(Args {
        pack_root,
        json,
        html,
    })
}

fn write_file(path: &Path, content: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)
}

fn print_diagnostic(severity: &str, d: &vv_pack_doctor::report::Diagnostic) {
    let location = match (&d.path, &d.id) {
        (Some(p), Some(id)) if p != id => format!("{} ({})", p, id),
        (Some(p), _) => p.clone(),
        (None, Some(id)) => id.clone(),
        _ => "-".to_string(),
    };
    let field = d
        .field
        .as_deref()
        .map(|f| format!(" .{}", f))
        .unwrap_or_default();
    println!(
        "{} [{}] {} :: {}{}",
        severity, d.check, location, d.message, field
    );
    if let Some(s) = &d.suggestion {
        println!("       fix: {}", s);
    }
}
