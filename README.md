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
- JSONL event output (`daemon run`)
- TCP JSONL streaming (`daemon serve`)

### Video (Architecture Ready)
- video backend registry and platform routing (`video` module)
- display discovery command (`video displays`)
- screen capture command scaffold (`video capture-screen`) for unified future A/V orchestration

## Quick CLI Examples

```bash
# list backends/providers/devices
lyricwave backends list
lyricwave providers list
lyricwave devices list

# list current active/candidate app processes
lyricwave capture apps-list

# video backend and display discovery
lyricwave video backends
lyricwave video displays

# video capture scaffold command (native implementation in progress)
lyricwave video capture-screen --out screen.mp4 --seconds 10

# capture system mix (10s)
lyricwave capture system --out system.wav --seconds 10

# capture one app by name
lyricwave capture app --out chrome.wav --name "Google Chrome" --seconds 10

# capture multiple apps into one mixed file
lyricwave capture app --out apps.wav --name "Google Chrome" --name "Music" --seconds 10

# split capture: one file per app + optional merged mix
lyricwave capture apps-split \
  --out-dir /tmp/lyricwave-split \
  --seconds 10 \
  --name "Google Chrome" \
  --name "Music" \
  --mix-out /tmp/lyricwave-mix.wav
```

## CI And Release

- CI workflow: `.github/workflows/ci.yml`
- Release workflow: `.github/workflows/release.yml`
- Tag release trigger: push `v*` tag (example: `v0.1.0`)

## Architecture

- `crates/lyricwave-core`
  - `audio`: backend traits, platform implementations, selection strategy
  - `pipeline`: provider abstractions/registry/events
  - `service`: orchestration layer
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
