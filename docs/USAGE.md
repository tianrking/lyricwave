# lyricwave CLI Usage

- Chinese version: [USAGE.zh-CN.md](./USAGE.zh-CN.md)

## 1. Install

Option A (recommended): download prebuilt package from [Releases](https://github.com/tianrking/lyricwave/releases).

Option B: build from source.

```bash
cargo build --workspace --release
```

## 2. Global Option

```bash
--audio-backend <ID>
--visual-backend <ID>
```

Current default is `cpal-native`.

## 3. Four-Function Map

- Record Audio: section 4-6 (`capture system/app/apps-split`)
- Record Visual: section 10 (`visual system/app/apps-list/apps-split`)
- Speech To Text (ASR): section 7 (`pipeline asr-file`, `pipeline run-once`)
- Unified A/V + ASR: section 11 (`record system/app/apps-split`) + section 7

## 4. Discovery Commands

```bash
lyricwave backends list
lyricwave providers list
lyricwave devices list
lyricwave capture apps-list
```

`capture apps-list` prints active/candidate app processes for app capture.

## 5. System Capture

### 5.1 Timed capture

```bash
lyricwave capture system --out system.wav --seconds 10
```

### 5.2 Manual stop capture

```bash
lyricwave capture system --out system.wav
```

Stop by pressing `Enter` or `Ctrl+C`.

### 5.3 Useful options

```bash
--sample-rate <HZ>
--channels <N>
--input-device <HINT>
--no-prefer-loopback
--stdout
```

## 6. App Capture (Mixed Output)

Capture one or more selected apps into one WAV output.

```bash
lyricwave capture app --out app.wav --name "Google Chrome" --seconds 10
lyricwave capture app --out apps.wav --name "Google Chrome" --name "Music" --seconds 10
lyricwave capture app --out app.wav --pid 12345 --seconds 10
```

Key options:

```bash
--out <FILE>
--seconds <N>
--pid <PID>        # repeatable
--name <TEXT>      # repeatable, case-insensitive contains match
--sample-rate <HZ>
--channels <N>
```

## 7. App Split Capture (Independent Files)

Export one WAV file per selected process.

```bash
lyricwave capture apps-split \
  --out-dir /tmp/lyricwave-split \
  --seconds 10 \
  --name "Google Chrome" \
  --name "Music"
```

Also create one optional mixed WAV from all split files:

```bash
lyricwave capture apps-split \
  --out-dir /tmp/lyricwave-split \
  --seconds 10 \
  --all-active \
  --mix-out /tmp/lyricwave-mix.wav
```

Key options:

```bash
--out-dir <DIR>
--seconds <N>
--all-active
--pid <PID>        # repeatable
--name <TEXT>      # repeatable
--mix-out <FILE>
--sample-rate <HZ>
--channels <N>
```

## 8. Pipeline Commands

### 8.1 One-shot pipeline (capture -> ASR -> translation)

```bash
lyricwave pipeline run-once \
  --seconds 8 \
  --asr-provider vibevoice \
  --vibevoice-dir /absolute/path/to/VibeVoice \
  --python-bin python \
  --target-lang zh \
  --translator-provider mock
```

### 8.2 File ASR

```bash
lyricwave pipeline asr-file \
  --audio /path/audio.wav \
  --asr-provider vibevoice \
  --vibevoice-dir /absolute/path/to/VibeVoice
```

## 9. Daemon Commands

What is this:
- `daemon` is for developers who want to build their own realtime overlay/client.
- If you just want recording + ASR, you can skip this section.

What each command does:
- `daemon run`: prints mock transcript events to current terminal (JSON lines).
- `daemon serve`: starts a TCP server that streams mock transcript events to other programs.

Typical use case:
- you write another app that reads live transcript events and renders subtitles on screen.

```bash
lyricwave daemon run --target-lang zh --interval-ms 500 --count 5
lyricwave daemon serve --host 127.0.0.1 --port 7878 --target-lang zh
```

How to understand output:
- `daemon run` output is a stream of JSON objects, one event per line.
- `daemon serve` itself prints server status; your client receives JSON events from the TCP socket.

## 10. Platform Notes

### macOS
- `capture app` / `apps-split` require Screen Recording permission.
- If you see `SCStreamErrorDomain -3801`, grant permission in:
  - `System Settings -> Privacy & Security -> Screen Recording`

### Linux
- App capture depends on PulseAudio/PipeWire tools (`pactl`, `parecord`).

### Windows
- App capture uses WASAPI process loopback.
- The selected process must be actively producing audio while recording.

## 11. Visual Commands

```bash
lyricwave visual backends
lyricwave visual displays
lyricwave visual system --out screen.mp4 --seconds 10
lyricwave visual app --out chrome.mp4 --name "Google Chrome" --seconds 10
lyricwave visual apps-list
lyricwave visual apps-split --out-dir /tmp/visual-split --seconds 10 --all-active
```

Notes:
- `visual app` / `visual apps-split` rely on native per-process visual routing and may return NotImplemented depending on OS backend state.

## 12. Unified Record Session (Audio / Visual / A+V)

Use dedicated commands for system-level and app-level composition.

```bash
# system composition
lyricwave record system --audio-out system.wav --visual-out system.mp4 --seconds 10

# app composition (single or multi selectors)
lyricwave record app --audio-out app.wav --visual-out app.mp4 --name "Google Chrome" --seconds 10
lyricwave record app --audio-out apps.wav --visual-out apps.mp4 --name "Chrome" --name "Music" --seconds 10

# split composition (one app => one audio + one visual file)
lyricwave record apps-split --out-dir /tmp/compose-split --seconds 10 --all-active

# manual stop (when command supports omitted --seconds): press Enter or Ctrl+C
lyricwave record app --audio-out app.wav --visual-out app.mp4 --name "Google Chrome"
```

Key options:

```bash
--audio-out <FILE>
--visual-out <FILE>
--seconds <N>            # optional for `record app/system`, required for apps-split
--sample-rate <HZ>       # audio
--channels <N>           # audio
--input-device <HINT>    # audio
--no-prefer-loopback     # audio
--fps <N>                # visual
--display <HINT>         # visual
--pid <PID>              # app/apps-split selector, repeatable
--name <TEXT>            # app/apps-split selector, repeatable
--all-active             # apps-split selector source
--no-audio               # apps-split only
--no-visual              # apps-split only
```
