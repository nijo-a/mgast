use anyhow::{Context, Result, anyhow};
use clap::{Parser, ValueEnum};
use colored::Colorize;
use inquire::{Editor, Text};
use serde::Serialize;
use sqlglot_rust::{Dialect, parse};
use std::{fs, path::PathBuf, println};

#[derive(Clone, Copy, Debug, Serialize, PartialEq, ValueEnum)]
#[serde(rename_all = "kebab-case")]
enum Action {
    Add,
    Check,
}

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    action: Action,

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
            let output = format!("{} passed", file_name);
            println!("{}", output.green());
        } else {
            let output = format!("{} failed", file_name);
            println!("{}", output.red());
        };
    }

    Ok(())
}

fn add_migration(directory: &PathBuf) -> Result<()> {
    let files = fs::read_dir(directory)
        .with_context(|| format!("Failed to read directory '{:?}'", directory))?;

    let mut ver = 1;

    let mut digits = 1;

    for file in files {
        let entry = file?;

        let file_name = entry.file_name();
        let file_name = file_name
            .to_str()
            .ok_or(anyhow!("Failed to parse file name"))?;

        let parts = file_name.split("_").collect::<Vec<_>>();

        if parts.len() <= 1 {
            continue;
        }

        if let Some(first) = parts.get(0) {
            digits = digits.max(first.len());

            if let Ok(file_ver) = u16::from_str_radix(*first, 10) {
                ver = ver.max(file_ver);
            }
        }
    }

    let next_ver = (ver + 1).to_string();
    let next_ver = "0".repeat(digits - next_ver.len()) + &next_ver;

    let query = Editor::new("Input query")
        .with_file_extension(".sql")
        .prompt()?;

    if query.is_empty() {
        return Err(anyhow!("No input given"));
    }

    if parse(&query, Dialect::Postgres).is_err() {
        return Err(anyhow!("Invalid query"));
    }

    let name = Text::new("Migration name")
        .with_help_message(&format!("Ver {}", next_ver))
        .prompt()?;

    let file_name = format!("{}_{}.sql", next_ver, name.replace(" ", "_").to_lowercase());

    let mut full_path = directory.clone();
    full_path.push(&file_name);

    let query_preview = &query[..(query.len() - 1).min(50)];

    println!("Writing query to {full_path:?}...");
    println!("{query_preview}");

    fs::write(full_path, query)?;

    println!("Success!");

    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    if !args.directory.is_dir() {
        return Err(anyhow!("Migration directory is a file"));
    }

    match args.action {
        Action::Add => add_migration(&args.directory)?,
        Action::Check => validate_files(&args.directory)?,
    };

    Ok(())
}
