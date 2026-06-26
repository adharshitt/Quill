use std::thread;
use std::time::Duration;
use crate::platform;

pub struct AllocationTask {
    pub preset: String,
    pub crf_or_bitrate: String,
    pub threads: String,
    pub pix_fmt: String,
    pub target_bitrate_kbps: u32,
    pub num_cpus: usize,
    pub system_ram_mb: u64,
    pub video_duration: f64,
    pub original_duration: f64,
    pub seek: Option<f64>,
    pub duration: Option<f64>,
}

pub fn allocate_transcode_task(
    input_file_path: &str,
    input_file_size: u64,
    _spatial_energy: f64,
    _temporal_energy: f64,
    seek: Option<f64>,
    duration: Option<f64>,
) -> Result<AllocationTask, String> {
    let system_ram_mb = platform::ram::detect_system_ram_mb()
        .map_err(|e| format!("Resource Allocation Failed (System RAM detection error): {}", e))?;

    let num_cpus = thread::available_parallelism()
        .map(|n| n.get())
        .map_err(|e| format!("Resource Allocation Failed (CPU count detection error): {}", e))?;
    
    let actual_duration = platform::duration::get_video_duration_safe(input_file_path, Duration::from_secs(5))
        .map_err(|e| format!("Resource Allocation Failed (FFprobe execution error): {}", e))?;

    let video_duration = duration.unwrap_or(actual_duration);

    if !video_duration.is_finite() || video_duration <= 0.0 {
        return Err("Resource Allocation Failed (Video duration is zero or non-finite)".to_string());
    }

    let proportion = if actual_duration > 0.0 {
        video_duration / actual_duration
    } else {
        1.0
    };
    
    let target_input_size = platform::math::calculate_proportional_size(input_file_size, proportion)?;
    let target_bitrate_bps = platform::math::calculate_target_bitrate(target_input_size, video_duration)?;
    let target_bitrate_kbps = (target_bitrate_bps / 1000).clamp(350, 4000);

    let preset = if num_cpus >= 12 && system_ram_mb >= 6000 {
        "10".to_string()
    } else {
        "12".to_string()
    };

    let ram_thread_cap = system_ram_mb / 800; // Cap of ~800MB per thread for SVT-AV1 OOM safety
    let threads_count = num_cpus.min(ram_thread_cap as usize).max(1);
    let threads = threads_count.to_string();

    let pix_fmt = "yuv420p10le".to_string();
    let crf_or_bitrate = format!("-b:v {}k", target_bitrate_kbps);

    Ok(AllocationTask {
        preset,
        crf_or_bitrate,
        threads,
        pix_fmt,
        target_bitrate_kbps,
        num_cpus,
        system_ram_mb,
        video_duration,
        original_duration: actual_duration,
        seek,
        duration,
    })
}
