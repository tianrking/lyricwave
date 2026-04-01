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
--visual-backend <ID>
```

当前默认后端为 `cpal-native`。

## 3. 四大功能导航

- 录制声音：第 5-7 节（`capture system/app/apps-split`）
- 录制画面：第 11 节（`visual system/app/apps-list/apps-split`）
- 声音转文字（ASR）：第 8 节（`pipeline asr-file`、`pipeline run-once`）
- 大一统音画+ASR：第 12 节（`record system/app/apps-split`）+ 第 8 节

## 4. 信息探测命令

```bash
lyricwave backends list
lyricwave providers list
lyricwave devices list
lyricwave capture apps-list
```

`capture apps-list` 用于列出当前活跃/可候选的应用进程。

## 5. 系统混音录制

### 5.1 定时录制

```bash
lyricwave capture system --out system.wav --seconds 10
```

### 5.2 手动停止录制

```bash
lyricwave capture system --out system.wav
```

按 `Enter` 或 `Ctrl+C` 停止。

### 5.3 常用参数

```bash
--sample-rate <HZ>
--channels <N>
--input-device <HINT>
--no-prefer-loopback
--stdout
```

## 6. 按应用录制（混合输出）

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

## 7. 多应用拆分录制（独立文件）

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

## 8. Pipeline 命令

### 8.1 一次性流水线（录音 -> ASR -> 翻译）

```bash
lyricwave pipeline run-once \
  --seconds 8 \
  --asr-provider vibevoice \
  --vibevoice-dir /absolute/path/to/VibeVoice \
  --python-bin python \
  --target-lang zh \
  --translator-provider mock
```

### 8.2 文件 ASR

```bash
lyricwave pipeline asr-file \
  --audio /path/audio.wav \
  --asr-provider vibevoice \
  --vibevoice-dir /absolute/path/to/VibeVoice
```

## 9. Daemon 命令

这是干嘛的：
- `daemon` 是给开发者做“实时字幕浮窗/外部客户端”对接的。
- 如果你只想录音、录屏、ASR，直接跳过这一节即可。

两个命令的区别：
- `daemon run`：在当前终端里持续打印“模拟转写事件”（JSON 每行一条）。
- `daemon serve`：启动一个 TCP 服务，把这些事件推送给别的程序。

常见场景：
- 你自己写一个客户端，实时接收事件并渲染字幕/翻译。

```bash
lyricwave daemon run --target-lang zh --interval-ms 500 --count 5
lyricwave daemon serve --host 127.0.0.1 --port 7878 --target-lang zh
```

怎么看输出：
- `daemon run`：终端里会看到一行一行 JSON 事件。
- `daemon serve`：终端主要显示服务状态；事件会发给连接该端口的客户端。

## 10. 平台说明

### macOS
- `capture app` / `apps-split` 需要“屏幕录制”权限。
- 若出现 `SCStreamErrorDomain -3801`：
  - 打开 `系统设置 -> 隐私与安全性 -> 屏幕录制` 授权。

### Linux
- 按应用录制依赖 PulseAudio/PipeWire 工具（`pactl`、`parecord`）。

### Windows
- 按应用录制使用 WASAPI process loopback。
- 目标进程必须在录制期间真实发声。

## 11. Visual 命令

```bash
lyricwave visual backends
lyricwave visual displays
lyricwave visual system --out screen.mp4 --seconds 10
lyricwave visual app --out chrome.mp4 --name "Google Chrome" --seconds 10
lyricwave visual apps-list
lyricwave visual apps-split --out-dir /tmp/visual-split --seconds 10 --all-active
```

说明：
- `visual app` / `visual apps-split` 依赖各平台原生“按进程画面路由”能力，部分平台当前可能返回 NotImplemented。

## 12. 统一录制会话（音频 / 画面 / 音画联合）

通过分层命令执行系统级与应用级音画联合录制。

```bash
# 系统级音画合成
lyricwave record system --audio-out system.wav --visual-out system.mp4 --seconds 10

# 应用级音画合成（可单选也可多选）
lyricwave record app --audio-out app.wav --visual-out app.mp4 --name "Google Chrome" --seconds 10
lyricwave record app --audio-out apps.wav --visual-out apps.mp4 --name "Chrome" --name "Music" --seconds 10

# 应用级拆分（每个应用输出一份音频+一份画面）
lyricwave record apps-split --out-dir /tmp/compose-split --seconds 10 --all-active

# 手动停止（不传 --seconds）：按 Enter 或 Ctrl+C
lyricwave record app --audio-out app.wav --visual-out app.mp4 --name "Google Chrome"
```

关键参数：

```bash
--audio-out <FILE>
--visual-out <FILE>
--seconds <N>            # `record app/system` 可选；apps-split 必填
--sample-rate <HZ>       # 音频
--channels <N>           # 音频
--input-device <HINT>    # 音频
--no-prefer-loopback     # 音频
--fps <N>                # 画面
--display <HINT>         # 画面
--pid <PID>              # app/apps-split 选择器，可重复
--name <TEXT>            # app/apps-split 选择器，可重复
--all-active             # apps-split 从活跃进程选择
--no-audio               # apps-split 专用
--no-visual              # apps-split 专用
```
