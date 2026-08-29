# Animation and Asset Guide

## Puppy atlas contract

The current puppy atlas is a transparent 4-column × 8-row image. Each source frame is square.

| Row | Behavior | Frames |
| --- | --- | --- |
| 0 | Idle | 4 |
| 1 | Walk | 4 |
| 2 | Run | 4 |
| 3 | Bark | 4 |
| 4 | Play | 4 |
| 5 | Sleep | 4 |
| 6 | Attention | 4 |
| 7 | Sniff | 4 |

The renderer mirrors right-facing art for left-facing movement. Do not include both directions unless a future art style requires asymmetric frames.

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

1. Confirm width is divisible by 4 and height by 8.
2. Confirm all cells have identical dimensions.
3. Confirm alpha is preserved.
4. Confirm no frame is clipped when mirrored.
5. Record the creator, source, tools, modifications, and license in `ASSETS.md`.

Do not commit layered source files unless they are useful and reasonably sized. Link to large source packages from the pull request when necessary.
