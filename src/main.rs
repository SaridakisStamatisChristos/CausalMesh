use causalmesh::{sync_pair, Replica};
use clap::{Parser, Subcommand};
use serde_json::json;
use std::error::Error;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "causalmesh", version, about = "Offline-first causal replicated state engine")]
struct Cli {
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Init { dir: PathBuf },
    Put { dir: PathBuf, key: String, value: String },
    Get { dir: PathBuf, key: String },
    Delete { dir: PathBuf, key: String },
    Status { dir: PathBuf },
    Verify { dir: PathBuf },
    ExportSummary { dir: PathBuf },
    Sync { dir_a: PathBuf, dir_b: PathBuf },
    DumpLog { dir: PathBuf },
    Snapshot { dir: PathBuf },
    Compact { dir: PathBuf },
    Doctor { dir: PathBuf },
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init { dir } => {
            let replica = Replica::open(&dir)?;
            print_value(cli.json, json!({"replica_id": replica.id(), "dir": dir}))?;
        }
        Command::Put { dir, key, value } => {
            let mut replica = Replica::open(dir)?;
            let dot = replica.put(key, value.into_bytes())?;
            print_value(cli.json, json!({"operation_id": dot}))?;
        }
        Command::Get { dir, key } => {
            let replica = Replica::open(dir)?;
            let values = replica.get(&key)?;
            if cli.json {
                print_value(true, json!({"key": key, "values": values}))?;
            } else if values.is_empty() {
                println!("<deleted-or-absent>");
            } else {
                for value in values {
                    println!("{}", String::from_utf8_lossy(&value));
                }
            }
        }
        Command::Delete { dir, key } => {
            let mut replica = Replica::open(dir)?;
            let dot = replica.delete(key)?;
            print_value(cli.json, json!({"operation_id": dot}))?;
        }
        Command::Status { dir } => {
            let replica = Replica::open(dir)?;
            print_serializable(&replica.status()?)?;
        }
        Command::Verify { dir } => {
            let replica = Replica::open(dir)?;
            replica.verify()?;
            print_value(cli.json, json!({"ok": true}))?;
        }
        Command::ExportSummary { dir } => {
            let replica = Replica::open(dir)?;
            print_serializable(&replica.summary()?)?;
        }
        Command::Sync { dir_a, dir_b } => {
            let mut a = Replica::open(dir_a)?;
            let mut b = Replica::open(dir_b)?;
            let report = sync_pair(&mut a, &mut b)?;
            print_serializable(&report)?;
        }
        Command::DumpLog { dir } => {
            let replica = Replica::open(dir)?;
            print_serializable(&replica.operations()?)?;
        }
        Command::Snapshot { dir } => {
            let replica = Replica::open(dir)?;
            print_serializable(&replica.snapshot()?)?;
        }
        Command::Compact { dir } => {
            let replica = Replica::open(dir)?;
            let snapshot = replica.compact_safely()?;
            print_value(
                cli.json,
                json!({
                    "snapshot_generation": snapshot.generation,
                    "destructive_log_gc": false,
                    "reason": "v0.1 retains history because safe stale-peer/tombstone GC is not proven"
                }),
            )?;
        }
        Command::Doctor { dir } => {
            let replica = Replica::open(dir)?;
            replica.verify()?;
            print_value(
                cli.json,
                json!({
                    "ok": true,
                    "replica_id": replica.id(),
                    "warning": "cloned directories cannot be identified until conflicting same-replica history is observed"
                }),
            )?;
        }
    }
    Ok(())
}

fn print_value(json_mode: bool, value: serde_json::Value) -> Result<(), serde_json::Error> {
    if json_mode {
        println!("{}", serde_json::to_string(&value)?);
    } else if let Some(text) = value.as_str() {
        println!("{text}");
    } else {
        println!("{}", serde_json::to_string_pretty(&value)?);
    }
    Ok(())
}

fn print_serializable<T: serde::Serialize>(value: &T) -> Result<(), serde_json::Error> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
