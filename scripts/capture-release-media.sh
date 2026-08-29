#!/bin/sh
set -eu

output_directory=${1:-docs/media}
mkdir -p "$output_directory"

for name in pet-on-desktop pack-preview ai-disabled-by-default; do
  printf 'Select the app area for %s.png, then press Return.\n' "$name"
  screencapture -i -o "$output_directory/$name.png"
done

printf 'Saved reviewed selections under %s. Record demo.mp4 separately with Shift+Command+5.\n' "$output_directory"
