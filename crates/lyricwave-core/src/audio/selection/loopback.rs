pub fn loopback_score(device_name: &str) -> i32 {
    let name = device_name.to_lowercase();

    let strong_keywords = [
        "blackhole",
        "stereo mix",
        "what u hear",
        "monitor",
        "vb-cable",
        "cable output",
        "soundflower",
        "loopback",
        "snd-aloop",
    ];

    let weak_keywords = ["mix", "virtual", "monitoring", "wave out", "capture"];

    let mut score = 0;

    for kw in strong_keywords {
        if name.contains(kw) {
            score += 10;
        }
    }

    for kw in weak_keywords {
        if name.contains(kw) {
            score += 3;
        }
    }

    score
}
