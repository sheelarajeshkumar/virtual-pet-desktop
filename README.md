# Virtual Pet Desktop

[![CI](https://github.com/sheelarajeshkumar/virtual-pet-desktop/actions/workflows/ci.yml/badge.svg)](https://github.com/sheelarajeshkumar/virtual-pet-desktop/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Platforms](https://img.shields.io/badge/platform-macOS%20%7C%20Windows-blue)](#platform-support)

An open-source, cross-platform animated desktop pet built with Tauri. Add pets, behaviors, animations, sounds, and community features for macOS and Windows.

## Features

- Transparent, click-through 128×128 desktop overlay
- Smooth walk, run, idle, bark, play, sleep, attention, and sniff animations
- Cursor following with acceleration and deceleration
- macOS fullscreen Spaces and mixed-DPI monitor support
- Native tray controls for Auto, Call over, Play, Bark, Sleep, and Settings
- One-shot bark action with a real CC0 dog recording
- Persistent pet name and species settings
- Lightweight JavaScript canvas renderer with a Rust movement engine

## Project status

Virtual Pet Desktop is an early community release. The puppy experience is the most complete. The cat renderer is currently a procedural placeholder, and adding a new species still requires small Rust and JavaScript changes. A data-driven pet-pack format is planned.

## Requirements

- [Node.js](https://nodejs.org/) 20 or newer
- [Rust](https://www.rust-lang.org/tools/install) 1.85 or newer
- Tauri's [platform prerequisites](https://v2.tauri.app/start/prerequisites/)

## Run locally

```bash
git clone git@github.com:sheelarajeshkumar/virtual-pet-desktop.git
cd virtual-pet-desktop
npm install
npm run dev
```

Use the paw icon in the system tray or macOS menu bar to control the pet.

## Controls

| Action | Behavior |
| --- | --- |
| Auto | Follows recent cursor movement, wanders, and eventually sleeps |
| Call over | Continuously follows the cursor |
| Play | Plays around the cursor |
| Bark | Barks once, then returns to Auto |
| Sleep | Walks home and sleeps |
| Name and pet | Updates the saved name and species |

## Checks

```bash
npm ci
npm run check
npm test
```

## Build installers

Build macOS artifacts on macOS and Windows artifacts on Windows:

```bash
npm run build
```

Unsigned development builds are created under `src-tauri/target/release/bundle`. Public releases must be Apple-signed/notarized and Windows code-signed.

## Platform support

| Platform | Status |
| --- | --- |
| macOS 10.15+ | Actively developed and tested |
| Windows 10/11 | Supported; community testing is welcome |
| Linux | Not currently supported |

## Contributing

Contributions of code, pixel art, animations, sounds, documentation, testing, and ideas are welcome. Start with [CONTRIBUTING.md](CONTRIBUTING.md), then read the relevant guide:

- [Architecture](docs/ARCHITECTURE.md)
- [Adding a pet](docs/ADDING_PETS.md)
- [Animation and asset guide](docs/ANIMATION_GUIDE.md)
- [Roadmap](ROADMAP.md)

Please report security issues privately according to [SECURITY.md](SECURITY.md).

## Asset credits

The puppy sprite sheets are original project assets. Bark audio is adapted from [Small Dog Barking Behind Door](https://bigsoundbank.com/small-dog-barking-behind-door-s3537.html) by Joseph SARDIN and Axeline T., released under CC0. See [ASSETS.md](ASSETS.md) for details.

## License

Source code and original project assets are available under the [MIT License](LICENSE). Third-party assets retain the licenses listed in [ASSETS.md](ASSETS.md).
