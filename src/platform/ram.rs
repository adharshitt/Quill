#[cfg(target_os = "linux")]
use std::fs::File;
#[cfg(target_os = "linux")]
use std::io::{BufRead, BufReader};

#[cfg(any(target_os = "macos", target_os = "windows"))]
use std::process::Command;

pub fn detect_system_ram_mb() -> Result<u64, String> {
    #[cfg(target_os = "linux")]
    {
        let file = File::open("/proc/meminfo")
            .map_err(|e| format!("Failed to open /proc/meminfo: {}", e))?;
        let reader = BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            if line.starts_with("MemTotal:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<u64>() {
                        return Ok(kb / 1024);
                    }
                }
            }
        }
        Err("MemTotal not found in /proc/meminfo".to_string())
    }

    #[cfg(target_os = "macos")]
    {
        let output = Command::new("sysctl")
            .args(&["-n", "hw.memsize"])
            .output()
            .map_err(|e| format!("Failed to execute sysctl: {}", e))?;
        if output.status.success() {
            let mem_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if let Ok(bytes) = mem_str.parse::<u64>() {
                return Ok(bytes / (1024 * 1024));
            }
        }
        Err("Failed to parse hw.memsize output from sysctl".to_string())
    }

    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell")
            .args(&["-Command", "(Get-CimInstance Win32_PhysicalMemory | Measure-Object -Property Capacity -Sum).Sum"])
            .output()
            .map_err(|e| format!("Failed to execute PowerShell RAM detection: {}", e))?;
        if output.status.success() {
            let mem_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if let Ok(bytes) = mem_str.parse::<u64>() {
                return Ok(bytes / (1024 * 1024));
            }
        }
        Err("Failed to parse PowerShell physical memory capacity".to_string())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        Err("Unsupported operating system for physical RAM detection".to_string())
    }
}
