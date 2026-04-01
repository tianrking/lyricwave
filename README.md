# lyricwave

<p align="center">
  <img src="https://img.shields.io/badge/Rust-2024%20Edition-black?logo=rust" alt="Rust" />
  <img src="https://img.shields.io/badge/CLI-Cross%20Platform-1f6feb" alt="CLI" />
  <img src="https://img.shields.io/github/actions/workflow/status/tianrking/lyricwave/ci.yml?branch=main&label=CI" alt="CI" />
  <img src="https://img.shields.io/github/actions/workflow/status/tianrking/lyricwave/release.yml?branch=main&label=Release" alt="Release" />
  <img src="https://img.shields.io/github/v/release/tianrking/lyricwave" alt="Latest Release" />
  <img src="https://img.shields.io/github/license/tianrking/lyricwave" alt="License" />
</p>

Cross-platform audio capture CLI for:
- system mix capture
- per-app capture (single/multiple)
- split per-app WAV export
- ASR + translation pipeline integration

Language:
- English (this file)
- [简体中文 README](./README.zh-CN.md)

## First-Time Quick Start

If this is your first time, start with these 3 commands:

```bash
lyricwave capture system --out /tmp/system.wav --seconds 10
lyricwave visual system --out /tmp/system.mp4 --seconds 10
lyricwave record system --audio-out /tmp/a.wav --visual-out /tmp/v.mp4 --seconds 10
```

You can ignore `daemon` commands unless you are building your own overlay/app integration.

## Download And Use (Recommended)

Get prebuilt binaries from [Releases](https://github.com/tianrking/lyricwave/releases):

- Linux: `.tar.gz` for `x86_64/i686/aarch64/armv7` + `.deb` for `x86_64`
- macOS: `.tar.gz` for Intel + Apple Silicon
- Windows: `.zip` with `lyricwave.exe` for `x86_64/i686`

Then run:

```bash
lyricwave --help
```

Detailed CLI usage:
- [Usage Guide (English)](./docs/USAGE.md)
- [使用手册（中文）](./docs/USAGE.zh-CN.md)
- [Architecture (English)](./docs/ARCHITECTURE.md)
- [架构说明（中文）](./docs/ARCHITECTURE.zh-CN.md)

## Build From Source

```bash
cargo build --workspace --release
```

## Platform Support

| Capability | macOS | Linux | Windows |
|---|---|---|---|
| System mix capture (`capture system`) | Yes (loopback input/device required) | Yes (loopback/monitor input may be required) | Yes |
| Per-app capture (`capture app`) | Yes (ScreenCaptureKit, requires Screen Recording permission) | Yes (PulseAudio/PipeWire) | Yes (WASAPI process loopback) |
| Active app discovery (`capture apps-list`) | Yes (capture candidates from ScreenCaptureKit) | Yes (active sink-input processes) | Yes (active WASAPI session processes) |
| Split per-app WAV (`capture apps-split`) | Yes | Yes | Yes |

## Feature Snapshot

### Audio
- device list + capability inspect
- auto input selection strategy: `hint > loopback > default > first`
- system capture to WAV / stdout PCM
- app capture (single or multiple selectors)
- split capture to independent per-process WAV files
- optional merged mix output from split files

### Pipeline
- pluggable provider registry
- offline VibeVoice file ASR integration (external repo)
- translators: `mock`, `passthrough`, `deepl`, `libretranslate`
- one-shot pipeline command (`pipeline run-once`)

### Daemon
- For developers building a live overlay/client, not required for normal recording usage
- `daemon run`: print mock subtitle events in terminal
- `daemon serve`: expose mock subtitle events over TCP for external clients

### Visual (Architecture Ready)
- visual backend registry and platform routing (`visual` module)
- display discovery command (`visual displays`)
- display/system capture command (`visual system`) and app routing entrypoint (`visual app`)

### Composition (Top-Level Orchestration)
- unified A/V coordinator (`composition` module) for:
  - audio-only session
  - visual-only session
  - audio + visual parallel session

## Four Core Functions

### 1) Record Audio
Use this when you only need sound (system mix, single app, multi-app mix, or per-app split files).

Common scenarios:
- game/system sound recording
- browser/music app recording
- collect each app into independent WAV files

Prerequisites:
- macOS app capture: grant Screen Recording permission for terminal host app
- Linux app capture: install `pactl` and `parecord`

Examples:

```bash
# system audio
lyricwave capture system --out system.wav --seconds 10

# single app audio
lyricwave capture app --out chrome.wav --name "Google Chrome" --seconds 10

# multi-app mixed audio
lyricwave capture app --out mixed.wav --name "Google Chrome" --name "Music" --seconds 10

# per-app independent audio files (+ optional merged mix)
lyricwave capture apps-split \
  --out-dir /tmp/audio-split \
  --seconds 10 \
  --all-active \
  --mix-out /tmp/audio-mix.wav
```

### 2) Record Visual (Screen/App Frames)
Use this when you only need image stream/frame output.

Common scenarios:
- screen-only recording
- app-oriented visual capture
- per-app independent visual files

Prerequisites:
- visual per-app routing depends on OS-native backend status and may return `NotImplemented` on some platforms

Examples:

```bash
# system visual
lyricwave visual system --out system.mp4 --seconds 10

# single app visual
lyricwave visual app --out chrome.mp4 --name "Google Chrome" --seconds 10

# list active visual process candidates
lyricwave visual apps-list

# per-app independent visual files
lyricwave visual apps-split --out-dir /tmp/visual-split --seconds 10 --all-active
```

### 3) Speech To Text (ASR)
Use this when you need transcript output from captured audio.

Common scenarios:
- turn recorded audio into text
- run subtitle/transcript generation pipeline
- optional translation after ASR

Prerequisites:
- prepare local VibeVoice repo path
- have Python runtime available
- set provider options (`--asr-provider`, `--vibevoice-dir`, `--model-path`)

Examples:

```bash
# ASR from existing audio file
lyricwave pipeline asr-file \
  --audio /path/audio.wav \
  --asr-provider vibevoice \
  --vibevoice-dir /absolute/path/to/VibeVoice

# capture system audio -> ASR -> translation (one-shot)
lyricwave pipeline run-once \
  --seconds 8 \
  --asr-provider vibevoice \
  --vibevoice-dir /absolute/path/to/VibeVoice \
  --python-bin python \
  --target-lang zh \
  --translator-provider mock
```

### 4) Unified A/V Recording (Video Workflow)
Use this when you need audio + visual together, while still allowing separated files per app.

Common scenarios:
- system-level A/V recording
- selected app A/V recording
- per-app split A/V outputs
- then run ASR on produced audio files

Examples:

```bash
# system audio + visual together
lyricwave record system --audio-out system.wav --visual-out system.mp4 --seconds 10

# selected app audio + visual together
lyricwave record app --audio-out app.wav --visual-out app.mp4 --name "Google Chrome" --seconds 10

# split mode: each app => one audio + one visual output
lyricwave record apps-split --out-dir /tmp/compose-split --seconds 10 --all-active

# then run ASR on a generated audio file
lyricwave pipeline asr-file \
  --audio /tmp/compose-split/Google_Chrome-12345.wav \
  --asr-provider vibevoice \
  --vibevoice-dir /absolute/path/to/VibeVoice
```

## CI And Release

- CI workflow: `.github/workflows/ci.yml`
- Release workflow: `.github/workflows/release.yml`
- Tag release trigger: push `v*` tag (example: `v0.1.0`)

## Architecture

- `crates/lyricwave-core`
  - `audio`: audio capture domain
  - `visual`: visual frame capture domain (display/window stream)
  - `composition`: top-level A/V session orchestration (compose audio/visual)
  - `pipeline`: ASR/translation processing domain (post-capture flow)
  - `service`: pipeline service helpers
- `crates/lyricwave-cli`
  - `cli.rs`: command model
  - `commands/*`: command handlers
  - `main.rs`: dispatcher


## Troubleshooting

### macOS `capture app` fails with `SCStreamErrorDomain -3801`
Enable `Screen Recording` permission for Terminal/host app:
- `System Settings -> Privacy & Security -> Screen Recording`

### Linux `capture app` fails
Check `pactl` / `parecord` availability and whether target app currently has active sink-input audio.

### Windows `capture app` gets empty output
Ensure the selected process is actively producing audio during recording and is not muted.
