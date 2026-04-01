use lyricwave_core::audio::audio_backends;

pub fn list() {
    println!("audio_backends:");
    for backend in audio_backends() {
        println!("- id={} note={}", backend.id, backend.note);
    }
}
