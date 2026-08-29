# Optional AI Companion

AI Companion is an optional local feature. It is disabled by default, while movement, care,
sounds, pet packs, and every other core feature continue to work without it.

## Start local Ollama

Install [Ollama](https://ollama.com/), then run:

```bash
ollama serve
ollama pull llama3.2
```

Open **AI Companion** from the tray, keep the endpoint at
`http://127.0.0.1:11434`, choose the installed model, and enable the master switch. The endpoint
accepts only plain HTTP loopback addresses (`localhost`, `127.0.0.1`, or `[::1]`); remote and
cloud endpoints are rejected.

## Features and privacy

- Chat automatically includes the current pet name, species, mood, needs, inventory, and sleep state; optional user context can add situational detail.
- Local memory is independently opt-in and retains at most 20 recent messages.
- Proactive suggestions are independently opt-in with a 5–240 minute cooldown.
- Voice output uses the operating system browser voice through `speechSynthesis` and falls back
  to text when unavailable.
- Model-suggested actions are restricted to `feed`, `play`, `wash`, `pet`, `sleep`, `wake`, and
  `bark`. A suggestion never runs until the user presses its confirmation button.
- No telemetry, cloud service, API key, or cloud-key storage is included. Settings and optional
  memory stay in the app's local configuration directory.
- Requests use a 3-second connection timeout, 15-second I/O timeout, a 64 KB request limit, and
  a 256 KB response limit. Ollama failure affects only the AI command.

Disable AI Companion at any time without changing the pet. Disable memory or press **Clear
memory** to remove saved conversation history.

## Tauri integration contract

`src-tauri/src/ai.rs` intentionally has no dependency on the pet engine. The app shell owns a
`Mutex<AiCompanion>` and exposes these commands to `ai.js`:

| Command | Input | Output |
| --- | --- | --- |
| `get_ai_settings` | none | `AiSettings` |
| `save_ai_settings` | `{ settings: AiSettings }` | `AiSettings` |
| `clear_ai_memory` | none | `()` |
| `ai_chat` | `{ input: AiChatInput }` | `AiChatResponse` |
| `ai_proactive` | `{ mood, context }` | `AiChatResponse` or `null` when cooling down |
| `confirm_ai_action` | `{ action }` | `()` after re-validating the allowlist |

For chat, lock only long enough to call `prepare_chat`, run `execute_chat` on Tauri's blocking
pool, then lock again for `finish_chat`. This keeps the local model request away from the UI and
pet tick loops. The confirmation command must parse `SuggestedAction` again before routing to the
existing care/mode functions.
