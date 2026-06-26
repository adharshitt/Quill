#!/bin/bash
# Quill - Interactive TUI Wrapper for AV1/H.264 Video Optimization

# 1. Ask for input file if not provided as a command-line argument
INPUT_PATH="$1"
if [ -z "$INPUT_PATH" ]; then
    INPUT_PATH=$(whiptail --title "Quill Video Optimizer" --inputbox "Enter the path of the video file to optimize:" 10 60 3>&1 1>&2 2>&3)
    if [ $? -ne 0 ]; then
        echo "Operation canceled by user."
        exit 0
    fi
fi

# Trim whitespace
INPUT_PATH=$(echo "$INPUT_PATH" | xargs)

# Check if input file exists
if [ ! -f "$INPUT_PATH" ]; then
    whiptail --title "Error" --msgbox "File '$INPUT_PATH' does not exist." 10 50
    exit 1
fi

# 2. Select the compression method
CHOICE=$(whiptail --title "Select Compression Method" --menu \
"Choose one of the 5 supported transcoding methods:" 18 75 5 \
"1" "🆕 Our Optimized Rust Wrapper (10-bit AV1, RAM-safe)" \
"2" "Default AV1 (Clean CPU - 4K, lp=4)" \
"3" "Default H.264 (Normal CPU - 4K)" \
"4" "Tool-Optimized AV1 (CRF 30 - 4K, lp=4)" \
"5" "Tool-Optimized AV1 (Target-Bitrate 913kbps)" \
3>&1 1>&2 2>&3)

if [ $? -ne 0 ]; then
    echo "Operation canceled by user."
    exit 0
fi

# 3. Ask for optional seek offset
SEEK_VAL=$(whiptail --title "Optional: Seek Offset" --inputbox "Enter seek offset in seconds (e.g., 10) or leave blank to start from beginning:" 10 60 3>&1 1>&2 2>&3)
if [ $? -ne 0 ]; then
    echo "Operation canceled by user."
    exit 0
fi
SEEK_VAL=$(echo "$SEEK_VAL" | xargs)

# 4. Ask for optional duration limit
DUR_VAL=$(whiptail --title "Optional: Duration Limit" --inputbox "Enter encoding duration in seconds (e.g., 10) or leave blank for full video:" 10 60 3>&1 1>&2 2>&3)
if [ $? -ne 0 ]; then
    echo "Operation canceled by user."
    exit 0
fi
DUR_VAL=$(echo "$DUR_VAL" | xargs)

# Resolve directories and base names
DIR_NAME=$(dirname "$INPUT_PATH")
BASE_NAME=$(basename "$INPUT_PATH")
FILE_NAME="${BASE_NAME%.*}"

# Construct FFmpeg options for seek and duration
SEEK_ARG_FFMPEG=""
if [ ! -z "$SEEK_VAL" ]; then
    SEEK_ARG_FFMPEG="-ss $SEEK_VAL"
fi

DUR_ARG_FFMPEG=""
if [ ! -z "$DUR_VAL" ]; then
    DUR_ARG_FFMPEG="-t $DUR_VAL"
fi

clear
echo "==============================================================="
echo "=== Quill: Launching Transcode Operation ==="
echo "==============================================================="

case "$CHOICE" in
    1)
        OUTPUT_PATH="${DIR_NAME}/output_${FILE_NAME}_rust_optimized.webm"
        SEEK_ARGS=""
        if [ ! -z "$SEEK_VAL" ]; then
            SEEK_ARGS="--seek $SEEK_VAL"
        fi
        DUR_ARGS=""
        if [ ! -z "$DUR_VAL" ]; then
            DUR_ARGS="--duration $DUR_VAL"
        fi
        
        echo "Executing: /home/harshit/Pending/Quill/win/target/release/win --input \"$INPUT_PATH\" --output \"$OUTPUT_PATH\" $SEEK_ARGS $DUR_ARGS"
        /home/harshit/Pending/Quill/win/target/release/win --input "$INPUT_PATH" --output "$OUTPUT_PATH" $SEEK_ARGS $DUR_ARGS
        ;;
    2)
        OUTPUT_PATH="${DIR_NAME}/output_${FILE_NAME}_default_av1.webm"
        echo "Executing: ffmpeg -y $SEEK_ARG_FFMPEG -i \"$INPUT_PATH\" $DUR_ARG_FFMPEG -c:v libsvtav1 -svtav1-params lp=4 -c:a libopus \"$OUTPUT_PATH\""
        ffmpeg -y $SEEK_ARG_FFMPEG -i "$INPUT_PATH" $DUR_ARG_FFMPEG -c:v libsvtav1 -svtav1-params lp=4 -c:a libopus "$OUTPUT_PATH"
        ;;
    3)
        OUTPUT_PATH="${DIR_NAME}/output_${FILE_NAME}_h264.mp4"
        echo "Executing: ffmpeg -y $SEEK_ARG_FFMPEG -i \"$INPUT_PATH\" $DUR_ARG_FFMPEG -c:v libx264 -crf 23 -preset fast -c:a aac -b:a 128k \"$OUTPUT_PATH\""
        ffmpeg -y $SEEK_ARG_FFMPEG -i "$INPUT_PATH" $DUR_ARG_FFMPEG -c:v libx264 -crf 23 -preset fast -c:a aac -b:a 128k "$OUTPUT_PATH"
        ;;
    4)
        OUTPUT_PATH="${DIR_NAME}/output_${FILE_NAME}_tool_crf.webm"
        echo "Executing: ffmpeg -y $SEEK_ARG_FFMPEG -i \"$INPUT_PATH\" $DUR_ARG_FFMPEG -c:v libsvtav1 -crf 30 -preset 9 -svtav1-params keyint=10s:tune=0:enable-overlays=1:lp=4 -pix_fmt yuv420p10le -c:a libopus \"$OUTPUT_PATH\""
        ffmpeg -y $SEEK_ARG_FFMPEG -i "$INPUT_PATH" $DUR_ARG_FFMPEG -c:v libsvtav1 -crf 30 -preset 9 -svtav1-params keyint=10s:tune=0:enable-overlays=1:lp=4 -pix_fmt yuv420p10le -c:a libopus "$OUTPUT_PATH"
        ;;
    5)
        OUTPUT_PATH="${DIR_NAME}/output_${FILE_NAME}_tool_bitrate.webm"
        echo "Executing: ffmpeg -y $SEEK_ARG_FFMPEG -i \"$INPUT_PATH\" $DUR_ARG_FFMPEG -c:v libsvtav1 -b:v 913k -preset 9 -svtav1-params keyint=10s:tune=0:enable-overlays=1:lp=4 -pix_fmt yuv420p10le -c:a libopus \"$OUTPUT_PATH\""
        ffmpeg -y $SEEK_ARG_FFMPEG -i "$INPUT_PATH" $DUR_ARG_FFMPEG -c:v libsvtav1 -b:v 913k -preset 9 -svtav1-params keyint=10s:tune=0:enable-overlays=1:lp=4 -pix_fmt yuv420p10le -c:a libopus "$OUTPUT_PATH"
        ;;
esac

STATUS_CODE=$?
echo ""
if [ $STATUS_CODE -eq 0 ]; then
    whiptail --title "Transcode Complete" --msgbox "Successfully transcoded file to:\n$OUTPUT_PATH" 12 65
    echo "🟢 SUCCESS: File transcoded and saved to: $OUTPUT_PATH"
else
    whiptail --title "Transcode Failed" --msgbox "Encoding failed. Check terminal output for details." 10 55
    echo "🔴 ERROR: Transcode failed with exit status $STATUS_CODE"
fi
