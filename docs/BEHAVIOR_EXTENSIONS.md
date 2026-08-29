# Community behavior extensions

Behavior extensions are declarative JSON routines. They can only select existing pet modes; they cannot execute JavaScript, access files, start programs, or use the network.

```json
{
  "schemaVersion": 1,
  "id": "example.happy-dance",
  "name": "Happy Dance",
  "description": "Play, bark once, then settle down.",
  "steps": [
    { "action": "play", "durationMs": 2500 },
    { "action": "bark", "durationMs": 700 },
    { "action": "auto", "durationMs": 500 }
  ]
}
```

Rules:

- IDs use lowercase letters, digits, dots, and hyphens, with no traversal segments.
- A routine contains 1–16 steps and finishes within 60 seconds.
- Each step lasts 250–10,000 ms.
- Allowed actions are `auto`, `follow`, `play`, `bark`, and `sleep`.
- Installed files are parsed, normalized, and copied into an app-managed directory. Symlinks are rejected.
- A routine always returns the pet to Auto after it finishes or is interrupted.

Import and run routines from **Behaviors…** in the tray menu. Start from [happy-dance.json](../examples/behaviors/happy-dance.json).
