use crate::audio::CaptureRequest;

pub fn append_common_ffmpeg_args(args: &mut Vec<String>, request: &CaptureRequest) {
    if let Some(sample_rate) = request.sample_rate {
        args.push("-ar".to_string());
        args.push(sample_rate.to_string());
    }

    if let Some(channels) = request.channels {
        args.push("-ac".to_string());
        args.push(channels.to_string());
    }

    if let Some(duration) = request.duration_secs {
        args.push("-t".to_string());
        args.push(duration.to_string());
    }
}
