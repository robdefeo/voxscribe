use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use tempfile::NamedTempFile;

use crate::error::AppError;

const SUPPORTED_EXTENSIONS: &[&str] = &["mp3", "wav", "m4a", "mp4", "ogg", "flac"];

/// Validate the input file and convert it to 16kHz mono f32 PCM samples.
pub fn load(input: &Path) -> Result<Vec<f32>> {
    validate(input)?;
    let pcm = convert_and_read(input)?;
    Ok(pcm)
}

fn validate(input: &Path) -> Result<()> {
    if !input.exists() {
        return Err(AppError::InputNotFound(input.to_path_buf()).into());
    }

    let ext = input
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if !SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
        return Err(AppError::UnsupportedFormat(ext).into());
    }

    Ok(())
}

fn convert_and_read(input: &Path) -> Result<Vec<f32>> {
    // Write converted audio to a named temp file that lives long enough to read
    let tmp = NamedTempFile::new().context("failed to create temp file")?;
    let tmp_path = tmp.path().with_extension("wav");

    let status = Command::new("ffmpeg")
        .args([
            "-i",
            input.to_str().unwrap_or_default(),
            "-ar",
            "16000",
            "-ac",
            "1",
            "-f",
            "wav",
            "-y", // overwrite without prompting
            tmp_path.to_str().unwrap_or_default(),
        ])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                AppError::FfmpegNotFound
            } else {
                AppError::FfmpegFailed(e.to_string())
            }
        })?;

    if !status.status.success() {
        let stderr = String::from_utf8_lossy(&status.stderr).to_string();
        return Err(AppError::FfmpegFailed(stderr).into());
    }

    read_wav_as_f32(&tmp_path)
}

fn read_wav_as_f32(path: &Path) -> Result<Vec<f32>> {
    let mut reader = hound::WavReader::open(path).context("failed to open converted WAV")?;
    let spec = reader.spec();

    let samples: Vec<f32> = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .collect::<std::result::Result<Vec<_>, _>>()
            .context("failed to read f32 WAV samples")?,
        hound::SampleFormat::Int => reader
            .samples::<i16>()
            .map(|s| s.map(|v| v as f32 / 32768.0))
            .collect::<std::result::Result<Vec<_>, _>>()
            .context("failed to read i16 WAV samples")?,
    };

    Ok(samples)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn rejects_missing_file() {
        let err = validate(&PathBuf::from("nonexistent.mp3")).unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn rejects_unsupported_extension() {
        // Create a temp file with unsupported extension to pass the existence check
        let tmp = tempfile::Builder::new().suffix(".xyz").tempfile().unwrap();
        let err = validate(tmp.path()).unwrap_err();
        assert!(err.to_string().contains("Unsupported format"));
    }

    #[test]
    fn accepts_supported_extensions() {
        for ext in SUPPORTED_EXTENSIONS {
            let tmp = tempfile::Builder::new()
                .suffix(&format!(".{ext}"))
                .tempfile()
                .unwrap();
            // File exists and has a valid extension — validation should pass
            assert!(validate(tmp.path()).is_ok(), "should accept .{ext}");
        }
    }
}
