# lyricwave Architecture

## Design Goal

Keep audio capture and visual capture as parallel domains, then compose them only at a higher layer.

This gives us:
- audio-only workflows
- visual-only workflows
- unified audio+visual workflows
- clear extension points for future processing and providers

## Core Module Layout

`crates/lyricwave-core/src`

- `audio/`
  - Audio domain only.
  - Owns audio backend traits, platform implementations, selection strategy, app/system capture.
- `visual/`
  - Visual domain only.
  - Owns visual backend traits, platform implementations, display/screen capture.
- `composition/`
  - A/V orchestration layer.
  - Accepts optional audio and optional visual requests in one session.
  - Supports audio-only, visual-only, or audio+visual parallel execution.
- `pipeline/`
  - Text pipeline domain only (ASR/translation providers, events, registry).
  - Should not depend on platform capture internals.
- `service.rs`
  - Pipeline orchestration helpers for transcript/translation flow.

## Dependency Direction

Recommended dependency direction inside `lyricwave-core`:

`audio` and `visual` (leaf capture domains) -> `composition` (session composition)  
`pipeline` (processing domain) independent from capture backends

CLI composes everything in command handlers:
- `capture ...` commands use `audio`
- `visual ...` commands use `visual`
- `record run ...` uses `composition` for unified session
- `pipeline ...` commands use `pipeline`/`service`

## Why `pipeline` Is Separate

`pipeline` is not another media capture backend.
It is a post-capture processing layer (ASR, translation, events), so it should stay separate from `audio` and `visual`.

## Evolution Path

This layout is ready for:
- native per-platform visual capture upgrades
- A/V synchronization metadata in `composition`
- optional mux/export layers
- optional realtime overlay modules (transcript + translation UI)
