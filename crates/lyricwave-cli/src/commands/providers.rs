use lyricwave_core::audio::audio_backends;
use lyricwave_core::pipeline::{asr_file_providers, asr_text_providers, translator_providers};

pub fn list() {
    let audio_backend_ids: Vec<&str> = audio_backends().iter().map(|b| b.id).collect();
    println!("audio_backends: {}", audio_backend_ids.join(", "));

    println!("text_asr:");
    for p in asr_text_providers() {
        println!(
            "- id={} capability={} mode={:?} setup_required={} note={}",
            p.id, p.capability, p.mode, p.requires_setup, p.note
        );
    }

    println!("file_asr:");
    for p in asr_file_providers() {
        println!(
            "- id={} capability={} mode={:?} setup_required={} note={}",
            p.id, p.capability, p.mode, p.requires_setup, p.note
        );
    }

    println!("translator:");
    for p in translator_providers() {
        println!(
            "- id={} capability={} mode={:?} setup_required={} note={}",
            p.id, p.capability, p.mode, p.requires_setup, p.note
        );
    }
}
