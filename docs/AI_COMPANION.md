# Optional AI Companion

AI Companion is local, optional, and disabled by default. Movement, care, sounds, pet packs,
and behavior extensions work normally when AI is disabled or no model server is installed.

## Supported providers

| Provider | Default endpoint | Required API |
| --- | --- | --- |
| Ollama | `http://127.0.0.1:11434` | `/api/tags`, `/api/chat`, and optional `/api/embed` |
| LM Studio | `http://127.0.0.1:1234` | `/v1/models`, `/v1/chat/completions`, and optional `/v1/embeddings` |
| llama.cpp | `http://127.0.0.1:8080` | OpenAI-compatible `/v1/models`, `/v1/chat/completions`, and optional `/v1/embeddings` |

Only plain HTTP loopback hosts (`localhost`, `127.0.0.1`, or `[::1]`) are accepted. Redirects,
cloud endpoints, credentials in URLs, and remote LAN addresses are rejected.

For Ollama:

```bash
ollama serve
ollama pull llama3.2
ollama pull nomic-embed-text # only needed for semantic recall
```

For LM Studio, load a local model and start its local server. For llama.cpp, a typical chat
server is `llama-server -m model.gguf --host 127.0.0.1 --port 8080`. Semantic recall stays
off unless that endpoint also provides a compatible embedding model. Use **Test provider** in
the app before enabling AI; it verifies server reachability and reports the available models.

## Features

- Streamed contextual chat includes the current pet, mood, needs, inventory, and sleep state.
- Loyal, playful, calm, curious, and gentle personalities change the system instruction.
- Per-pet history stores at most 100 messages; semantic recall stores at most 200 bounded vectors.
- Background suggestions continue while the AI window is closed, subject to a 5–240 minute cooldown.
- System text-to-speech supports voice selection and a 0.5–2.0 speaking-rate control.
- Microphone input is separately opt-in. WebView speech recognition may use an operating-system or
  cloud speech service, so the UI warns before it is enabled and requests local processing when the
  platform supports it.
- Suggested actions are limited to `feed`, `play`, `wash`, `pet`, `sleep`, `wake`, and `bark`.
- Suggested routines contain at most eight declarative steps and 30 seconds of delays. Actions and
  routines never run until the user confirms them.
- Chat can be searched, exported, imported, or cleared from the AI window.

## Privacy and storage

The app has no telemetry, account, cloud provider, or API-key field. AI requests go only to the
configured local loopback server. Saved chat and semantic memory are encrypted with
XChaCha20-Poly1305; the random encryption key is held by the operating-system credential store.
Settings do not contain conversation text.

Export is an explicit portability action and writes readable JSON at the path selected by the
user. Treat exports as private data. Import validates the archive size and message bounds before
re-encrypting it into local storage. Disabling memory or pressing **Clear** removes saved chat and
semantic memory without affecting the pet.

## Failure isolation and limits

Provider errors affect only AI. Chat uses a 30-second I/O timeout, a 128 KiB request limit, and a
512 KiB response limit. Local memory is capped at 4 MiB and imports at 2 MiB. Model names,
messages, contexts, vectors, suggested actions, and routine steps are validated in Rust before use.

## Tauri integration contract

`src-tauri/src/ai.rs` owns provider requests, streaming parsers, encrypted storage, semantic
search, and model-output validation without depending on `PetEngine`. `lib.rs` adds current pet
context and exposes commands for runtime settings, provider testing, normal/streamed chat, memory
search/export/import/clear, proactive suggestions, and action/routine confirmation. Confirmed
routines are installed through the same validator used by community behavior extensions.
