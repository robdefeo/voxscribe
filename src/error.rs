use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Input file not found: {0}")]
    InputNotFound(PathBuf),

    #[error("Unsupported format: {0}. Supported: mp3, wav, m4a, mp4, ogg, flac")]
    UnsupportedFormat(String),

    #[error("Failed to decode audio: {0}")]
    DecodeFailed(String),

    #[error("Failed to resample audio: {0}")]
    ResampleFailed(String),

    #[error(
        "Model file not found at {path}\n\
        Download models from: https://huggingface.co/ggerganov/whisper.cpp\n\
        Place the file at: {path}"
    )]
    ModelNotFound { path: PathBuf },

    #[error("Failed to download model: {0}\nCheck your network connection or use --model-path to specify a local file")]
    ModelDownloadFailed(String),

    #[error("Transcription error: {0}")]
    TranscriptionFailed(String),

    #[error("Invalid correction dictionary: {0}")]
    InvalidDict(String),

    #[error("Failed to write output: {0}")]
    OutputFailed(String),
}
