# Animation and Asset Guide

## Bundled atlas contract

The bundled puppy, cat and fox base atlases are transparent 4-column × 8-row images. Each source frame is square. The manifest is authoritative for custom pack dimensions, frame order, and timing.

| Row | Behavior | Frames |
| --- | --- | --- |
| 0 | Idle | 4 |
| 1 | Walk | 4 |
| 2 | Run | 4 |
| 3 | Bark or vocal action | 4 |
| 4 | Play | 4 |
| 5 | Sleep | 4 |
| 6 | Attention | 4 |
| 7 | Sniff | 4 |

The renderer mirrors right-facing art for left-facing movement. Do not include duplicate left-facing atlases unless a future format explicitly supports asymmetric frames.

## Directional movement

For pets that move around the desktop, use three camera groups:

- `horizontal`: side profile. The base atlas already fills this role unless overridden.
- `vertical`: `up` shows the back of the pet; `down` shows its face and chest.
- `diagonal`: three-quarter rear (`up`) and three-quarter front (`down`) views.

Keep the same body size, foot baseline, palette, outline, and frame cadence across atlases. The puppy's vertical and diagonal atlases are the reference layout: down walk/run in rows 0–1 and up walk/run in rows 2–3. The manifest is authoritative, so another row layout is valid.

## Visual requirements

- Use a transparent background.
- Keep the feet aligned to a consistent ground line.
- Keep body scale and anchor points consistent between frames.
- Make walk and run contact poses clearly different.
- Avoid large empty margins that make the pet appear too small.
- Test at the final 112 CSS-pixel pet size, not only while zoomed in.

## Motion principles

- Idle motion should be subtle and loop without a visible jump.
- A walk needs believable alternating contact and passing poses.
- A run needs clear compression, extension, flight, and landing.
- One-shot actions should reach their readable key pose before returning to Auto.
- Frame timing belongs in `animation_frame` in `engine.rs`; artwork should not encode timing assumptions.

## Export checks

Before submitting an atlas:

1. Confirm each atlas pixel size exactly matches its declared rows, columns, and frame size.
2. Confirm all cells in an atlas have identical dimensions.
3. Confirm alpha is preserved.
4. Confirm no frame is clipped when mirrored.
5. Check back/front and all four diagonals in **Preview pet packs…** from the tray.
6. Record the creator, source, tools, modifications, and license in the pack manifest and `ASSETS.md`.

Do not commit layered source files unless they are useful and reasonably sized. Link to large source packages from the pull request when necessary.
