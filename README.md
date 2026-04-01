# lyricwave

`lyricwave` is a Rust-first CLI for cross-platform system audio capture with ASR + translation pipeline primitives and overlay-ready event streaming.

## Project Status

- Architecture: modular and extensible (`audio/backends` + `pipeline/providers`)
- Core offline workflow: implemented (`capture -> asr -> translate`)
- Cross-platform code paths: macOS / Linux / Windows implemented
- Cross-platform CI: enabled (GitHub Actions matrix)

## Feature Matrix (Current)

### Audio / Capture

- [x] List output + input devices
- [x] Build system capture command templates for macOS/Linux/Windows
- [x] Capture to WAV
- [x] Auto-select input by strategy: `hint > loopback > default > first`
- [ ] Native FLAC output
- [x] Stream raw PCM to stdout
- [x] Select audio backend by id (`--audio-backend`)
- [ ] Per-app/process capture
- [ ] True per-OS loopback endpoints (WASAPI loopback/CoreAudio tap/PipeWire monitor) without manual routing

### ASR / Translation

- [x] Provider registry (directory-based, pluggable)
- [x] Offline external VibeVoice file ASR provider
- [x] Translator providers: `mock`, `passthrough`, `deepl`, `libretranslate`
- [x] One-shot offline main flow (`pipeline run-once`)
- [ ] Online ASR providers (OpenAI/Deepgram) runtime implementation
- [ ] Streaming ASR in daemon mode

### Daemon / Integration

- [x] JSONL event output (`daemon run`)
- [x] TCP JSONL event stream (`daemon serve`)
- [ ] WebSocket transport
- [ ] macOS floating overlay UI client

## Platform Support

Code-level support is implemented for all major desktop OS audio stacks via backend templates:

- macOS: `avfoundation` input template (loopback-capable device required)
- Linux: `pulse` input template (PulseAudio/PipeWire monitor may be required)
- Windows: `wasapi` input template

Notes:

- macOS has been the primary local runtime environment during development.
- Linux/Windows are covered by code paths and CI builds, but real-device behavioral validation is still recommended per environment.

## Installation

### Prerequisites

- Rust stable toolchain
- Optional for offline ASR:
  - local checkout of `microsoft/VibeVoice`
  - Python environment that can run VibeVoice ASR script

### Build

```bash
cargo build --workspace
```

### Quick Validation

```bash
cargo run -p lyricwave-cli -- backends list
cargo run -p lyricwave-cli -- providers list
cargo run -p lyricwave-cli -- devices list
```

## Core Commands

```bash
# Audio backend catalog
cargo run -p lyricwave-cli -- backends list

# Provider catalog
cargo run -p lyricwave-cli -- providers list

# Explicit backend selection
cargo run -p lyricwave-cli -- --audio-backend cpal-native devices list

# Capture system audio to file (native CPAL path)
cargo run -p lyricwave-cli -- capture system --out out.wav --seconds 10

# Capture with explicit input-device hint
cargo run -p lyricwave-cli -- capture system --out out.wav --seconds 10 --input-device "BlackHole"

# Disable loopback-first selection (fallback to default/first unless hint matches)
cargo run -p lyricwave-cli -- capture system --out out.wav --seconds 10 --no-prefer-loopback

# Manual stop recording (press Enter or Ctrl+C to stop)
cargo run -p lyricwave-cli -- capture system --out out.wav

# Main one-shot workflow: capture -> ASR -> translate -> JSON
cargo run -p lyricwave-cli -- pipeline run-once \
  --seconds 8 \
  --asr-provider vibevoice \
  --vibevoice-dir /absolute/path/to/VibeVoice \
  --python-bin python \
  --target-lang zh \
  --translator-provider mock

# run-once also supports device hint/selection policy
cargo run -p lyricwave-cli -- pipeline run-once \
  --seconds 8 \
  --input-device "Stereo Mix" \
  --no-prefer-loopback \
  --asr-provider vibevoice \
  --vibevoice-dir /absolute/path/to/VibeVoice

# Daemon JSON stream
cargo run -p lyricwave-cli -- daemon run --target-lang zh --interval-ms 500 --count 5

# Daemon TCP JSONL stream
cargo run -p lyricwave-cli -- daemon serve --host 127.0.0.1 --port 7878 --target-lang zh
```

## Provider Configuration

### DeepL

- `DEEPL_API_KEY` (required)
- `DEEPL_BASE_URL` (optional, default: `https://api-free.deepl.com`)

### LibreTranslate

- `LIBRETRANSLATE_BASE_URL` (optional, default: `http://127.0.0.1:5000`)
- `LIBRETRANSLATE_API_KEY` (optional)

### VibeVoice ASR (external mode)

- `lyricwave` does not vendor `microsoft/VibeVoice` source code.
- Provide local path via `--vibevoice-dir`.
- Invoked entrypoint:
  - `python demo/vibevoice_asr_inference_from_file.py --model_path ... --audio_files ...`

## Architecture

- `crates/lyricwave-core`
  - `audio`
    - backend trait + backend registry
    - `audio/backends/` one backend per file
    - `audio/selection/` input selection strategy and loopback scoring
    - `audio/backends/platform/` OS strategy modules
  - `pipeline`
    - event schema + event hub + traits
    - `pipeline/providers/` one provider per file + central registry
  - `service`
    - orchestration layer combining ASR + translation + events
- `crates/lyricwave-cli`
  - `cli.rs`: command definitions
  - `commands/*`: command handlers by domain
  - `main.rs`: thin dispatcher

## CI

Workflow file:

- `.github/workflows/ci.yml`

CI runs on push/PR to `main` and includes:

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets`
- `cargo check --workspace`
- `cargo test --workspace`

Matrix:

- `ubuntu-latest`
- `macos-latest`
- `windows-latest`

Additional architecture smoke checks:

- `x86_64-unknown-linux-gnu` (required)
- `i686-unknown-linux-gnu` (experimental, non-blocking)
- `aarch64-unknown-linux-gnu` (experimental, non-blocking)
- `armv7-unknown-linux-gnueabihf` (experimental, non-blocking)

## Release Packaging

Release workflow file:

- `.github/workflows/release.yml`

Trigger:

- Push tag `v*` (for example: `v0.1.0`)
- Or manual `workflow_dispatch` with tag input

Release artifacts:

- Linux:
  - `x86_64-unknown-linux-gnu`: `.tar.gz` + `.deb`
  - `i686-unknown-linux-gnu`: `.tar.gz`
  - `aarch64-unknown-linux-gnu`: `.tar.gz`
  - `armv7-unknown-linux-gnueabihf`: `.tar.gz`
- macOS:
  - `x86_64-apple-darwin`: `.tar.gz`
  - `aarch64-apple-darwin`: `.tar.gz`
- Windows:
  - `x86_64-pc-windows-msvc`: `.zip` (contains `.exe`)
  - `i686-pc-windows-msvc`: `.zip` (contains `.exe`)

Each release also includes `SHA256SUMS.txt`.

## Troubleshooting

### `capture` fails on macOS with no usable input

Install/configure a loopback-capable virtual audio device and pass correct input selector/device hint.

### `pipeline asr-file` fails for vibevoice

Check:

- `--vibevoice-dir` points to valid VibeVoice checkout
- Python env has required dependencies
- `--model-path` matches installed/available model

### `deepl` provider says missing key

Set `DEEPL_API_KEY` in your shell before running command.

### `libretranslate` returns HTTP error

Check `LIBRETRANSLATE_BASE_URL` and whether your server/public endpoint requires API key.
