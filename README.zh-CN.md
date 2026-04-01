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
- [Architecture (English)](./docs/ARCHITECTURE.md)
- [架构说明（中文）](./docs/ARCHITECTURE.zh-CN.md)

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

## 四大核心功能

### 1）录制声音（Audio）
只要声音，不要画面时使用。支持系统整体、单应用、多应用混合、多应用独立文件。

常见场景：
- 游戏/系统声音录制
- 浏览器/音乐播放器录制
- 每个应用分别导出独立 WAV

前置配置：
- macOS 按应用录制需要给终端宿主授权“屏幕录制”
- Linux 按应用录制需要 `pactl`、`parecord`

示例：

```bash
# 系统声音
lyricwave capture system --out system.wav --seconds 10

# 单应用声音
lyricwave capture app --out chrome.wav --name "Google Chrome" --seconds 10

# 多应用混合声音
lyricwave capture app --out mixed.wav --name "Google Chrome" --name "Music" --seconds 10

# 多应用独立声音文件（可选再输出一个混合文件）
lyricwave capture apps-split \
  --out-dir /tmp/audio-split \
  --seconds 10 \
  --all-active \
  --mix-out /tmp/audio-mix.wav
```

### 2）录制画面（Visual）
只要画面流，不要声音时使用。支持系统画面、指定应用画面、多应用独立画面文件。

常见场景：
- 纯录屏
- 指定应用画面采集
- 每个应用画面单独输出

前置配置：
- 按应用画面路由依赖平台原生能力，部分平台可能返回 `NotImplemented`

示例：

```bash
# 系统画面
lyricwave visual system --out system.mp4 --seconds 10

# 单应用画面
lyricwave visual app --out chrome.mp4 --name "Google Chrome" --seconds 10

# 查看可选画面应用
lyricwave visual apps-list

# 多应用独立画面文件
lyricwave visual apps-split --out-dir /tmp/visual-split --seconds 10 --all-active
```

### 3）声音转文字（ASR）
把录音文件转成文本，可选再翻译。

常见场景：
- 录音生成转写文本
- 字幕/会议记录生成
- ASR 后再接翻译

前置配置：
- 准备本地 VibeVoice 仓库目录
- 准备 Python 运行环境
- 配置 provider 参数（`--asr-provider`、`--vibevoice-dir`、`--model-path`）

示例：

```bash
# 已有音频文件 -> ASR
lyricwave pipeline asr-file \
  --audio /path/audio.wav \
  --asr-provider vibevoice \
  --vibevoice-dir /absolute/path/to/VibeVoice

# 录制系统声音 -> ASR -> 翻译（一条命令）
lyricwave pipeline run-once \
  --seconds 8 \
  --asr-provider vibevoice \
  --vibevoice-dir /absolute/path/to/VibeVoice \
  --python-bin python \
  --target-lang zh \
  --translator-provider mock
```

### 4）大一统：音画一起（Video Workflow）
需要声音+画面一起时使用。支持系统级、指定应用级、按应用拆分输出；并可继续接 ASR。

常见场景：
- 系统级音画录制
- 指定应用音画录制
- 每个应用分别输出音频+画面
- 对生成音频继续做 ASR

示例：

```bash
# 系统级音画
lyricwave record system --audio-out system.wav --visual-out system.mp4 --seconds 10

# 指定应用音画
lyricwave record app --audio-out app.wav --visual-out app.mp4 --name "Google Chrome" --seconds 10

# 按应用拆分（每个应用输出一份 audio + 一份 visual）
lyricwave record apps-split --out-dir /tmp/compose-split --seconds 10 --all-active

# 对拆分出来的音频继续做 ASR
lyricwave pipeline asr-file \
  --audio /tmp/compose-split/Google_Chrome-12345.wav \
  --asr-provider vibevoice \
  --vibevoice-dir /absolute/path/to/VibeVoice
```

## CI 与发版

- CI 工作流：`.github/workflows/ci.yml`
- Release 工作流：`.github/workflows/release.yml`
- 打标签触发发版：`v*`（例如 `v0.1.0`）

## 架构摘要

- `crates/lyricwave-core`
  - `audio`：音频采集域
  - `visual`：画面帧采集域（显示器/窗口流）
  - `composition`：A/V 顶层会话编排（组合 audio/visual）
  - `pipeline`：ASR/翻译处理域（采集后的处理流）
  - `service`：pipeline 服务辅助层
- `crates/lyricwave-cli`
  - `cli.rs`：命令模型定义
  - `commands/*`：各命令处理器
  - `main.rs`：命令分发


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
