# lyricwave

`lyricwave` is a Rust-first CLI for cross-platform system audio capture with a future-ready subtitle and translation pipeline.

## Current status

This repository includes:

- Workspace split into `lyricwave-core` and `lyricwave-cli`
- `lyricwave devices list` to inspect output devices
- `lyricwave capture system` command stub with a stable interface
- Event hub primitives for future realtime transcript/translation overlays (including macOS floating window)

## Quick start

```bash
cargo run -p lyricwave-cli -- devices list
cargo run -p lyricwave-cli -- capture system
```

## Planned milestones

1. Implement loopback capture backend per platform
   - Windows: WASAPI loopback
   - macOS: CoreAudio + virtual device path
   - Linux: PipeWire / PulseAudio monitor
2. Add realtime ASR and translation plugin traits
3. Add daemon mode + websocket stream for overlay clients
4. Build macOS floating subtitle overlay app on top of event stream
