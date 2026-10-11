use std::path::PathBuf;

use causalmesh::{sync_bidirectional, Replica, Result};
use clap::{Parser, Subcommand};
use serde::Serialize;

#[derive(Parser)]
#[command(
    name = "causalmesh",
    version,
    about = "Deterministic offline-first replicated state engine"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Init {
        dir: PathBuf,
    },
    Put {
        dir: PathBuf,
        key: String,
        value: String,
    },
    Get {
        dir: PathBuf,
        key: String,
        #[arg(long)]
        json: bool,
    },
    Delete {
        dir: PathBuf,
        key: String,
    },
    Status {
        dir: PathBuf,
    },
    Verify {
        dir: PathBuf,
    },
    ExportSummary {
        dir: PathBuf,
    },
    Sync {
        dir_a: PathBuf,
        dir_b: PathBuf,
    },
    DumpLog {
        dir: PathBuf,
    },
    Snapshot {
        dir: PathBuf,
    },
}

#[derive(Serialize)]
struct GetOutput {
    key: String,
    values: Vec<String>,
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Init { dir } => {
            let replica = Replica::init(&dir)?;
            println!("{}", serde_json::to_string_pretty(&replica.status()?)?);
        }
        Command::Put { dir, key, value } => {
            let mut replica = Replica::open(dir)?;
            let id = replica.put(key, value.into_bytes())?;
            println!("{id}");
        }
        Command::Get { dir, key, json } => {
            let replica = Replica::open(dir)?;
            let values = replica
                .get(&key)
                .into_iter()
                .map(|v| String::from_utf8_lossy(&v).into_owned())
                .collect::<Vec<_>>();
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&GetOutput { key, values })?
                );
            } else {
                for value in values {
                    println!("{value}");
                }
            }
        }
        Command::Delete { dir, key } => {
            let mut replica = Replica::open(dir)?;
            println!("{}", replica.delete(key)?);
        }
        Command::Status { dir } | Command::Verify { dir } => {
            let replica = Replica::open(dir)?;
            println!("{}", serde_json::to_string_pretty(&replica.status()?)?);
        }
        Command::ExportSummary { dir } => {
            let replica = Replica::open(dir)?;
            println!("{}", serde_json::to_string_pretty(&replica.summary())?);
        }
        Command::Sync { dir_a, dir_b } => {
            let mut a = Replica::open(dir_a)?;
            let mut b = Replica::open(dir_b)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&sync_bidirectional(&mut a, &mut b)?)?
            );
        }
        Command::DumpLog { dir } => {
            let replica = Replica::open(dir)?;
            println!("{}", serde_json::to_string_pretty(&replica.operations())?);
        }
        Command::Snapshot { dir } => {
            let replica = Replica::open(dir)?;
            println!("{}", replica.snapshot()?.display());
        }
    }
    Ok(())
}
