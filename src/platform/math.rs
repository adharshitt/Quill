pub fn calculate_proportional_size(input_file_size: u64, proportion: f64) -> Result<f64, String> {
    if proportion.is_nan() || proportion.is_infinite() || proportion <= 0.0 {
        return Err(format!("Invalid proportional scale: {}", proportion));
    }
    
    let size = input_file_size as f64 * proportion;
    if size.is_nan() || size.is_infinite() || size <= 0.0 {
        return Err(format!("Proportional size calculation result is invalid: {}", size));
    }
    
    Ok(size)
}

pub fn calculate_target_bitrate(target_input_size: f64, video_duration: f64) -> Result<u32, String> {
    if video_duration.is_nan() || video_duration.is_infinite() || video_duration <= 0.0 {
        return Err("Video duration is invalid for bitrate calculation".to_string());
    }
    
    let target_bitrate_bps = (target_input_size * 0.075 * 8.0) / video_duration;
    if target_bitrate_bps.is_nan() || target_bitrate_bps.is_infinite() || target_bitrate_bps < 0.0 {
        return Err("Calculated target bitrate result is invalid".to_string());
    }
    
    Ok(target_bitrate_bps as u32)
}
