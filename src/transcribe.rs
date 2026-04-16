use std::path::{Path, PathBuf};

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct Segment {
    pub start_ms: i64,
    pub end_ms: i64,
    pub text: String,
}

pub struct TranscribeOptions<'a> {
    pub model: &'a str,
    pub model_path: Option<&'a Path>,
    pub language: Option<&'a str>,
    pub prompt: Option<&'a str>,
    pub word_timestamps: bool,
}

// grcov-excl-start: real whisper model loading requires integration tests or an injected context seam
pub fn transcribe(samples: &[f32], opts: TranscribeOptions<'_>) -> Result<Vec<Segment>> {
    let model_file = resolve_model_path(opts.model, opts.model_path)?;

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message(format!("Loading model {}…", opts.model));
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let ctx = WhisperContext::new_with_params(&model_file, WhisperContextParameters::default())
        .map_err(|e| AppError::TranscriptionFailed(e.to_string()))?;

    spinner.set_message("Transcribing…");

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

    if let Some(lang) = opts.language {
        params.set_language(Some(lang));
    }

    if let Some(prompt) = opts.prompt {
        params.set_initial_prompt(prompt);
    }

    params.set_token_timestamps(opts.word_timestamps);

    let mut state = ctx
        .create_state()
        .map_err(|e| AppError::TranscriptionFailed(e.to_string()))?;

    state
        .full(params, samples)
        .map_err(|e| AppError::TranscriptionFailed(e.to_string()))?;

    spinner.finish_and_clear();

    let n = state.full_n_segments();
    let mut segments = Vec::with_capacity(n as usize);

    for i in 0..n {
        if let Some(seg) = state.get_segment(i) {
            let text = seg
                .to_str_lossy()
                .map_err(|e| AppError::TranscriptionFailed(e.to_string()))?;
            // whisper timestamps are in centiseconds; convert to milliseconds
            let start_ms = seg.start_timestamp() * 10;
            let end_ms = seg.end_timestamp() * 10;

            segments.push(Segment {
                start_ms,
                end_ms,
                text: text.trim().to_string(),
            });
        }
    }

    Ok(segments)
}
// grcov-excl-stop

fn resolve_model_path(model: &str, override_path: Option<&Path>) -> Result<PathBuf> {
    if let Some(p) = override_path {
        if p.exists() {
            return Ok(p.to_path_buf());
        }
        return Err(AppError::ModelNotFound {
            path: p.to_path_buf(),
        }
        .into());
    }

    let default_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("voxscribe")
        .join("models");

    let path = default_dir.join(format!("ggml-{model}.bin"));

    if path.exists() {
        Ok(path)
    } else {
        Err(AppError::ModelNotFound { path }.into())
    }
}

// grcov-excl-start: exclude inline unit tests from production coverage
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_model_path_uses_override_when_exists() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let result = resolve_model_path("large", Some(tmp.path()));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), tmp.path());
    }

    #[test]
    fn resolve_model_path_errors_when_override_missing() {
        let path = PathBuf::from("/nonexistent/model.bin");
        let err = resolve_model_path("large", Some(&path)).unwrap_err();
        assert!(err.to_string().contains("Model file not found"));
    }
}
// grcov-excl-stop
