use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Input file not found: {0}")]
    InputNotFound(PathBuf),

    #[error("Unsupported format: {0}. Supported: mp3, wav, m4a, mp4, ogg, flac")]
    UnsupportedFormat(String),

    #[error("ffmpeg not found — install ffmpeg and ensure it is in PATH")]
    FfmpegNotFound,

    #[error("ffmpeg conversion failed:\n{0}")]
    FfmpegFailed(String),

    #[error(
        "Model file not found at {path}\n\
        Download models from: https://huggingface.co/ggerganov/whisper.cpp\n\
        Place the file at: {path}"
    )]
    ModelNotFound { path: PathBuf },

    #[error("Transcription error: {0}")]
    TranscriptionFailed(String),

    #[error("Invalid correction dictionary: {0}")]
    InvalidDict(String),

    #[error("Failed to write output: {0}")]
    OutputFailed(String),
}
