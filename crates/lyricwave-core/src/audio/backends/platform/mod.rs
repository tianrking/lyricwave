#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
mod unsupported;
#[cfg(target_os = "windows")]
mod windows;

mod common;

use crate::audio::CaptureRequest;

#[cfg(target_os = "linux")]
pub fn ffmpeg_input_args(request: &CaptureRequest) -> Vec<String> {
    linux::ffmpeg_input_args(request)
}
#[cfg(target_os = "macos")]
pub fn ffmpeg_input_args(request: &CaptureRequest) -> Vec<String> {
    macos::ffmpeg_input_args(request)
}
#[cfg(target_os = "windows")]
pub fn ffmpeg_input_args(request: &CaptureRequest) -> Vec<String> {
    windows::ffmpeg_input_args(request)
}
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn ffmpeg_input_args(request: &CaptureRequest) -> Vec<String> {
    unsupported::ffmpeg_input_args(request)
}

#[cfg(target_os = "linux")]
pub fn capability_note() -> &'static str {
    linux::capability_note()
}
#[cfg(target_os = "macos")]
pub fn capability_note() -> &'static str {
    macos::capability_note()
}
#[cfg(target_os = "windows")]
pub fn capability_note() -> &'static str {
    windows::capability_note()
}
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn capability_note() -> &'static str {
    unsupported::capability_note()
}

pub fn append_common_ffmpeg_args(args: &mut Vec<String>, request: &CaptureRequest) {
    common::append_common_ffmpeg_args(args, request);
}
