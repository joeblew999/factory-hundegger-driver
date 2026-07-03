//! `btlx` — a small command-line tool for the factory floor.
//!
//! Built so a shop can validate our BTLx handling against *their* real files
//! without a Rust toolchain: download the binary for your OS, run it on a `.btlx`
//! your CAD or machine produced.
//!
//!   btlx inspect path/to/your-file.btlx
//!   btlx demo > sample.btlx
//!   btlx sim --dispatch ./in --status ./status   # stand-in machine for the driver loop

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use factory_btlx::btlx::{model::*, to_xml};
use factory_btlx::inspect::{SERIALISABLE, inspect_str};

#[derive(Parser)]
#[command(
    name = "btlx",
    version,
    about = "Read and write Hundegger BTLx timber-CNC files"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Read a real .btlx file and report what's in it (version, parts, processings).
    Inspect {
        /// Path to a .btlx file from your CAD or machine.
        file: PathBuf,
    },
    /// Print a small sample .btlx to standard output.
    Demo,
    /// Act as a stand-in Hundegger controller: process dispatched .btlx files and
    /// write status lines — so the driver's dispatch→telemetry loop runs with no
    /// real machine.
    Sim {
        /// Directory the driver dispatches .btlx files into.
        #[arg(long)]
        dispatch: PathBuf,
        /// Directory to write the status log into (status.jsonl).
        #[arg(long)]
        status: PathBuf,
        /// Keep running, polling the dispatch dir (Ctrl-C to stop).
        #[arg(long)]
        watch: bool,
    },
}

fn main() -> ExitCode {
    match Cli::parse().command {
        Command::Inspect { file } => inspect_cmd(&file),
        Command::Demo => demo_cmd(),
        Command::Sim {
            dispatch,
            status,
            watch,
        } => sim_cmd(&dispatch, &status, watch),
    }
}

fn sim_cmd(dispatch: &Path, status: &Path, watch: bool) -> ExitCode {
    let once = || match factory_btlx::sim::process_pending(dispatch, status) {
        Ok(run) => {
            for j in &run.completed {
                println!("✓ completed {j}");
            }
            for j in &run.failed {
                println!("✗ failed    {j}");
            }
            true
        }
        Err(e) => {
            eprintln!("error: {e}");
            false
        }
    };
    if !watch {
        return if once() {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        };
    }
    println!(
        "simulating a Hundegger controller — watching {}",
        dispatch.display()
    );
    loop {
        once();
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

fn inspect_cmd(file: &Path) -> ExitCode {
    let xml = match std::fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read {}: {e}", file.display());
            return ExitCode::FAILURE;
        }
    };
    let report = match inspect_str(&xml) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: {} is not valid XML: {e}", file.display());
            return ExitCode::FAILURE;
        }
    };

    println!("File:    {}", file.display());
    println!("Version: {}", report.version.as_deref().unwrap_or("(none)"));
    println!("Parts:   {}", report.parts);
    println!("Processings ({} total):", report.total_processings());
    if report.processings.is_empty() {
        println!("  (none)");
    }
    for (name, count) in &report.processings {
        let mark = if SERIALISABLE.contains(&name.as_str()) {
            "ok"
        } else {
            "read-only"
        };
        println!("  {count:>5}  {name:<22} [{mark}]");
    }

    let unsupported = report.unsupported();
    println!();
    if unsupported.is_empty() {
        println!("We can read and write every processing in this file.");
    } else {
        println!(
            "We can READ this file. We cannot yet WRITE these processing types: {}.",
            unsupported.join(", ")
        );
        println!("Send this file to the dev team — it tells us exactly what to add next.");
    }
    ExitCode::SUCCESS
}

fn demo_cmd() -> ExitCode {
    let part = Part::new(3000.0, 160.0, 80.0)
        .designation("beam-1")
        .with_processings(vec![Processing::Drilling(Drilling::new(
            "bolt-hole",
            1,
            RefPlane::Global(3),
            500.0,
            80.0,
            80.0,
            12.0,
        ))]);
    match to_xml(&Btlx::new(Project::new("demo-project", vec![part]))) {
        Ok(xml) => {
            print!("{xml}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}
