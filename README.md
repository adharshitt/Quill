# Quill

Quill is an optimized, resource-aware, multi-module video transcoding engine written in Rust, paired with an interactive Terminal User Interface (TUI) wrapper. It performs spatial-temporal video telemetry analysis using a lightweight Discrete Cosine Transform (DCT) pre-scan, forecasts encoding complexity, dynamically maps thread limits to ensure memory safety (OOM prevention), and calculates optimal target bitrates to achieve high compression ratios without visual quality starvation.

---

## 🛠️ Architecture

The project is structured modularly:
*   **[src/main.rs](file:///home/harshit/Pending/Quill/win/src/main.rs)**: CLI argument parser featuring path sanitization to prevent flag injection vulnerability.
*   **[src/telemetry.rs](file:///home/harshit/Pending/Quill/win/src/telemetry.rs)**: Sub-millisecond pre-scan that seeks directly to byte boundaries to extract spatial, temporal, and contrast features with a `< 1 KB` RAM footprint, returning a deterministic Complexity Index (CI).
*   **[src/allocator.rs](file:///home/harshit/Pending/Quill/win/src/allocator.rs)**: Host system resource analyzer that maps cpu cores and caps concurrent threads based on detected memory to prevent kernel OOM termination. It also calculates proportional target bitrates to satisfy the $\ge 92\%$ compression goal.
*   **[src/transcode.rs](file:///home/harshit/Pending/Quill/win/src/transcode.rs)**: Command builder and process execution orchestrator for FFmpeg subprocesses.
*   **[quill.sh](file:///home/harshit/Pending/Quill/win/quill.sh)**: A `whiptail`-powered interactive Terminal User Interface (TUI) wrapper.

---

## 🚀 Key Features

*   **Native 4K Memory Safety**: Avoids WSL/container crashes by limiting SVT-AV1 CPU thread buffer allocation to a safe parallel execution count (e.g., `lp=4` or `lp=6`) dynamically depending on system memory.
*   **Proportional Anti-Starvation Bitrate**: Maximizes compression ratio to hit $\ge 92\%$ reduction targets, while preventing video blockiness by scaling the upper bitrate limit proportionally to 4.0 Mbps.
*   **Flag/Command Injection Protection**: Ensures file paths do not start with hyphens to avoid flag hijacking in sub-invocations.
*   **Interactive TUI**: Interactive configuration and selection of 5 compression methods, segment-seeking (`--seek`), and duration limits (`--duration`).

---

## 📦 Installation & Setup

### Prerequisites
Make sure `rustc`, `cargo`, `ffmpeg`, and `whiptail` are installed on your Linux / WSL environment:
```bash
sudo apt update
sudo apt install -y ffmpeg build-essential dialog
```

### 1. Build the Rust Optimizer
Clone the repository and build the project in release mode:
```bash
cargo build --release
```

### 2. Install the Quill Wrapper Globally
Install the TUI script as a global command in your system `PATH`:
```bash
sudo cp quill.sh /usr/local/bin/quill
sudo chmod +x /usr/local/bin/quill
```

---

## 🎮 Usage

You can launch the **Interactive TUI** from any folder by typing a single word:
```bash
quill
```

Alternatively, run the TUI by passing the input file directly:
```bash
quill path/to/video.mp4
```

### Supported Compression Methods
1.  **Quill AD01**: 10-bit AV1, RAM-safe, proportional bitrate.
2.  **Default AV1**: Clean CPU-based SVT-AV1 (lp=4, Preset 10).
3.  **Default H.264**: Standard libx264 fast CPU compression.
4.  **Tool-Optimized AV1 (CRF 30)**: Target CRF-based high-quality AV1 encoding.
5.  **Tool-Optimized AV1 (Target-Bitrate)**: Bitrate-restricted AV1 encoding.

### Segment Capping (Seek & Duration)
The interface will automatically prompt you for optional seek offsets and duration limits. Perfect for test-transcoding short segments:
*   **Seek Offset**: Seek to any seconds mark in the input file (uses fast input-seeking `-ss`).
*   **Duration**: Cap the transcode length to a specified duration (uses `-t` output-capping).
