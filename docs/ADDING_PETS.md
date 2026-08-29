# Adding a Pet

Bundled pets are data-driven. A contributor adds a validated pack and one catalog entry; the renderer and Rust engine do not need a species-specific code change.

## 1. Create the pack

Copy the example structure:

```text
src/pet-packs/my-pet/
├── pet-pack.json
├── assets/
│   └── atlas.png
└── sounds/
    └── action.wav
```

Follow [PET_PACK_FORMAT.md](PET_PACK_FORMAT.md) and [ANIMATION_GUIDE.md](ANIMATION_GUIDE.md). Use a transparent, uniform atlas and record every asset's creator, source, changes, and license in the manifest.

## 2. Add the catalog entry

Add the pack to `src/pet-packs/catalog.json`:

```json
{
  "id": "my-pet",
  "name": "My Pet",
  "manifest": "pet-packs/my-pet/pet-pack.json"
}
```

The short ID must contain lowercase letters, numbers, or single hyphens and must be unique. The settings screen discovers the entry automatically.

## 3. Validate

```bash
node scripts/validate-pet-pack.mjs src/pet-packs/my-pet
npm run check
npm test
```

The validator rejects unsafe paths, missing assets, incorrect PNG dimensions, invalid frame references, and undocumented licenses.

## 4. Test every behavior

Run `npm run dev`, select the pet in Settings, and verify idle, walk, run, bark, play, sleep, attention, and sniff/groom behavior in both facing directions. Confirm sounds start only on state entry and obey the volume setting.

## Install a community pack

To test a pack without bundling it, open the tray menu, choose **Pet packs…**, then import the folder containing `pet-pack.json`. The app validates paths, assets, atlas dimensions, required animations, and license records before copying the referenced files into its private app-data directory. Select the installed pack in Settings.

Community pack IDs must use a reverse-domain-style value such as `example.my-pet`. Removing the active community pack safely switches the pet back to Puppy. Bundled Puppy and Cat packs cannot be removed from this screen.

## Current boundary

The built-in `cat` ID uses the cat movement profile. All other IDs currently use the default pet movement profile. Per-pack motion profiles should be proposed separately before changing the manifest schema.
