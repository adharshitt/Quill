use std::process::Command;
use std::time::Instant;
use crate::allocator::AllocationTask;

pub fn run_ffmpeg_transcode(
    input_path: &str,
    output_path: &str,
    task: &AllocationTask,
) -> std::io::Result<f64> {
    let svt_params = format!(
        "keyint=10s:tune=0:enable-overlays=1:lp={}",
        task.threads
    );
    
    let bitrate_parts: Vec<&str> = task.crf_or_bitrate.split_whitespace().collect();
    
    let mut ffmpeg_args = Vec::new();
    
    ffmpeg_args.push("-y");
    
    let seek_str = if let Some(s) = task.seek {
        Some(s.to_string())
    } else {
        None
    };
    if let Some(ref s) = seek_str {
        ffmpeg_args.push("-ss");
        ffmpeg_args.push(s);
    }
    
    ffmpeg_args.push("-i");
    ffmpeg_args.push(input_path);
    
    let duration_str = if let Some(d) = task.duration {
        Some(d.to_string())
    } else {
        None
    };
    if let Some(ref d) = duration_str {
        ffmpeg_args.push("-t");
        ffmpeg_args.push(d);
    }
    
    ffmpeg_args.push("-c:v");
    ffmpeg_args.push("libsvtav1");
    
    for part in &bitrate_parts {
        ffmpeg_args.push(part);
    }
    
    ffmpeg_args.push("-preset");
    ffmpeg_args.push(&task.preset);
    
    ffmpeg_args.push("-svtav1-params");
    ffmpeg_args.push(&svt_params);
    
    ffmpeg_args.push("-pix_fmt");
    ffmpeg_args.push(&task.pix_fmt);
    
    ffmpeg_args.push("-c:a");
    ffmpeg_args.push("libopus");
    
    ffmpeg_args.push(output_path);

    println!("Running Command: ffmpeg {}", ffmpeg_args.join(" "));
    let start_encode = Instant::now();
    
    let status = Command::new("ffmpeg")
        .args(&ffmpeg_args)
        .status()?;

    if status.success() {
        let elapsed = start_encode.elapsed().as_secs_f64();
        Ok(elapsed)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("FFmpeg process failed with exit code: {}", status.code().unwrap_or(-1)),
        ))
    }
}
