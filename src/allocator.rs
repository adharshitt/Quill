use std::fs::File;
use std::io::{BufRead, BufReader};
use std::process::Command;
use std::thread;

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

fn get_system_ram_mb() -> u64 {
    if let Ok(file) = File::open("/proc/meminfo") {
        let reader = BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            if line.starts_with("MemTotal:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<u64>() {
                        return kb / 1024;
                    }
                }
            }
        }
    }
    4096
}

fn get_video_duration(file_path: &str) -> f64 {
    let output = Command::new("ffprobe")
        .args(&[
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
            file_path
        ])
        .output();

    if let Ok(out) = output {
        if out.status.success() {
            let dur_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if let Ok(dur) = dur_str.parse::<f64>() {
                return dur;
            }
        }
    }
    50.0
}

pub fn allocate_transcode_task(
    input_file_path: &str,
    input_file_size: u64,
    _spatial_energy: f64,
    _temporal_energy: f64,
    seek: Option<f64>,
    duration: Option<f64>,
) -> AllocationTask {
    let system_ram_mb = get_system_ram_mb();
    let num_cpus = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    
    let actual_duration = get_video_duration(input_file_path);
    let video_duration = duration.unwrap_or(actual_duration);

    let proportion = if actual_duration > 0.0 {
        video_duration / actual_duration
    } else {
        1.0
    };
    let target_input_size = input_file_size as f64 * proportion;

    let target_bitrate_bps = if video_duration > 0.0 {
        ((target_input_size * 0.075 * 8.0) / video_duration) as u32
    } else {
        600_000
    };
    
    let target_bitrate_kbps = (target_bitrate_bps / 1000).clamp(350, 4000);

    let preset = if num_cpus >= 12 && system_ram_mb >= 6000 {
        "10".to_string()
    } else {
        "12".to_string()
    };

    let threads = if system_ram_mb < 2000 {
        "2".to_string()
    } else if system_ram_mb < 4000 {
        "4".to_string()
    } else {
        "6".to_string()
    };

    let pix_fmt = "yuv420p10le".to_string();
    let crf_or_bitrate = format!("-b:v {}k", target_bitrate_kbps);

    AllocationTask {
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
    }
}
