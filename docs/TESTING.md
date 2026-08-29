# Testing

## Automated checks

```bash
npm ci
npm run check
npm test
```

`npm run check` validates JavaScript syntax, both bundled pet packs, Rust formatting, and Rust compilation. `npm test` runs the pet-pack loader tests plus the Rust movement, care, inventory, routine, and external-pack security tests.

CI runs Rust formatting, Clippy, JavaScript syntax checks, tests, and a Tauri compile check on macOS and Windows.

## Manual smoke test

Before merging behavior, animation, window, or audio changes:

- Start with `npm run dev`.
- Confirm the overlay is transparent and mouse clicks pass through it.
- Move the cursor slowly and quickly; verify walk/run selection and smooth stopping.
- Verify left and right facing.
- Test Auto, Call over, Play, Bark, Sleep, Care, and Settings.
- Switch between puppy and cat; verify every behavior uses the correct pack row.
- Change size, speed, volume, reduced motion, and care difficulty.
- Confirm inventory and care actions persist, restocking restores supplies, and low needs display a visible request above the pet.
- Enable the day/night routine and verify recent cursor movement still wakes/follows at night.
- Enable notifications, grant permission, and verify critical care alerts are rate-limited; disable them again.
- Import a valid community pack, select it in Settings, then remove it from Pet packs.
- Confirm Bark plays once per selection and returns to Auto.
- On macOS, test another normal Space and a Chrome or Terminal fullscreen Space.
- On multiple monitors, test different display scaling and negative monitor origins when available.
- Check CPU use while idle and asleep.

Use [WINDOWS_TESTING.md](WINDOWS_TESTING.md) for the Windows 10/11 packaging and GUI checklist.

## Adding tests

Keep engine tests in `src-tauri/src/engine.rs`. Add the smallest test that would fail if the behavior regressed. Platform integration still requires manual testing because CI cannot reliably automate desktop Spaces, audio output, or window layering.
