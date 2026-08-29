# Asset Licensing and Provenance

Every bundled asset must have a known source and a license compatible with open-source redistribution.

## Project assets

| Asset | Source | License |
| --- | --- | --- |
| `src/assets/dog-atlas-v1.png` | Original Virtual Pet Desktop artwork | MIT |
| `src/assets/dog-atlas-v2.png` | Original Virtual Pet Desktop artwork | MIT |
| `src/pet-packs/puppy/assets/dog-atlas.png` | Packaged copy of original Virtual Pet Desktop artwork | MIT |
| `src/pet-packs/puppy/assets/dog-directional-atlas.png` | Original project artwork created with OpenAI image generation from the puppy reference and resized to a 4×4 atlas | MIT |
| `src/pet-packs/puppy/assets/dog-diagonal-atlas.png` | Original project artwork created with OpenAI image generation from the puppy references and resized to a 4×4 atlas | MIT |
| `src/pet-packs/cat/assets/cat-atlas.png` | Original project artwork created with OpenAI image generation and resized to a 4×8 atlas | MIT |
| `src/pet-packs/fox/assets/fox-atlas.png` | Original project artwork created with OpenAI image generation and resized to a 4×8 atlas | MIT |
| `src/pet-packs/fox/assets/fox-directional-atlas.png` | Original project artwork created with OpenAI image generation and resized to a 4×4 front/back atlas | MIT |
| `src-tauri/icons/` | Original Virtual Pet Desktop artwork and generated icon sizes | MIT |

## Third-party assets

| Asset | Creator/source | Changes | License |
| --- | --- | --- | --- |
| `src/assets/bark.wav` | Joseph SARDIN and Axeline T., [Small Dog Barking Behind Door](https://bigsoundbank.com/small-dog-barking-behind-door-s3537.html) | A single bark was trimmed, filtered, faded, normalized, converted to mono, and resampled to 44.1 kHz PCM WAV | [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/) |
| `src/assets/dog-snoring.mp3` | Filmscore, [Small Dog Snoring 3](https://freesound.org/people/Filmscore/sounds/516871/) | Freesound high-quality MP3 preview bundled unchanged | [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/) |
| `src/pet-packs/puppy/sounds/` | Packaged copies of the bark and snoring sources above | No additional changes | CC0 1.0 |

## Contribution requirements

Asset pull requests must include the original creator, a stable source URL, the exact license, and a summary of modifications. Do not submit assets copied from commercial games, websites, social media, or AI tools without clear redistribution rights.
