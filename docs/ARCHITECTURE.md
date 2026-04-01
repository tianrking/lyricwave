# lyricwave Architecture

## Design Goal

Keep audio capture and video capture as parallel domains, then compose them only at a higher layer.

This gives us:
- audio-only workflows
- video-only workflows
- unified audio+video workflows
- clear extension points for future processing and providers

## Core Module Layout

`crates/lyricwave-core/src`

- `audio/`
  - Audio domain only.
  - Owns audio backend traits, platform implementations, selection strategy, app/system capture.
- `video/`
  - Video domain only.
  - Owns video backend traits, platform implementations, display/screen capture.
- `recording/`
  - A/V orchestration layer.
  - Accepts optional audio and optional video requests in one session.
  - Supports audio-only, video-only, or audio+video parallel execution.
- `pipeline/`
  - Text pipeline domain only (ASR/translation providers, events, registry).
  - Should not depend on platform capture internals.
- `service.rs`
  - Pipeline orchestration helpers for transcript/translation flow.

## Dependency Direction

Recommended dependency direction inside `lyricwave-core`:

`audio` and `video` (leaf capture domains) -> `recording` (composition)  
`pipeline` (processing domain) independent from capture backends

CLI composes everything in command handlers:
- `capture ...` commands use `audio`
- `video ...` commands use `video`
- `record run ...` uses `recording` for unified session
- `pipeline ...` commands use `pipeline`/`service`

## Why `pipeline` Is Separate

`pipeline` is not another media capture backend.
It is a post-capture processing layer (ASR, translation, events), so it should stay separate from `audio` and `video`.

## Evolution Path

This layout is ready for:
- native per-platform video capture upgrades
- A/V synchronization metadata in `recording`
- optional mux/export layers
- optional realtime overlay modules (transcript + translation UI)
