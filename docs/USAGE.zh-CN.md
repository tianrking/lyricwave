# lyricwave CLI 使用手册

- English version: [USAGE.md](./USAGE.md)

## 1. 安装方式

方式 A（推荐）：从 [Releases](https://github.com/tianrking/lyricwave/releases) 下载对应平台预编译包。

方式 B：源码构建。

```bash
cargo build --workspace --release
```

## 2. 全局参数

```bash
--audio-backend <ID>
```

当前默认后端为 `cpal-native`。

## 3. 信息探测命令

```bash
lyricwave backends list
lyricwave providers list
lyricwave devices list
lyricwave capture apps-list
```

`capture apps-list` 用于列出当前活跃/可候选的应用进程。

## 4. 系统混音录制

### 4.1 定时录制

```bash
lyricwave capture system --out system.wav --seconds 10
```

### 4.2 手动停止录制

```bash
lyricwave capture system --out system.wav
```

按 `Enter` 或 `Ctrl+C` 停止。

### 4.3 常用参数

```bash
--sample-rate <HZ>
--channels <N>
--input-device <HINT>
--no-prefer-loopback
--stdout
```

## 5. 按应用录制（混合输出）

将一个或多个应用录到同一个 WAV 文件。

```bash
lyricwave capture app --out app.wav --name "Google Chrome" --seconds 10
lyricwave capture app --out apps.wav --name "Google Chrome" --name "Music" --seconds 10
lyricwave capture app --out app.wav --pid 12345 --seconds 10
```

关键参数：

```bash
--out <FILE>
--seconds <N>
--pid <PID>        # 可重复
--name <TEXT>      # 可重复，大小写不敏感的包含匹配
--sample-rate <HZ>
--channels <N>
```

## 6. 多应用拆分录制（独立文件）

把每个目标应用分别导出为独立 WAV 文件。

```bash
lyricwave capture apps-split \
  --out-dir /tmp/lyricwave-split \
  --seconds 10 \
  --name "Google Chrome" \
  --name "Music"
```

可选再输出一个混合文件：

```bash
lyricwave capture apps-split \
  --out-dir /tmp/lyricwave-split \
  --seconds 10 \
  --all-active \
  --mix-out /tmp/lyricwave-mix.wav
```

关键参数：

```bash
--out-dir <DIR>
--seconds <N>
--all-active
--pid <PID>        # 可重复
--name <TEXT>      # 可重复
--mix-out <FILE>
--sample-rate <HZ>
--channels <N>
```

## 7. Pipeline 命令

### 7.1 一次性流水线（录音 -> ASR -> 翻译）

```bash
lyricwave pipeline run-once \
  --seconds 8 \
  --asr-provider vibevoice \
  --vibevoice-dir /absolute/path/to/VibeVoice \
  --python-bin python \
  --target-lang zh \
  --translator-provider mock
```

### 7.2 文件 ASR

```bash
lyricwave pipeline asr-file \
  --audio /path/audio.wav \
  --asr-provider vibevoice \
  --vibevoice-dir /absolute/path/to/VibeVoice
```

## 8. Daemon 命令

```bash
lyricwave daemon run --target-lang zh --interval-ms 500 --count 5
lyricwave daemon serve --host 127.0.0.1 --port 7878 --target-lang zh
```

## 9. 平台说明

### macOS
- `capture app` / `apps-split` 需要“屏幕录制”权限。
- 若出现 `SCStreamErrorDomain -3801`：
  - 打开 `系统设置 -> 隐私与安全性 -> 屏幕录制` 授权。

### Linux
- 按应用录制依赖 PulseAudio/PipeWire 工具（`pactl`、`parecord`）。

### Windows
- 按应用录制使用 WASAPI process loopback。
- 目标进程必须在录制期间真实发声。

## 10. Video 命令（架构骨架）

```bash
lyricwave video backends
lyricwave video displays
lyricwave video capture-screen --out screen.mp4 --seconds 10
```

说明：
- 这是统一的 video 架构入口。
- 各平台原生录屏实现已经有后端路由，后续会逐步填充。
