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
```

Current default is `cpal-native`.

## 3. Discovery Commands

```bash
lyricwave backends list
lyricwave providers list
lyricwave devices list
lyricwave capture apps-list
```

`capture apps-list` prints active/candidate app processes for app capture.

## 4. System Capture

### 4.1 Timed capture

```bash
lyricwave capture system --out system.wav --seconds 10
```

### 4.2 Manual stop capture

```bash
lyricwave capture system --out system.wav
```

Stop by pressing `Enter` or `Ctrl+C`.

### 4.3 Useful options

```bash
--sample-rate <HZ>
--channels <N>
--input-device <HINT>
--no-prefer-loopback
--stdout
```

## 5. App Capture (Mixed Output)

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

## 6. App Split Capture (Independent Files)

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

## 7. Pipeline Commands

### 7.1 One-shot pipeline (capture -> ASR -> translation)

```bash
lyricwave pipeline run-once \
  --seconds 8 \
  --asr-provider vibevoice \
  --vibevoice-dir /absolute/path/to/VibeVoice \
  --python-bin python \
  --target-lang zh \
  --translator-provider mock
```

### 7.2 File ASR

```bash
lyricwave pipeline asr-file \
  --audio /path/audio.wav \
  --asr-provider vibevoice \
  --vibevoice-dir /absolute/path/to/VibeVoice
```

## 8. Daemon Commands

```bash
lyricwave daemon run --target-lang zh --interval-ms 500 --count 5
lyricwave daemon serve --host 127.0.0.1 --port 7878 --target-lang zh
```

## 9. Platform Notes

### macOS
- `capture app` / `apps-split` require Screen Recording permission.
- If you see `SCStreamErrorDomain -3801`, grant permission in:
  - `System Settings -> Privacy & Security -> Screen Recording`

### Linux
- App capture depends on PulseAudio/PipeWire tools (`pactl`, `parecord`).

### Windows
- App capture uses WASAPI process loopback.
- The selected process must be actively producing audio while recording.

## 10. Video Commands (Architecture Scaffold)

```bash
lyricwave video backends
lyricwave video displays
lyricwave video capture-screen --out screen.mp4 --seconds 10
```

Notes:
- This is the unified video architecture entrypoint.
- Native per-OS screen recorder implementations are wired by backend platform modules and will be filled incrementally.
