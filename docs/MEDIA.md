# Release media workflow

Release screenshots and recordings must show the real application and exclude personal
notifications, filenames, accounts, browser tabs, and model conversations.

## Current reviewed media

- `pet-on-desktop.png` — the real 128×128 transparent pet overlay capture.
- `pack-preview.png` — the real Tauri preview showing the fox rear diagonal run.
- `ai-disabled-by-default.png` — the real AI window with every optional AI feature off.
- `demo.mp4` — a 15-second, 20 fps recording of all four fox diagonal walk/run rows.
- `linkedin-demo.mp4` — a captioned 30-second, 1080p project overview for social posts.

The current captures were reviewed on macOS. A Windows capture set is still required before a
stable Windows release; CI packaging is not a substitute for visual review.

## Deterministic capture mode

Development builds accept a release-only environment selector without changing saved state:

```bash
VIRTUAL_PET_RELEASE_MEDIA=ai npm run dev
VIRTUAL_PET_RELEASE_MEDIA=preview npm run dev
```

The AI mode forces all AI toggles off. Preview mode selects the bundled fox and cycles only the
front/back diagonal walk and run rows. This selector is read only at startup and has no production
UI.

On macOS, `scripts/macos-window-ids.m` identifies the app-owned window. The included
`scripts/macos-capture-window.m` captures fully composited frames at a fixed rate, avoiding partial
WebView redraws in window recordings. Encode reviewed frames with H.264/YUV420p and `faststart` for
GitHub playback. `scripts/capture-release-media.sh` remains available for interactive screenshots.
Run `scripts/create-linkedin-demo.sh` to rebuild the social video from the reviewed captures and
the three SVG cards stored beside it.

On Windows, use `Win+Shift+S` and the Snipping Tool recorder, then complete the privacy and GUI
checks in [WINDOWS_TESTING.md](WINDOWS_TESTING.md). Never present generated artwork as an app
screenshot; generated pet assets must be identified in [ASSETS.md](../ASSETS.md).
