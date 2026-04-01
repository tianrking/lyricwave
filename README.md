# lyricwave

`lyricwave` is a Rust-first CLI for cross-platform system audio capture with a subtitle and translation pipeline designed for future desktop overlays.

## What is implemented now

- Cross-platform backend abstraction with capability metadata
- System capture command generation via `ffmpeg` (macOS / Linux / Windows templates)
- Output device discovery via `cpal`
- Structured daemon event stream (`JSONL`) for future floating window clients
- Pluggable ASR / translation interfaces with mock engines
- External offline VibeVoice ASR provider integration (via process invocation)

## CLI commands

```bash
# List devices and backend capability notes
cargo run -p lyricwave-cli -- devices list

# Capture system audio to WAV (requires ffmpeg in PATH)
cargo run -p lyricwave-cli -- capture system --out out.wav --seconds 10

# Capture to FLAC
cargo run -p lyricwave-cli -- capture system --out out.flac --format flac --seconds 10

# Stream raw PCM to stdout
cargo run -p lyricwave-cli -- capture system --stdout --seconds 5 > out.pcm

# Pipeline demo (mock ASR + mock translation)
cargo run -p lyricwave-cli -- pipeline demo --text "hello from lyricwave" --target-lang zh

# Offline ASR via external VibeVoice repo (no vendoring in lyricwave)
cargo run -p lyricwave-cli -- pipeline asr-file-vibevoice \
  --audio ./sample.wav \
  --vibevoice-dir /absolute/path/to/VibeVoice \
  --model-path microsoft/VibeVoice-ASR \
  --python-bin python \
  --target-lang zh

# Daemon JSON stream for overlay integration
cargo run -p lyricwave-cli -- daemon run --target-lang zh --interval-ms 500 --count 5

# Daemon TCP JSONL stream for GUI overlay clients
cargo run -p lyricwave-cli -- daemon serve --host 127.0.0.1 --port 7878 --target-lang zh
# Then connect from another terminal:
nc 127.0.0.1 7878
```

## Architecture

- `crates/lyricwave-core`
  - `audio`: backend trait, capability model, capture request, ffmpeg command builder
  - `pipeline`: event schema, event hub, ASR/translator plugin traits, external VibeVoice adapter
  - `service`: orchestration layer joining ASR + translation + events
- `crates/lyricwave-cli`
  - user-facing command interface and process execution

## Platform note

The capture command is unified, but each OS still depends on local audio stack setup:

- macOS: use a loopback-capable input (for example a virtual device) for full system mix capture
- Linux: PulseAudio/PipeWire monitor source may be needed
- Windows: WASAPI input/endpoint availability depends on system setup

## VibeVoice external mode notes

- `lyricwave` does not include `microsoft/VibeVoice` source code.
- You provide a local VibeVoice checkout via `--vibevoice-dir`.
- `lyricwave` invokes this VibeVoice entrypoint:
  - `python demo/vibevoice_asr_inference_from_file.py --model_path ... --audio_files ...`
