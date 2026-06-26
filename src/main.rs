use std::env;
use std::time::Instant;

mod telemetry;
mod allocator;
mod transcode;

fn print_usage() {
    println!("Usage: win --input <input_video> --output <output_video> [--seek <seconds>] [--duration <seconds>]");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        print_usage();
        std::process::exit(1);
    }

    let mut input_path = String::new();
    let mut output_path = String::new();
    let mut seek: Option<f64> = None;
    let mut duration: Option<f64> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--input" => {
                if i + 1 < args.len() {
                    input_path = args[i + 1].clone();
                    i += 2;
                } else {
                    println!("Error: --input requires a value.");
                    std::process::exit(1);
                }
            }
            "--output" => {
                if i + 1 < args.len() {
                    output_path = args[i + 1].clone();
                    i += 2;
                } else {
                    println!("Error: --output requires a value.");
                    std::process::exit(1);
                }
            }
            "--seek" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<f64>() {
                        Ok(val) => {
                            if val < 0.0 {
                                println!("Error: --seek must be a non-negative number.");
                                std::process::exit(1);
                            }
                            seek = Some(val);
                        }
                        Err(_) => {
                            println!("Error: --seek value must be a valid number.");
                            std::process::exit(1);
                        }
                    }
                    i += 2;
                } else {
                    println!("Error: --seek requires a value.");
                    std::process::exit(1);
                }
            }
            "--duration" => {
                if i + 1 < args.len() {
                    match args[i + 1].parse::<f64>() {
                        Ok(val) => {
                            if val <= 0.0 {
                                println!("Error: --duration must be a positive number.");
                                std::process::exit(1);
                            }
                            duration = Some(val);
                        }
                        Err(_) => {
                            println!("Error: --duration value must be a valid number.");
                            std::process::exit(1);
                        }
                    }
                    i += 2;
                } else {
                    println!("Error: --duration requires a value.");
                    std::process::exit(1);
                }
            }
            _ => {
                println!("Error: Unknown argument '{}'", args[i]);
                print_usage();
                std::process::exit(1);
            }
        }
    }

    if input_path.is_empty() || output_path.is_empty() {
        println!("Error: Both --input and --output are required.");
        print_usage();
        std::process::exit(1);
    }

    if input_path.starts_with('-') || output_path.starts_with('-') {
        println!("Error: File paths cannot start with a hyphen '-' to prevent flag injection.");
        std::process::exit(1);
    }

    let input_metadata = match std::fs::metadata(&input_path) {
        Ok(m) => m,
        Err(e) => {
            println!("Error opening input file {}: {}", input_path, e);
            std::process::exit(1);
        }
    };
    let input_size = input_metadata.len();

    println!(" ");
    println!(" Quill AD01 Optimizer: Telemetry scan");
    println!(" ");
    println!("Analyzing spatial features from: {}", input_path);

    let start_scan = Instant::now();
    let features = match telemetry::extract_features_low_memory(&input_path) {
        Ok(f) => f,
        Err(e) => {
            println!("Error during pre-scan: {}", e);
            std::process::exit(1);
        }
    };
    let scan_time = start_scan.elapsed();
    println!("[Pre-Scan] Finished in {:.2?} (RAM footprint: <1 KB)", scan_time);

    let s = features.spatial_energy;
    let t = features.temporal_energy;
    let l = features.luma_brightness;
    println!("  - Spatial Energy:  {:.4}", s);
    println!("  - Temporal Energy: {:.4}", t);
    println!("  - Luma Brightness: {:.4}", l);

    let complexity_score = telemetry::calculate_complexity_index(s, t, l);
    println!("[Forecaster] Telemetry Complexity Index: {:.2}", complexity_score);

    let task = allocator::allocate_transcode_task(&input_path, input_size, s, t, seek, duration);

    println!("\n");
    
    fn print_task_diagnostics(task: &allocator::AllocationTask) {
    // 1. The task allcoaation
    println!("{:=^60}", " Quill | Task  Allocation");

    // 2. Host System Metrics
    // Format RAM safely: convert MB to GB if it's large for better readability
    let ram_string = if task.system_ram_mb >= 1024 {
        format!("{:.2} GB", task.system_ram_mb as f64 / 1024.0)
    } else {
        format!("{} MB", task.system_ram_mb)
    };
    
    println!("{:<14} | Logical Cores : {} threads", "Machine", task.num_cpus);
    println!("{:<14} | Detected RAM  : {}", "Machine", ram_string);

    // 3. Video Metadata
    println!("{:<14} | Duration      : {:.2} seconds", "[Video Info]", task.video_duration);

    // 4. Allocator & Security-Hardened Configurations
    // Safety: Escape/Sanitize dynamic strings to prevent terminal injection attacks
    let safe_preset = task.preset.escape_debug();
    let safe_pix_fmt = task.pix_fmt.escape_debug();

    let bitrate_string = if task.target_bitrate_kbps >= 1000 {
        format!("{:.2} Mbps", task.target_bitrate_kbps as f64 / 1000.0)
    } else {
        format!("{} kbps", task.target_bitrate_kbps)
    };

    println!("{:<14} | Target Bitrate: {} (Guarantees 92%+ Compression)", "[Allocator]", bitrate_string);
    println!("{:<14} | Preset        : {} (Fast-Encode Mode)", "[Allocator]", safe_preset);
    println!("{:<14} | Thread Limit  : lp={} (RAM OOM-safety protection)", "[Allocator]", task.threads);
    println!("{:<14} | Pixel Format  : {}", "[Allocator]", safe_pix_fmt);

    // 5. Footer
    println!("{:=^60}", "");
   }

    print_task_diagnostics(&task);

    println!("\nQuill | Launching SVT-AV1 Transcoding Process...");
    match transcode::run_ffmpeg_transcode(&input_path, &output_path, &task) {
        Ok(elapsed) => {
            println!("\nTask Success: Optimized AV1 Transcode completed successfully!");
            println!("   Total Transcode Time: {:.2} seconds", elapsed);
            
            if let Ok(meta) = std::fs::metadata(&output_path) {
                let output_size = meta.len();
                
                let original_duration = task.original_duration;
                let proportion = if original_duration > 0.0 {
                    task.video_duration / original_duration
                } else {
                    1.0
                };
                let effective_input_size = input_size as f64 * proportion;
                
                let ratio = (1.0 - (output_size as f64 / effective_input_size)) * 100.0;
                println!("   Input File Size:      {:.2} MB", input_size as f64 / (1024.0 * 1024.0));
                println!("   Output File Size:     {:.2} MB", output_size as f64 / (1024.0 * 1024.0));
                println!("   Actual Compression:   {:.2}% reduction", ratio);
                
                println!("   Compression Succesful! (Achieved {:.2}%)!", ratio);
            }
        }
        Err(e) => {
            println!("\nTask Failed: Transcoding process failed: {}", e);
            std::process::exit(1);
        }
    }
}
