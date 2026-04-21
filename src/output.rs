use serde::Serialize;

use crate::cli::OutputFormat;
use crate::transcribe::Segment;

#[derive(Serialize)]
struct JsonOutput<'a> {
    segments: Vec<JsonSegment<'a>>,
}

#[derive(Serialize)]
struct JsonSegment<'a> {
    start_ms: i64,
    end_ms: i64,
    text: &'a str,
}

pub fn format(segments: &[Segment], fmt: &OutputFormat, timestamps: bool) -> String {
    match fmt {
        OutputFormat::Txt => format_txt(segments, timestamps),
        OutputFormat::Json => format_json(segments),
        OutputFormat::Srt => format_srt(segments),
        OutputFormat::Vtt => format_vtt(segments),
    }
}

fn format_txt(segments: &[Segment], timestamps: bool) -> String {
    segments
        .iter()
        .map(|s| {
            if timestamps {
                format!("[{}] {}", ms_to_hms(s.start_ms), s.text)
            } else {
                s.text.clone()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_json(segments: &[Segment]) -> String {
    let out = JsonOutput {
        segments: segments
            .iter()
            .map(|s| JsonSegment {
                start_ms: s.start_ms,
                end_ms: s.end_ms,
                text: &s.text,
            })
            .collect(),
    };
    serde_json::to_string_pretty(&out).unwrap_or_default()
}

fn format_srt(segments: &[Segment]) -> String {
    segments
        .iter()
        .enumerate()
        .map(|(i, s)| {
            format!(
                "{}\n{} --> {}\n{}\n",
                i + 1,
                ms_to_srt_timestamp(s.start_ms),
                ms_to_srt_timestamp(s.end_ms),
                s.text,
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_vtt(segments: &[Segment]) -> String {
    let cues = segments
        .iter()
        .enumerate()
        .map(|(i, s)| {
            format!(
                "{}\n{} --> {}\n{}\n",
                i + 1,
                ms_to_vtt_timestamp(s.start_ms),
                ms_to_vtt_timestamp(s.end_ms),
                s.text,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("WEBVTT\n\n{cues}")
}

/// Format milliseconds as `HH:MM:SS`
pub fn ms_to_hms(ms: i64) -> String {
    let total_secs = ms / 1000;
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    format!("{h:02}:{m:02}:{s:02}")
}

/// Format milliseconds as `HH:MM:SS.mmm` (WebVTT format)
fn ms_to_vtt_timestamp(ms: i64) -> String {
    let total_secs = ms / 1000;
    let millis = ms % 1000;
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    format!("{h:02}:{m:02}:{s:02}.{millis:03}")
}

/// Format milliseconds as `HH:MM:SS,mmm` (SRT format)
fn ms_to_srt_timestamp(ms: i64) -> String {
    let total_secs = ms / 1000;
    let millis = ms % 1000;
    let h = total_secs / 3600;
    let m = (total_secs % 3600) / 60;
    let s = total_secs % 60;
    format!("{h:02}:{m:02}:{s:02},{millis:03}")
}

// grcov-excl-start: exclude inline unit tests from production coverage
#[cfg(test)]
mod tests {
    use super::*;

    fn seg(start_ms: i64, end_ms: i64, text: &str) -> Segment {
        Segment {
            start_ms,
            end_ms,
            text: text.to_string(),
        }
    }

    #[test]
    fn txt_no_timestamps() {
        let segs = vec![seg(0, 1000, "Hello"), seg(1000, 2000, "world")];
        let out = format(&segs, &OutputFormat::Txt, false);
        assert_eq!(out, "Hello\nworld");
    }

    #[test]
    fn txt_with_timestamps() {
        let segs = vec![seg(0, 1000, "Hello"), seg(61_000, 62_000, "world")];
        let out = format(&segs, &OutputFormat::Txt, true);
        assert!(out.contains("[00:00:00] Hello"));
        assert!(out.contains("[00:01:01] world"));
    }

    #[test]
    fn json_contains_fields() {
        let segs = vec![seg(500, 1500, "Test")];
        let out = format(&segs, &OutputFormat::Json, false);
        assert!(out.contains("\"start_ms\""));
        assert!(out.contains("\"text\""));
        assert!(out.contains("\"Test\""));
    }

    #[test]
    fn srt_format() {
        let segs = vec![seg(0, 1500, "Hello"), seg(2000, 3500, "world")];
        let out = format(&segs, &OutputFormat::Srt, false);
        assert!(out.contains("1\n00:00:00,000 --> 00:00:01,500"));
        assert!(out.contains("2\n00:00:02,000 --> 00:00:03,500"));
    }

    #[test]
    fn vtt_format() {
        let segs = vec![seg(0, 1500, "Hello"), seg(2000, 3500, "world")];
        let out = format(&segs, &OutputFormat::Vtt, false);
        assert!(out.starts_with("WEBVTT\n\n"));
        assert!(out.contains("1\n00:00:00.000 --> 00:00:01.500"));
        assert!(out.contains("2\n00:00:02.000 --> 00:00:03.500"));
    }

    #[test]
    fn ms_to_hms_rounds_correctly() {
        assert_eq!(ms_to_hms(0), "00:00:00");
        assert_eq!(ms_to_hms(3661_000), "01:01:01");
        assert_eq!(ms_to_hms(59_999), "00:00:59");
    }
}
// grcov-excl-stop
