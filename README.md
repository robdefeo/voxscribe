# voxscribe

Offline audio transcription using local [Whisper](https://github.com/ggerganov/whisper.cpp) models.

## Installation

```bash
mise install
just build
```

## Usage

```bash
voxscribe <INPUT> [OPTIONS]
```

The first run downloads the selected model from HuggingFace and caches it locally. All subsequent runs work fully offline.

### Models

| Model | Size | Notes |
|---|---|---|
| `large-v3-turbo` | ~809 MB | **Recommended.** Best speed/accuracy balance for most audio. |
| `large-v3` | ~1.5 GB | Maximum accuracy. ~2x slower than turbo. |
| `medium` | ~769 MB | Good accuracy, faster than large. |
| `small` | ~466 MB | Fast, less accurate. |
| `base` | ~141 MB | Very fast, lower accuracy. |
| `tiny` | ~75 MB | Fastest, lowest accuracy. |

### Examples

```bash
# Transcribe to stdout
voxscribe audio.m4a

# Transcribe to file using the recommended model
voxscribe audio.m4a --model large-v3-turbo --output transcript.txt

# SRT subtitles with timestamps
voxscribe audio.mp4 --format srt --timestamps --output subtitles.srt

# Force language and apply a correction dictionary
voxscribe audio.m4a --language en --dict corrections.json

# Use a locally downloaded model file
voxscribe audio.m4a --model-path ~/models/ggml-large-v3-turbo.bin
```

### Options

Run `voxscribe --help` for the full list of options.
