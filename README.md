# lyricwave

`lyricwave` is a Rust-first CLI for cross-platform system audio capture with a subtitle and translation pipeline designed for future desktop overlays.

## What is implemented now

- Cross-platform backend abstraction with capability metadata
- System capture command generation via `ffmpeg` (macOS / Linux / Windows templates)
- Output device discovery via `cpal`
- Structured daemon event stream (`JSONL`) for future floating window clients
- Pluggable ASR / translation interfaces with mock engines

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

# Daemon JSON stream for overlay integration
cargo run -p lyricwave-cli -- daemon run --target-lang zh --interval-ms 500 --count 5
```

## Architecture

- `crates/lyricwave-core`
  - `audio`: backend trait, capability model, capture request, ffmpeg command builder
  - `pipeline`: event schema, event hub, ASR/translator plugin traits
  - `service`: orchestration layer joining ASR + translation + events
- `crates/lyricwave-cli`
  - user-facing command interface and process execution

## Platform note

The capture command is unified, but each OS still depends on local audio stack setup:

- macOS: use a loopback-capable input (for example a virtual device) for full system mix capture
- Linux: PulseAudio/PipeWire monitor source may be needed
- Windows: WASAPI input/endpoint availability depends on system setup
