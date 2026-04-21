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
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    spinner.set_message(format!("Downloading model {}…", opts.model));
    spinner.enable_steady_tick(std::time::Duration::from_millis(80));

    let model_file = resolve_model_path(opts.model, opts.model_path)
        .inspect_err(|_| spinner.finish_and_clear())?;

    spinner.set_message(format!("Loading model {}…", opts.model));

    let ctx = WhisperContext::new_with_params(&model_file, WhisperContextParameters::default())
        .map_err(|e| AppError::TranscriptionFailed(e.to_string()))?;

    spinner.set_message("Transcribing… 0% (0s)");

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

    let start = std::time::Instant::now();
    let progress_spinner = spinner.clone();
    params.set_progress_callback_safe(move |pct: i32| {
        let elapsed = start.elapsed().as_secs();
        progress_spinner.set_message(format!("Transcribing… {pct}% ({elapsed}s)"));
    });

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
        return if p.exists() {
            Ok(p.to_path_buf())
        } else {
            Err(AppError::ModelNotFound {
                path: p.to_path_buf(),
            }
            .into())
        };
    }

    // HF Hub cache — fetches from ggerganov/whisper.cpp if not already cached.
    // Downloads to ~/.cache/huggingface/hub (shared with other HF-aware tools).
    // Respects HF_HUB_CACHE and HF_HOME env vars.
    let filename = format!("ggml-{model}.bin");
    let path = hf_hub::api::sync::Api::new()
        .map_err(|e| AppError::ModelDownloadFailed(e.to_string()))?
        .model("ggerganov/whisper.cpp".to_string())
        .get(&filename)
        .map_err(|e| AppError::ModelDownloadFailed(e.to_string()))?;
    Ok(path)
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

    #[test]
    fn resolve_model_path_hf_cache_miss_errors() {
        let tmp = tempfile::tempdir().unwrap();
        // Point HF Hub at an empty temp dir and redirect the endpoint to an
        // unreachable local address so no real network I/O occurs.
        unsafe {
            std::env::set_var("HF_HUB_CACHE", tmp.path());
            std::env::set_var("HF_ENDPOINT", "http://127.0.0.1:0");
        }
        let result = resolve_model_path("large", None);
        unsafe {
            std::env::remove_var("HF_ENDPOINT");
            std::env::remove_var("HF_HUB_CACHE");
        }
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Failed to download model"),
            "unexpected error: {err}"
        );
    }
}
// grcov-excl-stop
