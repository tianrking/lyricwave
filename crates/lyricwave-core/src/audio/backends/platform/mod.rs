#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
mod unsupported;
#[cfg(target_os = "windows")]
mod windows;

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
