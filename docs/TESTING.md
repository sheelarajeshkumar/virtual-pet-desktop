# Testing

## Automated checks

```bash
npm ci
npm run check
npm test
```

`npm run check` validates JavaScript syntax, Rust formatting, and Rust compilation. `npm test` runs the engine unit tests.

CI runs Rust formatting, Clippy, JavaScript syntax checks, tests, and a Tauri compile check on macOS and Windows.

## Manual smoke test

Before merging behavior, animation, window, or audio changes:

- Start with `npm run dev`.
- Confirm the overlay is transparent and mouse clicks pass through it.
- Move the cursor slowly and quickly; verify walk/run selection and smooth stopping.
- Verify left and right facing.
- Test Auto, Call over, Play, Bark, Sleep, and Settings.
- Confirm Bark plays once per selection and returns to Auto.
- On macOS, test another normal Space and a Chrome or Terminal fullscreen Space.
- On multiple monitors, test different display scaling and negative monitor origins when available.
- Check CPU use while idle and asleep.

## Adding tests

Keep engine tests in `src-tauri/src/engine.rs`. Add the smallest test that would fail if the behavior regressed. Platform integration still requires manual testing because CI cannot reliably automate desktop Spaces, audio output, or window layering.
