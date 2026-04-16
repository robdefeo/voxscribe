mod audio;
mod cli;
mod error;
mod output;
mod transcribe;

use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};

use anyhow::{Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use tracing::info;

use crate::cli::Args;
use crate::error::AppError;
use crate::transcribe::TranscribeOptions;

// grcov-excl-start: process entrypoint wiring and terminal dispatch are covered indirectly
fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("warn")),
        )
        .init();

    let args = Args::parse();

    // Audio loading
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message("Loading and converting audio…");
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    info!("Loading audio from {:?}", args.input);
    let samples = audio::load(&args.input)?;
    spinner.finish_and_clear();

    info!("Loaded {} samples", samples.len());

    // Transcription
    let opts = TranscribeOptions {
        model: &args.model,
        model_path: args.model_path.as_deref(),
        language: args.language.as_deref(),
        prompt: args.prompt.as_deref(),
        word_timestamps: args.word_timestamps,
    };

    let mut segments = transcribe::transcribe(&samples, opts)?;

    // Apply correction dictionary if provided
    if let Some(dict_path) = &args.dict {
        let corrections = load_dict(dict_path)?;
        for seg in &mut segments {
            for (wrong, right) in &corrections {
                seg.text = seg.text.replace(wrong.as_str(), right.as_str());
            }
        }
    }

    // Format output
    let show_timestamps = args.timestamps || args.word_timestamps;
    let formatted = output::format(&segments, &args.format, show_timestamps);

    // Write output
    match &args.output {
        Some(path) => {
            fs::write(path, &formatted).map_err(|e| AppError::OutputFailed(e.to_string()))?;
            eprintln!("Transcript written to {}", path.display());
        }
        None => {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            writeln!(handle, "{formatted}").context("failed to write to stdout")?;
        }
    }

    Ok(())
}
// grcov-excl-stop

fn load_dict(path: &std::path::Path) -> Result<HashMap<String, String>> {
    let content = fs::read_to_string(path).map_err(|e| AppError::InvalidDict(e.to_string()))?;
    let map: HashMap<String, String> = serde_json::from_str(&content)
        .map_err(|e| AppError::InvalidDict(format!("invalid JSON: {e}")))?;
    Ok(map)
}
