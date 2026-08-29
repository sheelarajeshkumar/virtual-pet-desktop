# Adding a Pet

The current release does not yet have a plug-in pet-pack format. Adding a species requires a focused Rust and JavaScript change.

## 1. Prepare the assets

Create an atlas that follows [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md). Record the asset source and license in `ASSETS.md`.

## 2. Add the species to Rust

In `src-tauri/src/engine.rs`:

1. Add a variant to `PetKind`.
2. Update `PetKind::parse`.
3. Choose appropriate stop distances and walk/run/play speeds.
4. Add or update one focused engine test.

Keep movement rules in the engine rather than the renderer.

## 3. Expose the species in settings

Update `src/settings.html` and `src/settings.js` so users can choose and save the new value. Preserve keyboard access and visible labels.

## 4. Render the species

In `src/pet.js`:

1. Load the atlas once.
2. Define the behavior-to-row mapping.
3. Draw the current four-frame animation from the snapshot.
4. Respect `snapshot.facing` by mirroring horizontally.
5. Keep rendering inside the existing 256×256 canvas and 128×128 overlay.

## 5. Add optional sounds

Bundle short local audio under `src/assets/`. Trigger action sounds on state entry, not on every render tick. Keep sounds optional and document their licenses.

## 6. Verify behavior

Run:

```bash
npm run check
npm test
npm run dev
```

Manually verify both facing directions, each animation row, slow and fast cursor movement, fullscreen behavior on macOS, and tray actions.

## Future pet-pack format

The planned format will move species metadata, animation rows, speeds, and sound mappings into validated data files. Until that lands, avoid introducing a one-off abstraction for a single new pet; extend the existing model cleanly.
