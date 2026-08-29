# Troubleshooting

## The pet does not start

Run `npm run dev` from the repository root and read the first Tauri or Cargo error. Confirm Node.js, Rust, and the platform-specific Tauri prerequisites are installed.

## The pet is hidden behind fullscreen apps on macOS

Use the current `main` branch. The pet requires the Accessory activation policy plus AppKit fullscreen-overlay collection behavior. Restart the app after native window changes.

## The pet does not follow the cursor

Select **Auto** or **Call over** from the paw menu. Sleep and Play are persistent modes; Bark is one-shot and returns to Auto.

## Bark has no sound

Confirm system output is not muted and `src/assets/bark.wav` exists. Fully restart the app after replacing a bundled audio file because the WebView may cache it.

## A monitor transition stops at an edge

Include the operating-system version, monitor arrangement, resolution, and scaling in a bug report. On macOS, also state whether Displays have separate Spaces is enabled.

## Build fails after dependency changes

```bash
npm ci
cargo clean --manifest-path src-tauri/Cargo.toml
npm run check
```

Do not delete user settings or application-support directories while troubleshooting a build.
