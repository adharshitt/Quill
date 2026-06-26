use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

pub struct PreAnalysisFeatures {
    pub spatial_energy: f64,
    pub temporal_energy: f64,
    pub luma_brightness: f64,
}

fn compute_dct(block: &[f64; 8]) -> [f64; 8] {
    let mut dct = [0.0f64; 8];
    let n_f64 = 8.0f64;
    for k in 0..8 {
        let mut sum = 0.0f64;
        for n in 0..8 {
            sum += block[n] * (std::f64::consts::PI * k as f64 * (2.0 * n as f64 + 1.0) / 16.0).cos();
        }
        let alpha = if k == 0 {
            (1.0 / n_f64).sqrt()
        } else {
            (2.0 / n_f64).sqrt()
        };
        dct[k] = alpha * sum;
    }
    dct
}

pub fn extract_features_low_memory(file_path: &str) -> std::io::Result<PreAnalysisFeatures> {
    let mut file = File::open(file_path)?;
    let file_len = file.metadata()?.len();
    
    if file_len < 512 {
        return Ok(PreAnalysisFeatures {
            spatial_energy: 0.05,
            temporal_energy: 0.1,
            luma_brightness: 0.5,
        });
    }

    let sample_size = 128;
    let step = file_len / sample_size as u64;
    let mut total_luma = 0.0f64;
    let mut buf = [0u8; 1];
    
    for i in 0..sample_size {
        file.seek(SeekFrom::Start(i as u64 * step))?;
        file.read_exact(&mut buf)?;
        total_luma += buf[0] as f64 / 255.0;
    }
    let luma_brightness = total_luma / sample_size as f64;

    let num_blocks = 16;
    let block_stride = file_len / num_blocks as u64;
    let mut spatial_sum = 0.0f64;
    let mut block_buf = [0u8; 8];

    for b in 0..num_blocks {
        file.seek(SeekFrom::Start(b as u64 * block_stride))?;
        file.read_exact(&mut block_buf)?;
        
        let mut block = [0.0f64; 8];
        for i in 0..8 {
            block[i] = block_buf[i] as f64 / 255.0;
        }
        
        let dct_coeffs = compute_dct(&block);
        
        for k in 4..8 {
            spatial_sum += dct_coeffs[k].abs();
        }
    }
    let spatial_energy = spatial_sum / (num_blocks as f64 * 4.0);

    let num_strides = 8;
    let stride_size = 32;
    let diff_stride = (file_len - stride_size as u64) / num_strides as u64;
    let mut temporal_diff = 0.0f64;
    let mut stride_buf_1 = [0u8; 32];
    let mut stride_buf_2 = [0u8; 32];

    for s in 0..(num_strides - 1) {
        let pos1 = s as u64 * diff_stride;
        let pos2 = (s + 1) as u64 * diff_stride;
        
        file.seek(SeekFrom::Start(pos1))?;
        file.read_exact(&mut stride_buf_1)?;
        
        file.seek(SeekFrom::Start(pos2))?;
        file.read_exact(&mut stride_buf_2)?;
        
        let mut diff = 0.0f64;
        for i in 0..stride_size {
            diff += (stride_buf_1[i] as f64 - stride_buf_2[i] as f64).abs();
        }
        temporal_diff += diff / stride_size as f64;
    }
    let temporal_energy = ((temporal_diff / num_strides as f64) / 255.0).min(1.0);

    Ok(PreAnalysisFeatures {
        spatial_energy,
        temporal_energy,
        luma_brightness,
    })
}

pub fn calculate_complexity_index(s: f64, t: f64, l: f64) -> f64 {
    let index = 10.0 * (s * 0.6 + t * 0.3 + l * 0.1);
    index.clamp(0.0, 10.0)
}
