# Architecture

Virtual Pet Desktop is a small Tauri v2 application with a static JavaScript frontend and a Rust movement engine.

## Runtime flow

```text
System cursor and monitors
          |
          v
Tauri command: tick() ---------> Rust PetEngine
          |                       - mode and behavior
          |                       - velocity and position
          |                       - animation frame
          v
Move native overlay window
          |
          v
PetSnapshot returned to JavaScript
          |
          v
Canvas renders sprite frame and plays action audio
```

The frontend schedules the next `tick` using the delay returned in each snapshot. Movement states update at roughly 60 Hz, while idle and sleep states use slower intervals to reduce work.

## Source layout

| Path | Responsibility |
| --- | --- |
| `src/index.html` | Pet overlay document |
| `src/pet.js` | Canvas rendering, sprite selection, bark playback, Tauri calls |
| `src/pet.css` | Transparent overlay styling and display size |
| `src/settings.*` | Pet name and species settings UI |
| `src/assets/` | Sprite atlases and audio |
| `src-tauri/src/engine.rs` | Platform-neutral state, movement, behavior, animation timing, tests |
| `src-tauri/src/lib.rs` | Tauri commands, native window handling, monitor geometry, tray, settings storage |
| `src-tauri/tauri.conf.json` | Window, bundle, security, and app metadata |

## State model

`Mode` represents a user-selected control state: Auto, Follow, Play, Bark, or Sleep. `Behavior` represents the animation currently rendered, such as Walk, Run, Attention, or Sniff.

The Bark menu action is intentionally one-shot. The engine displays it briefly and then returns to Auto. Auto follows recent cursor movement, wanders after inactivity, and sleeps after extended inactivity.

## Platform boundaries

Shared movement logic stays in `engine.rs`. Native coordinate and window behavior stays in `lib.rs` behind platform `cfg` gates.

- macOS converts Tauri geometry into a shared logical desktop space and uses AppKit overlay flags for fullscreen Spaces.
- Windows uses physical desktop coordinates and Tauri's native window APIs.

Do not move native window calls into the renderer or add operating-system conditionals to `engine.rs`.

## Persistence

Settings are serialized as JSON under the operating system's application config directory. No cloud service, telemetry, account, or network runtime is required.

## Security model

The pet WebView loads bundled local files under the configured content security policy. The overlay is click-through and exposes only the Tauri commands registered in `lib.rs`. New commands should validate all frontend input and expose the minimum required capability.
