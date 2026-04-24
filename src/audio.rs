use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result};
use rubato::{FftFixedIn, Resampler};
use symphonia::core::audio::{Channels, SampleBuffer, SignalSpec};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use crate::error::AppError;

const SUPPORTED_EXTENSIONS: &[&str] = &["mp3", "wav", "m4a", "mp4", "ogg", "flac"];
const TARGET_RATE: u32 = 16_000;
const RESAMPLE_CHUNK: usize = 1024;

/// Validate the input file and convert it to 16kHz mono f32 PCM samples.
pub fn load(input: &Path) -> Result<Vec<f32>> {
    validate(input)?;
    decode_to_mono_16k(input)
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

fn decode_to_mono_16k(input: &Path) -> Result<Vec<f32>> {
    let file = File::open(input).context("failed to open input file")?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = input.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| AppError::DecodeFailed(e.to_string()))?;

    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or_else(|| AppError::DecodeFailed("no decodable audio track".into()))?;
    let track_id = track.id;

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| AppError::DecodeFailed(e.to_string()))?;

    let mut interleaved: Vec<f32> = Vec::new();
    let mut spec: Option<SignalSpec> = None;
    let mut sample_buf: Option<SampleBuffer<f32>> = None;

    loop {
        let packet = match format.next_packet() {
            Ok(p) => p,
            Err(SymphoniaError::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(SymphoniaError::ResetRequired) => {
                decoder = symphonia::default::get_codecs()
                    .make(&decoder.codec_params().clone(), &DecoderOptions::default())
                    .map_err(|e| AppError::DecodeFailed(e.to_string()))?;
                sample_buf = None;
                spec = None;
                continue;
            }
            Err(e) => return Err(AppError::DecodeFailed(e.to_string()).into()),
        };

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                let buf_spec = *decoded.spec();
                if spec.is_none() {
                    spec = Some(buf_spec);
                }
                let sbuf = sample_buf.get_or_insert_with(|| {
                    SampleBuffer::<f32>::new(decoded.capacity() as u64, buf_spec)
                });
                if decoded.capacity() > sbuf.capacity() {
                    *sbuf = SampleBuffer::<f32>::new(decoded.capacity() as u64, buf_spec);
                }
                sbuf.copy_interleaved_ref(decoded);
                interleaved.extend_from_slice(sbuf.samples());
            }
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(SymphoniaError::IoError(ref e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(e) => return Err(AppError::DecodeFailed(e.to_string()).into()),
        }
    }

    let spec = spec.ok_or_else(|| AppError::DecodeFailed("no audio frames decoded".into()))?;
    if interleaved.is_empty() {
        return Err(AppError::DecodeFailed("no audio frames decoded".into()).into());
    }

    let channels = spec.channels;
    let channel_count = channels.count();
    let mono = downmix_to_mono(&interleaved, channels);

    if spec.rate == TARGET_RATE && channel_count == 1 {
        return Ok(mono);
    }

    resample_to_target(&mono, spec.rate)
}

fn downmix_to_mono(interleaved: &[f32], channels: Channels) -> Vec<f32> {
    let channel_count = channels.count();
    if channel_count <= 1 {
        return interleaved.to_vec();
    }

    let frames = interleaved.len() / channel_count;
    let mut mono = Vec::with_capacity(frames);

    debug_assert_eq!(
        interleaved.len() % channel_count,
        0,
        "interleaved sample count is not a multiple of channel_count"
    );

    if channel_count == 2 {
        for frame in interleaved.chunks_exact(channel_count) {
            mono.push((frame[0] + frame[1]) * 0.5);
        }
        return mono;
    }

    // ITU-R BS.775 coefficients for surround → mono. Weights follow the
    // bit-order of `Channels::iter()`. Unknown channels contribute zero.
    let weights: Vec<f32> = channels.iter().map(bs775_weight).collect();
    let weight_sum: f32 = weights.iter().sum();

    if weight_sum < f32::EPSILON {
        // Fallback: no recognised speaker positions — arithmetic mean.
        for frame in interleaved.chunks_exact(channel_count) {
            let sum: f32 = frame.iter().sum();
            mono.push(sum / channel_count as f32);
        }
        return mono;
    }

    for frame in interleaved.chunks_exact(channel_count) {
        let mixed: f32 = frame.iter().zip(weights.iter()).map(|(s, w)| s * w).sum();
        mono.push(mixed / weight_sum);
    }
    mono
}

fn bs775_weight(channel: Channels) -> f32 {
    match channel {
        Channels::FRONT_LEFT | Channels::FRONT_RIGHT => 0.707,
        Channels::FRONT_CENTRE => 1.0,
        Channels::LFE1 | Channels::LFE2 => 0.0,
        Channels::REAR_LEFT | Channels::REAR_RIGHT => 0.5,
        Channels::SIDE_LEFT | Channels::SIDE_RIGHT => 0.5,
        Channels::REAR_CENTRE => 0.5,
        _ => 0.0,
    }
}

fn resample_to_target(mono: &[f32], src_rate: u32) -> Result<Vec<f32>> {
    if src_rate == TARGET_RATE {
        return Ok(mono.to_vec());
    }

    let mut resampler = FftFixedIn::<f32>::new(
        src_rate as usize,
        TARGET_RATE as usize,
        RESAMPLE_CHUNK,
        2,
        1,
    )
    .map_err(|e| AppError::ResampleFailed(e.to_string()))?;

    let mut output: Vec<f32> =
        Vec::with_capacity(mono.len() * TARGET_RATE as usize / src_rate as usize + RESAMPLE_CHUNK);
    let mut chunk_out = vec![vec![0.0f32; resampler.output_frames_max()]];
    let mut cursor = 0;

    while mono.len() - cursor >= RESAMPLE_CHUNK {
        let chunk_in = [&mono[cursor..cursor + RESAMPLE_CHUNK]];
        let (_, written) = resampler
            .process_into_buffer(&chunk_in, &mut chunk_out, None)
            .map_err(|e| AppError::ResampleFailed(e.to_string()))?;
        output.extend_from_slice(&chunk_out[0][..written]);
        cursor += RESAMPLE_CHUNK;
    }

    // Flush tail — pass None when empty to avoid rubato clearing the padded buffer to len 0.
    let tail = &mono[cursor..];
    let tail_arg: Option<&[&[f32]]> = if tail.is_empty() { None } else { Some(&[tail]) };
    let (_, written) = resampler
        .process_partial_into_buffer(tail_arg, &mut chunk_out, None)
        .map_err(|e| AppError::ResampleFailed(e.to_string()))?;
    output.extend_from_slice(&chunk_out[0][..written]);

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hound::{SampleFormat, WavSpec, WavWriter};
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    #[test]
    fn rejects_missing_file() {
        let err = validate(&PathBuf::from("nonexistent.mp3")).unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn rejects_unsupported_extension() {
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
            assert!(validate(tmp.path()).is_ok(), "should accept .{ext}");
        }
    }

    fn write_i16_wav(samples: &[i16], rate: u32, channels: u16) -> NamedTempFile {
        let tmp = tempfile::Builder::new().suffix(".wav").tempfile().unwrap();
        let spec = WavSpec {
            channels,
            sample_rate: rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
        let mut writer = WavWriter::create(tmp.path(), spec).unwrap();
        for s in samples {
            writer.write_sample(*s).unwrap();
        }
        writer.finalize().unwrap();
        tmp
    }

    #[test]
    fn decodes_16k_mono_wav_as_identity() {
        let samples: Vec<i16> = (0..1600)
            .map(|i| ((i as f32).sin() * 16_000.0) as i16)
            .collect();
        let tmp = write_i16_wav(&samples, 16_000, 1);

        let out = load(tmp.path()).unwrap();
        assert_eq!(out.len(), 1600);
        assert!(out.iter().all(|s| (-1.0..=1.0).contains(s)));
    }

    #[test]
    fn resamples_44100_mono_wav_to_16k() {
        let samples: Vec<i16> = (0..4410)
            .map(|i| ((i as f32 * 0.1).sin() * 16_000.0) as i16)
            .collect();
        let tmp = write_i16_wav(&samples, 44_100, 1);

        let out = load(tmp.path()).unwrap();
        let expected = 1600;
        let tolerance = 256;
        assert!(
            out.len().abs_diff(expected) <= tolerance,
            "got {}, expected ~{}",
            out.len(),
            expected
        );
    }

    #[test]
    fn downmixes_stereo_wav_to_mono() {
        // Interleaved L=+16000, R=-16000 → mean 0.
        let mut samples = Vec::with_capacity(3200);
        for _ in 0..1600 {
            samples.push(16_000i16);
            samples.push(-16_000i16);
        }
        let tmp = write_i16_wav(&samples, 16_000, 2);

        let out = load(tmp.path()).unwrap();
        assert_eq!(out.len(), 1600);
        let mean: f32 = out.iter().sum::<f32>() / out.len() as f32;
        assert!(mean.abs() < 1e-4, "mean was {mean}");
    }

    #[test]
    fn resamples_chunk_aligned_input() {
        // 2*RESAMPLE_CHUNK frames at 44.1kHz — tail is empty after the main loop,
        // exercising the None branch in process_partial_into_buffer.
        let samples: Vec<i16> = (0..2 * RESAMPLE_CHUNK)
            .map(|i| ((i as f32 * 0.1).sin() * 16_000.0) as i16)
            .collect();
        let tmp = write_i16_wav(&samples, 44_100, 1);

        let out = load(tmp.path()).unwrap();
        assert!(!out.is_empty(), "chunk-aligned input produced no output");
    }

    #[test]
    fn resamples_short_sub_chunk_input() {
        // 500 frames at 44.1kHz — shorter than RESAMPLE_CHUNK (1024).
        let samples: Vec<i16> = (0..500)
            .map(|i| ((i as f32 * 0.1).sin() * 16_000.0) as i16)
            .collect();
        let tmp = write_i16_wav(&samples, 44_100, 1);

        let out = load(tmp.path()).unwrap();
        assert!(!out.is_empty(), "sub-chunk input produced no output");
    }

    #[test]
    fn downmixes_surround_5_1_to_mono() {
        // 5.1 layout: FL, FR, FC, LFE, RL, RR (6 channels).
        // FC has weight 1.0; FL/FR = 0.707; RL/RR = 0.5; LFE = 0.0.
        // With all channels at 1.0: mixed = 0.707+0.707+1.0+0.0+0.5+0.5 = 3.414,
        // weight_sum = same → output = 1.0 per frame.
        let channels = Channels::FRONT_LEFT
            | Channels::FRONT_RIGHT
            | Channels::FRONT_CENTRE
            | Channels::LFE1
            | Channels::REAR_LEFT
            | Channels::REAR_RIGHT;
        let channel_count = channels.count();
        let frames = 100;
        let interleaved = vec![1.0f32; frames * channel_count];

        let mono = downmix_to_mono(&interleaved, channels);
        assert_eq!(mono.len(), frames);
        for s in &mono {
            assert!((s - 1.0).abs() < 1e-5, "expected 1.0, got {s}");
        }
    }

    #[test]
    fn downmixes_7_1_to_mono() {
        // 7.1 layout adds SIDE_LEFT and SIDE_RIGHT to 5.1.
        // SL/SR weight = 0.5; all channels at 1.0 → output = weight_sum/weight_sum = 1.0.
        let channels = Channels::FRONT_LEFT
            | Channels::FRONT_RIGHT
            | Channels::FRONT_CENTRE
            | Channels::LFE1
            | Channels::REAR_LEFT
            | Channels::REAR_RIGHT
            | Channels::SIDE_LEFT
            | Channels::SIDE_RIGHT;
        let channel_count = channels.count();
        let frames = 100;
        let interleaved = vec![1.0f32; frames * channel_count];

        let mono = downmix_to_mono(&interleaved, channels);
        assert_eq!(mono.len(), frames);
        for s in &mono {
            assert!((s - 1.0).abs() < 1e-5, "expected 1.0, got {s}");
        }
    }

    #[test]
    fn downmixes_rear_centre_to_mono() {
        // FL, FR, FC, REAR_CENTRE — all at 1.0. REAR_CENTRE weight = 0.5.
        // weight_sum = 0.707+0.707+1.0+0.5 = 2.914 → output = 2.914/2.914 = 1.0.
        let channels = Channels::FRONT_LEFT
            | Channels::FRONT_RIGHT
            | Channels::FRONT_CENTRE
            | Channels::REAR_CENTRE;
        let channel_count = channels.count();
        let frames = 50;
        let interleaved = vec![1.0f32; frames * channel_count];

        let mono = downmix_to_mono(&interleaved, channels);
        assert_eq!(mono.len(), frames);
        for s in &mono {
            assert!((s - 1.0).abs() < 1e-5, "expected 1.0, got {s}");
        }
    }

    #[test]
    fn downmixes_all_zero_weight_channels_uses_arithmetic_mean() {
        // LFE1, LFE2, TOP_CENTRE all have bs775 weight 0 → weight_sum = 0 → arithmetic mean.
        let channels = Channels::LFE1 | Channels::LFE2 | Channels::TOP_CENTRE;
        let frames = 10;
        let interleaved: Vec<f32> = (0..frames).flat_map(|_| [1.0f32, 2.0, 3.0]).collect();

        let mono = downmix_to_mono(&interleaved, channels);
        assert_eq!(mono.len(), frames);
        for s in &mono {
            assert!(
                (s - 2.0).abs() < 1e-5,
                "expected arithmetic mean 2.0, got {s}"
            );
        }
    }

    #[test]
    fn bs775_weight_returns_zero_for_unrecognized_channel() {
        assert_eq!(bs775_weight(Channels::TOP_CENTRE), 0.0);
    }

    #[test]
    fn rejects_corrupt_wav() {
        let tmp = tempfile::Builder::new().suffix(".wav").tempfile().unwrap();
        std::fs::write(tmp.path(), b"junk").unwrap();

        let err = load(tmp.path()).unwrap_err();
        assert!(
            err.to_string().contains("decode") || err.to_string().contains("Failed"),
            "unexpected error: {err}"
        );
    }
}
