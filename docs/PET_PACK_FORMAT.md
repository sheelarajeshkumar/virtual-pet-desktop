# Pet Pack Format

A pet pack is a directory containing a `pet-pack.json` manifest and every asset it references. The bundled puppy is the canonical example in `src/pet-packs/puppy/`.

## Validate a pack

```bash
node scripts/validate-pet-pack.mjs src/pet-packs/puppy
```

Validation is intentionally dependency-free so contributors only need Node.js. Add a valid bundled pack to `src/pet-packs/catalog.json`, or import its folder from **Pet packs…** in the tray menu. Imported packs are copied into app-managed storage after stricter native validation.

## Manifest

`schemaVersion` is currently `1`. Unknown fields are allowed so compatible tooling can add metadata without breaking older validators.

| Field | Required | Contract |
| --- | --- | --- |
| `schemaVersion` | Yes | Integer `1` |
| `id` | Yes | Lowercase reverse-domain-style identifier using letters, digits, dots, and hyphens |
| `name` | Yes | Non-empty display name |
| `version` | Yes | Semantic version such as `1.0.0` |
| `species` | Yes | Lowercase identifier |
| `description` | Yes | Non-empty summary |
| `authors` | Yes | Non-empty array of objects with a `name` |
| `atlas` | Yes | PNG file plus positive `columns`, `rows`, `frameWidth`, and `frameHeight` |
| `animations` | Yes | Named animation definitions; `idle`, `walk`, `run`, and `sleep` are required |
| `directional` | No | Direction-specific horizontal, vertical, or diagonal atlas groups |
| `sounds` | No | Named local audio definitions |
| `license` | Yes | Pack SPDX identifier and per-asset provenance |

### Atlas and animations

The atlas must be a uniform grid. Its pixel width must equal `columns × frameWidth`; its height must equal `rows × frameHeight`.

Each animation defines:

- `row`: zero-based atlas row.
- `frames`: non-empty array of zero-based atlas columns, in playback order.
- `frameDurationMs`: duration of each frame, from 40 to 60,000 milliseconds.
- `loop`: whether playback repeats.

The base atlas is also the fallback for every direction, so existing packs remain valid. The renderer mirrors right-facing art for left-facing movement.

### Optional directional animation

Use `directional` when a side-view frame would look wrong while moving toward or away from the viewer. Each group owns a normal atlas definition. `horizontal.animations` maps behavior names directly; `vertical` and `diagonal` nest behaviors below `up` and `down`. Missing groups or behaviors fall back to the base atlas.

```json
"directional": {
  "vertical": {
    "atlas": {
      "file": "assets/vertical.png",
      "columns": 4,
      "rows": 4,
      "frameWidth": 224,
      "frameHeight": 224
    },
    "animations": {
      "down": {
        "walk": { "row": 0, "frames": [0, 1, 2, 3], "frameDurationMs": 120, "loop": true },
        "run": { "row": 1, "frames": [0, 1, 2, 3], "frameDurationMs": 85, "loop": true }
      },
      "up": {
        "walk": { "row": 2, "frames": [0, 1, 2, 3], "frameDurationMs": 120, "loop": true },
        "run": { "row": 3, "frames": [0, 1, 2, 3], "frameDurationMs": 85, "loop": true }
      }
    }
  }
}
```

`diagonal` has the same `up`/`down` shape. A `horizontal` override uses `"animations": { "walk": {...}, "run": {...} }`. Every directional atlas must be a safe local PNG, match its declared grid dimensions, and have a `license.assets` entry.

Catalog entries use a short lowercase `id`, display `name`, and pack-relative `manifest` URL. The current movement engine gives the built-in `cat` ID the cat motion profile and uses the default pet profile for other IDs.

### Sounds

Each sound has a `file`, `volume` between `0` and `1`, and Boolean `loop`. Supported file extensions are `.wav`, `.mp3`, `.ogg`, and `.m4a`. Action names should match animation or behavior names where practical.

### Paths and licenses

All paths are POSIX-style and relative to the pack directory. Absolute paths, backslashes, empty segments, and `..` traversal are rejected. Symlinks that resolve outside the pack are also rejected.

Every referenced base/directional atlas or sound must have exactly one entry in `license.assets` containing:

- `path`: the same relative asset path.
- `creator`: original creator or project.
- `source`: stable source URL or repository URL.
- `spdx`: SPDX license identifier, for example `MIT` or `CC0-1.0`.
- `modifications`: what changed, or `Unmodified`.

Only submit assets whose licenses permit redistribution in this project.

## Minimal example

```json
{
  "schemaVersion": 1,
  "id": "example.fox",
  "name": "Fox",
  "version": "1.0.0",
  "species": "fox",
  "description": "A small desktop fox.",
  "authors": [{ "name": "Example Artist" }],
  "atlas": {
    "file": "assets/atlas.png",
    "columns": 4,
    "rows": 4,
    "frameWidth": 224,
    "frameHeight": 224
  },
  "animations": {
    "idle": { "row": 0, "frames": [0, 1, 2, 3], "frameDurationMs": 400, "loop": true },
    "walk": { "row": 1, "frames": [0, 1, 2, 3], "frameDurationMs": 120, "loop": true },
    "run": { "row": 2, "frames": [0, 1, 2, 3], "frameDurationMs": 80, "loop": true },
    "sleep": { "row": 3, "frames": [0, 1, 2, 3], "frameDurationMs": 500, "loop": true }
  },
  "license": {
    "spdx": "MIT",
    "assets": [{
      "path": "assets/atlas.png",
      "creator": "Example Artist",
      "source": "https://example.com/fox",
      "spdx": "CC0-1.0",
      "modifications": "Unmodified"
    }]
  }
}
```

## Validator self-test

```bash
node scripts/validate-pet-pack.mjs --self-test
```

The self-test validates the bundled puppy and confirms that unsafe traversal and invalid frame definitions are rejected. `npm run check:pets` validates every bundled pack used by the application.

## Safe extension boundary

Pet packs are data, not plugins: manifests cannot contain JavaScript or executable commands. Community routines use a separate bounded JSON format with native validation; see [BEHAVIOR_EXTENSIONS.md](BEHAVIOR_EXTENSIONS.md). Unknown pack metadata remains inert.
