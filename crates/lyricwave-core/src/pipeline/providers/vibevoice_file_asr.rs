use std::path::{Path, PathBuf};
use std::process::Command;

use crate::pipeline::AsrFileEngine;

pub struct VibeVoiceFileAsrProvider {
    pub python_bin: String,
    pub repo_dir: PathBuf,
    pub model_path: String,
}

impl AsrFileEngine for VibeVoiceFileAsrProvider {
    fn name(&self) -> &'static str {
        "vibevoice-asr"
    }

    fn transcribe_file(&self, input_audio: &Path) -> Result<String, String> {
        let output = Command::new(&self.python_bin)
            .current_dir(&self.repo_dir)
            .arg("demo/vibevoice_asr_inference_from_file.py")
            .arg("--model_path")
            .arg(&self.model_path)
            .arg("--audio_files")
            .arg(input_audio)
            .output()
            .map_err(|err| format!("failed to spawn VibeVoice ASR command: {err}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(if stderr.is_empty() {
                format!("VibeVoice ASR exited with status {}", output.status)
            } else {
                stderr
            });
        }

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if stdout.is_empty() {
            return Err("VibeVoice ASR returned empty output".to_string());
        }

        Ok(stdout)
    }
}
