#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
output="${1:-$repo_root/docs/media/linkedin-demo.mp4}"
temp_dir="$(mktemp -d "${TMPDIR:-/tmp}/virtual-pet-linkedin.XXXXXX")"
trap 'rm -rf "$temp_dir"' EXIT

qlmanage -t -s 1920 -o "$temp_dir" \
  "$repo_root/docs/media/linkedin-title.svg" \
  "$repo_root/docs/media/linkedin-ai.svg" \
  "$repo_root/docs/media/linkedin-close.svg"

ffmpeg -y \
  -loop 1 -framerate 30 -t 5 -i "$temp_dir/linkedin-title.svg.png" \
  -loop 1 -framerate 30 -t 5 -i "$repo_root/src/pet-packs/fox/assets/fox-diagonal-atlas.png" \
  -i "$repo_root/docs/media/demo.mp4" \
  -loop 1 -framerate 30 -t 6 -i "$temp_dir/linkedin-ai.svg.png" \
  -loop 1 -framerate 30 -t 6 -i "$repo_root/docs/media/ai-disabled-by-default.png" \
  -loop 1 -framerate 30 -t 4 -i "$temp_dir/linkedin-close.svg.png" \
  -filter_complex "
    [0:v]crop=1920:1080:0:420[titlebg];
    [1:v]crop=224:224:0:0,scale=450:450:flags=lanczos[titlefox];
    [titlebg][titlefox]overlay=x=1400:y=315:format=auto,
      fade=t=in:st=0:d=0.5,fade=t=out:st=4.5:d=0.5,format=yuv420p[title];
    [2:v]fps=30,trim=duration=15,setpts=PTS-STARTPTS,
      scale=1352:1080:force_original_aspect_ratio=decrease,
      pad=1920:1080:(ow-iw)/2:(oh-ih)/2:0x0d0c11,
      format=yuv420p[motion];
    [3:v]crop=1920:1080:0:420[aibg];
    [4:v]scale=-2:960:flags=lanczos[aishot];
    [aibg][aishot]overlay=x=1070:y=60:format=auto,
      fade=t=in:st=0:d=0.4,fade=t=out:st=5.6:d=0.4,format=yuv420p[aicard];
    [5:v]crop=1920:1080:0:420,
      fade=t=in:st=0:d=0.4,format=yuv420p[close];
    [title][motion][aicard][close]concat=n=4:v=1:a=0,format=yuv420p[out]
  " \
  -map "[out]" -t 30 -r 30 -c:v libx264 -preset medium -crf 20 \
  -movflags +faststart -an "$output"

ffprobe -v error -show_entries format=duration,size \
  -show_entries stream=codec_name,width,height,r_frame_rate,pix_fmt \
  -of json "$output"
