# lyricwave 架构说明

## 设计目标

把音频采集和画面采集保持为并列领域，在更高层再做组合。

这样可以同时支持：
- 仅音频工作流
- 仅画面工作流
- 音频+画面联合工作流
- 未来处理能力与 provider 的平滑扩展

## 核心模块布局

`crates/lyricwave-core/src`

- `audio/`
  - 仅负责音频领域。
  - 包含音频后端 trait、各平台实现、设备选择策略、系统/应用采集。
- `visual/`
  - 仅负责画面领域。
  - 包含 visual 后端 trait、各平台实现、显示器/屏幕采集。
- `composition/`
  - A/V 编排层。
  - 一次会话里接收可选的音频请求和可选的画面请求。
  - 支持音频-only、画面-only、音频+画面并行三种模式。
- `pipeline/`
  - 文本处理领域（ASR/翻译 provider、事件、注册中心）。
  - 不应依赖平台采集的内部细节。
- `service.rs`
  - 为 transcript/translation 流程提供 pipeline 编排辅助。

## 依赖方向

`lyricwave-core` 内建议依赖方向：

`audio` 与 `visual`（采集叶子域） -> `composition`（组合层）  
`pipeline`（处理域）与采集后端解耦，保持独立

CLI 层负责组装：
- `capture ...` 命令使用 `audio`
- `visual ...` 命令使用 `visual`
- `record run ...` 通过 `composition` 执行统一会话
- `pipeline ...` 命令使用 `pipeline`/`service`

## 为什么 `pipeline` 独立

`pipeline` 不是采集后端的一部分，而是采集后的处理层（ASR、翻译、事件流）。  
因此它应与 `audio`、`visual` 并列但职责不同，保持独立最合理。

## 后续扩展路径

该布局天然支持：
- 各平台原生视频采集能力持续增强
- `composition` 增加 A/V 同步元数据
- 可选复用/封装导出层（mux/export）
- 可选实时浮窗字幕模块（识别文本 + 翻译文本）
