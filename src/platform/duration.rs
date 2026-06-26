use std::process::Command;
use std::time::{Duration, Instant};
use std::thread;

pub fn get_video_duration_safe(file_path: &str, timeout: Duration) -> Result<f64, String> {
    let mut child = Command::new("ffprobe")
        .args(&[
            "-v", "error",
            "-show_entries", "format=duration",
            "-of", "default=noprint_wrappers=1:nokey=1",
            file_path
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn ffprobe: {}", e))?;

    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if status.success() {
                    let mut stdout = child.stdout.take().ok_or("Failed to read stdout")?;
                    let mut output_str = String::new();
                    use std::io::Read;
                    stdout.read_to_string(&mut output_str)
                        .map_err(|e| format!("Failed to read ffprobe output: {}", e))?;
                    
                    let dur_str = output_str.trim();
                    if let Ok(dur) = dur_str.parse::<f64>() {
                        if dur.is_finite() && dur > 0.0 {
                            return Ok(dur);
                        } else {
                            return Err(format!("Invalid video duration value parsed: {}", dur_str));
                        }
                    }
                    return Err(format!("Failed to parse duration string: '{}'", dur_str));
                } else {
                    let mut stderr = child.stderr.take().ok_or("Failed to read stderr")?;
                    let mut err_str = String::new();
                    use std::io::Read;
                    stderr.read_to_string(&mut err_str).ok();
                    return Err(format!("ffprobe exited with error: {}", err_str.trim()));
                }
            }
            Ok(None) => {
                if start.elapsed() >= timeout {
                    child.kill().ok();
                    return Err(format!("ffprobe execution timed out after {:?}", timeout));
                }
                thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                child.kill().ok();
                return Err(format!("Error waiting for ffprobe: {}", e));
            }
        }
    }
}
