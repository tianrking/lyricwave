use std::path::{Path, PathBuf};
use std::process::Command;

pub trait AsrEngine: Send + Sync {
    fn name(&self) -> &'static str;
    fn transcribe_text(&self, input: &str) -> String;
}

pub trait AsrFileEngine: Send + Sync {
    fn name(&self) -> &'static str;
    fn transcribe_file(&self, input_audio: &Path) -> Result<String, String>;
}

pub trait Translator: Send + Sync {
    fn name(&self) -> &'static str;
    fn translate(&self, input: &str, target_lang: &str) -> String;
}

pub struct MockAsrEngine;

impl AsrEngine for MockAsrEngine {
    fn name(&self) -> &'static str {
        "mock-asr"
    }

    fn transcribe_text(&self, input: &str) -> String {
        input.trim().to_string()
    }
}

pub struct MockTranslator;

impl Translator for MockTranslator {
    fn name(&self) -> &'static str {
        "mock-translator"
    }

    fn translate(&self, input: &str, target_lang: &str) -> String {
        format!("[{target_lang}] {input}")
    }
}

pub struct VibeVoiceAsrEngine {
    pub python_bin: String,
    pub repo_dir: PathBuf,
    pub model_path: String,
}

impl AsrFileEngine for VibeVoiceAsrEngine {
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
