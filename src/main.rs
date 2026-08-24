use anyhow::{Context, Result, anyhow};
use clap::{Parser, ValueEnum};
use serde::Serialize;
use sqlglot_rust::{Dialect, parse};
use std::{fs, path::PathBuf};

#[derive(Clone, Copy, Debug, Serialize, PartialEq, ValueEnum)]
#[serde(rename_all = "kebab-case")]
enum VmgAction {
    Add,
    Check,
}

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    action: VmgAction,

    #[arg(help = "Migration directory")]
    directory: PathBuf,
}

fn validate_files(directory: &PathBuf) -> Result<()> {
    let files = fs::read_dir(directory)
        .with_context(|| format!("Failed to read directory '{:?}'", directory))?;

    for file in files {
        let entry = file?;

        let file_name = entry.file_name();
        let file_name = file_name
            .to_str()
            .ok_or(anyhow!("Failed to parse file name"))?;

        if !file_name.ends_with(".sql") {
            continue;
        }

        let query = fs::read_to_string(entry.path())?;

        if parse(&query, Dialect::Postgres).is_ok() {
            println!("{file_name} passed");
        } else {
            println!("{file_name} failed");
        };
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    if !args.directory.is_dir() {
        return Err(anyhow!("Migration directory is a file"));
    }

    match args.action {
        VmgAction::Add => {}
        VmgAction::Check => validate_files(&args.directory)?,
    };

    Ok(())
}
