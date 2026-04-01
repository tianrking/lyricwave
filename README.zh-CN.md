# lyricwave

<p align="center">
  <img src="https://img.shields.io/badge/Rust-2024%20Edition-black?logo=rust" alt="Rust" />
  <img src="https://img.shields.io/badge/CLI-跨平台-1f6feb" alt="CLI" />
  <img src="https://img.shields.io/github/actions/workflow/status/tianrking/lyricwave/ci.yml?branch=main&label=CI" alt="CI" />
  <img src="https://img.shields.io/github/actions/workflow/status/tianrking/lyricwave/release.yml?branch=main&label=Release" alt="Release" />
  <img src="https://img.shields.io/github/v/release/tianrking/lyricwave" alt="Latest Release" />
  <img src="https://img.shields.io/github/license/tianrking/lyricwave" alt="License" />
</p>

一个跨平台音频采集 CLI，支持：
- 系统混音录制
- 按应用录制（单个/多个）
- 多应用拆分导出（每个应用独立 WAV）
- ASR 与翻译流水线集成

语言版本：
- [English README](./README.md)
- 中文（当前）

## 下载即用（推荐）

从 [Releases](https://github.com/tianrking/lyricwave/releases) 下载对应平台包：

- Linux：`x86_64/i686/aarch64/armv7` 的 `.tar.gz`，`x86_64` 另有 `.deb`
- macOS：Intel / Apple Silicon 的 `.tar.gz`
- Windows：`x86_64/i686` 的 `.zip`（内含 `lyricwave.exe`）

下载后直接运行：

```bash
lyricwave --help
```

详细 CLI 手册：
- [Usage Guide (English)](./docs/USAGE.md)
- [使用手册（中文）](./docs/USAGE.zh-CN.md)

## 从源码构建

```bash
cargo build --workspace --release
```

## 平台支持

| 能力 | macOS | Linux | Windows |
|---|---|---|---|
| 系统混音录制（`capture system`） | 支持（通常需要回环输入设备） | 支持（可能需要 monitor/loopback 输入） | 支持 |
| 按应用录制（`capture app`） | 支持（ScreenCaptureKit，需要屏幕录制权限） | 支持（PulseAudio/PipeWire） | 支持（WASAPI process loopback） |
| 当前活跃应用发现（`capture apps-list`） | 支持（基于 ScreenCaptureKit 可采集应用候选） | 支持（基于 sink-input 活跃进程） | 支持（基于 WASAPI session 活跃进程） |
| 多应用拆分导出（`capture apps-split`） | 支持 | 支持 | 支持 |

## 快速示例

```bash
# 查看后端/服务/设备
lyricwave backends list
lyricwave providers list
lyricwave devices list

# 查看当前活跃/候选音频应用
lyricwave capture apps-list

# 查看 video 后端与显示器
lyricwave video backends
lyricwave video displays

# video 录屏命令骨架（原生实现正在完善中）
lyricwave video capture-screen --out screen.mp4 --seconds 10

# 录制系统混音（10秒）
lyricwave capture system --out system.wav --seconds 10

# 按应用录制（单应用）
lyricwave capture app --out chrome.wav --name "Google Chrome" --seconds 10

# 按应用录制（多应用混合到一个文件）
lyricwave capture app --out apps.wav --name "Google Chrome" --name "Music" --seconds 10

# 多应用拆分：每个应用一个 wav，可选额外输出混合 wav
lyricwave capture apps-split \
  --out-dir /tmp/lyricwave-split \
  --seconds 10 \
  --name "Google Chrome" \
  --name "Music" \
  --mix-out /tmp/lyricwave-mix.wav
```

## CI 与发版

- CI 工作流：`.github/workflows/ci.yml`
- Release 工作流：`.github/workflows/release.yml`
- 打标签触发发版：`v*`（例如 `v0.1.0`）

## 常见问题

### macOS `capture app` 报 `SCStreamErrorDomain -3801`
这是系统权限（TCC）问题，请到：
- `系统设置 -> 隐私与安全性 -> 屏幕录制`
给 Terminal/宿主应用授权。

### Linux `capture app` 失败
检查：
- `pactl` / `parecord` 是否已安装
- 目标应用是否当前正在发声（有活跃 sink-input）

### Windows `capture app` 输出空白
检查：
- 目标进程是否在录制期间持续发声
- 目标是否被静音或被企业安全策略限制
