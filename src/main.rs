mod cli;
mod display;
mod error;
mod monitor;
mod ntapi;
mod privilege;
mod purge;
mod stats;
mod statswindow;
mod tray;

use std::process;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};

use cli::{Cli, Command, PurgeTarget};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Stats => {
            match stats::collect_stats() {
                Ok(s) => display::render(&s, cli.json),
                Err(e) => {
                    eprintln!("Error collecting stats: {e}");
                    process::exit(1);
                }
            }
        }
        Command::Purge { target } => {
            run_purge(target, cli.json);
        }
        Command::Monitor => {
            monitor::run();
        }
    }
}

fn run_purge(target: PurgeTarget, json: bool) {
    // Check admin
    if !privilege::is_elevated() {
        eprintln!("Error: Purge commands require Administrator privileges.");
        eprintln!("Please run this program as Administrator.");
        process::exit(1);
    }

    // Enable privileges
    if let Err(e) = privilege::elevate_for_purge() {
        eprintln!("Error enabling privileges: {e}");
        process::exit(1);
    }

    // Collect before stats
    let before = match stats::collect_stats() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error collecting stats: {e}");
            process::exit(1);
        }
    };

    if !json {
        println!("=== Before Purge ===\n");
        display::render(&before, false);
        println!();
    }

    // Run purge with spinner
    let description = match &target {
        PurgeTarget::WorkingSets => "Purging working sets...",
        PurgeTarget::Standby { low_only } => {
            if *low_only {
                "Purging low-priority standby list..."
            } else {
                "Purging standby list..."
            }
        }
        PurgeTarget::Modified => "Flushing modified page list...",
        PurgeTarget::All => "Purging all memory regions...",
    };

    let (tx, rx) = mpsc::channel::<Result<String, String>>();

    let target_clone = match &target {
        PurgeTarget::WorkingSets => PurgeTargetSimple::WorkingSets,
        PurgeTarget::Standby { low_only } => PurgeTargetSimple::Standby(*low_only),
        PurgeTarget::Modified => PurgeTargetSimple::Modified,
        PurgeTarget::All => PurgeTargetSimple::All,
    };

    thread::spawn(move || {
        let result = execute_purge(target_clone);
        let _ = tx.send(result);
    });

    // Spinner on main thread
    if !json {
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .tick_strings(&["\u{2807}", "\u{2819}", "\u{2839}", "\u{2838}", "\u{283c}", "\u{2834}", "\u{2826}", "\u{2827}", "\u{2807}", "\u{280f}"])
                .template("{spinner} {msg}")
                .expect("invalid spinner template"),
        );
        spinner.set_message(description.to_string());
        spinner.enable_steady_tick(Duration::from_millis(80));

        let result = loop {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(r) => break r,
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    break Err("Worker thread panicked".to_string());
                }
            }
        };

        spinner.finish_and_clear();

        match result {
            Ok(msg) => {
                println!("\u{2713} {msg}");
            }
            Err(msg) => {
                eprintln!("\u{2717} {msg}");
                process::exit(1);
            }
        }
    } else {
        // JSON mode: just wait, no spinner
        match rx.recv() {
            Ok(Ok(_)) => {}
            Ok(Err(msg)) => {
                eprintln!("{}", serde_json::json!({ "error": msg }));
                process::exit(1);
            }
            Err(_) => {
                eprintln!("{}", serde_json::json!({ "error": "Worker thread panicked" }));
                process::exit(1);
            }
        }
    }

    // Collect after stats
    let after = match stats::collect_stats() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error collecting stats: {e}");
            process::exit(1);
        }
    };

    if json {
        let output = serde_json::json!({
            "before": before,
            "after": after,
            "delta": {
                "available_physical_mb": after.available_physical_mb - before.available_physical_mb,
                "standby_mb": after.standby_mb - before.standby_mb,
                "modified_mb": after.modified_mb - before.modified_mb,
            }
        });
        println!("{}", serde_json::to_string_pretty(&output).unwrap());
    } else {
        println!("\n=== After Purge ===\n");
        display::render(&after, false);
        display::print_diff(&before, &after);
    }
}

/// Simple enum without clap derives for sending across threads.
enum PurgeTargetSimple {
    WorkingSets,
    Standby(bool),
    Modified,
    All,
}

fn execute_purge(target: PurgeTargetSimple) -> Result<String, String> {
    match target {
        PurgeTargetSimple::WorkingSets => {
            let result = purge::purge_working_sets().map_err(|e| e.to_string())?;
            Ok(format!(
                "Trimmed {} processes ({} skipped)",
                result.processes_trimmed, result.processes_skipped
            ))
        }
        PurgeTargetSimple::Standby(low_only) => {
            purge::purge_standby(low_only).map_err(|e| e.to_string())?;
            let label = if low_only {
                "low-priority standby list"
            } else {
                "standby list"
            };
            Ok(format!("Purged {label}"))
        }
        PurgeTargetSimple::Modified => {
            purge::purge_modified().map_err(|e| e.to_string())?;
            Ok("Flushed modified page list".to_string())
        }
        PurgeTargetSimple::All => {
            let ws = purge::purge_working_sets().map_err(|e| e.to_string())?;
            purge::purge_modified().map_err(|e| e.to_string())?;
            purge::purge_standby(false).map_err(|e| e.to_string())?;
            Ok(format!(
                "Purged all (trimmed {} processes, flushed modified, purged standby)",
                ws.processes_trimmed
            ))
        }
    }
}
