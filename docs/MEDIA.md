# Release media workflow

Release media must show the real application. Do not composite a pet over an unrelated desktop or include personal notifications, filenames, accounts, or browser tabs.

## Capture set

Create these files under `docs/media/` for a release:

1. `pet-on-desktop.png` — the pet overlay on a clean desktop.
2. `pack-preview.png` — **Preview pet packs…** showing a directional walk/run frame.
3. `ai-disabled-by-default.png` — AI Companion with its master switch off.
4. `demo.mp4` — 15–30 seconds showing cursor follow, Bark selected once, Sleep, and switching pets.

Use a 16:9 desktop at 1920×1080 or higher, keep the system pointer away from the pet, and capture one macOS and one Windows example before a stable release.

## macOS screenshots

Run the app, open the intended window, then use the interactive capture script:

```bash
./scripts/capture-release-media.sh docs/media
```

The script uses interactive selection for every image so it cannot silently capture the whole desktop. Review every file before committing it. Record `demo.mp4` with the macOS screenshot toolbar (`Shift+Command+5`).

## Windows screenshots

Use `Win+Shift+S` for each image and the Snipping Tool screen recorder for `demo.mp4`. Complete the privacy review in [WINDOWS_TESTING.md](WINDOWS_TESTING.md).

Generated artwork may be used as a clearly labelled promotional illustration, but it must not be presented as an application screenshot.
