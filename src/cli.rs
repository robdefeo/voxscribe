use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(
    name = "voxscribe",
    about = "Audio transcription using Whisper models. First run downloads the selected model; subsequent runs work fully offline.",
    max_term_width = 100
)]
pub struct Args {
    /// Audio or video file to transcribe (mp3, wav, m4a, mp4, ogg, flac)
    pub input: PathBuf,

    #[arg(
        long,
        default_value = "large-v3-turbo",
        help = "Whisper model to use. Downloaded from HuggingFace on first use; cached locally thereafter. \
                Recommended: large-v3-turbo for the best speed/accuracy balance for most audio. \
                Use large-v3 for maximum accuracy at about 2x slower speed. \
                Use medium or small for faster transcription with lower accuracy."
    )]
    pub model: String,

    /// Output file path (defaults to stdout)
    #[arg(long, short)]
    pub output: Option<PathBuf>,

    /// Output format
    #[arg(long, short, default_value = "txt", value_enum)]
    pub format: OutputFormat,

    /// Include segment-level timestamps in output
    #[arg(long)]
    pub timestamps: bool,

    /// Include word-level timestamps (implies --timestamps)
    #[arg(long)]
    pub word_timestamps: bool,

    /// Context prompt to guide transcription (e.g. domain-specific terms)
    #[arg(long)]
    pub prompt: Option<String>,

    /// Path to JSON correction dictionary {"wrong": "right"}
    #[arg(long)]
    pub dict: Option<PathBuf>,

    /// Force a specific language (e.g. "en", "de", "fr")
    #[arg(long)]
    pub language: Option<String>,

    /// Override path to model file instead of using the HuggingFace cache
    #[arg(long)]
    pub model_path: Option<PathBuf>,
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum OutputFormat {
    Txt,
    Json,
    Srt,
}
