use anyhow::{Context, Result};
use clap::Parser;
use serde::Serialize;
use std::io::{self, Read};
use std::process;

use semlocal::cli::{Cli, Command};
use semlocal::search::{self, SearchResult};
use semlocal::store::Store;

#[cfg(feature = "embed")]
use semlocal::embed::Embedder;

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("error: {e:#}");
        process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Write { text } => cmd_write(&cli.src, &cli.collection, &resolve_text(text)?),
        Command::Search { text, top, json } => cmd_search(&cli.src, &cli.collection, &resolve_text(text)?, top, json),
        Command::Delete { id } => cmd_delete(&cli.src, &cli.collection, &id),
    }
}

fn resolve_text(arg: Option<String>) -> Result<String> {
    match arg {
        Some(ref s) if s != "-" => Ok(s.clone()),
        _ => {
            let mut buf = String::new();
            io::stdin()
                .read_to_string(&mut buf)
                .context("failed to read from stdin")?;
            let text = buf.trim().to_string();
            anyhow::ensure!(!text.is_empty(), "no text provided (pass an argument or pipe via stdin)");
            Ok(text)
        }
    }
}

fn cmd_write(src: &std::path::Path, collection: &str, text: &str) -> Result<()> {
    let mut embedder = Embedder::new()?;
    let embedding = embedder.embed(text)?;
    let store = Store::open(src, true)?;
    let id = uuid::Uuid::new_v4().to_string();
    store.insert(&id, collection, text, &embedding)?;
    println!("{id}");
    Ok(())
}

fn cmd_search(src: &std::path::Path, collection: &str, text: &str, top: usize, json: bool) -> Result<()> {
    let store = Store::open(src, false)?;
    let mut embedder = Embedder::new()?;
    let query_embedding = embedder.embed(text)?;
    let entries = store.all_entries(collection)?;
    let results = search::search(&query_embedding, &entries, top);

    if json {
        print_json(&results)?;
    } else {
        print_plain(&results);
    }

    Ok(())
}

fn cmd_delete(src: &std::path::Path, collection: &str, id: &str) -> Result<()> {
    let store = Store::open(src, false)?;
    let deleted = store.delete(id, collection)?;
    if !deleted {
        anyhow::bail!("entry not found: {id}");
    }
    Ok(())
}

#[derive(Serialize)]
struct JsonResult {
    id: String,
    score: f32,
    content: String,
}

fn print_json(results: &[SearchResult]) -> Result<()> {
    let out: Vec<JsonResult> = results
        .iter()
        .map(|r| JsonResult {
            id: r.id.clone(),
            score: r.score,
            content: r.content.clone(),
        })
        .collect();
    println!("{}", serde_json::to_string_pretty(&out)?);
    Ok(())
}

fn print_plain(results: &[SearchResult]) {
    for r in results {
        println!("[{:.2}] {} {}", r.score, r.id, r.content);
    }
}
