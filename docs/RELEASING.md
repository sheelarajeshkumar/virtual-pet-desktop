# Releasing

Only maintainers publish releases.

## Prepare

1. Ensure CI is green on `main`.
2. Choose a semantic version.
3. Update the version in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.
4. Run `npm install --package-lock-only` to refresh `package-lock.json`.
5. Move relevant entries from `Unreleased` in `CHANGELOG.md` into the dated version.
6. Run `npm run check`, `npm test`, and `npm run build` on each target platform.

## Sign and package

- macOS releases must be signed with a Developer ID certificate and notarized by Apple.
- Windows releases should be Authenticode-signed.
- Never commit certificates, private keys, notarization credentials, or signing passwords.

Follow Tauri's current [distribution documentation](https://v2.tauri.app/distribute/) when configuring release secrets.

## Publish

```bash
git tag -s v0.2.0 -m "Virtual Pet Desktop v0.2.0"
git push origin main --follow-tags
```

Create a GitHub release from the tag, paste the matching changelog section, attach signed installers and checksums, and mark prereleases clearly.
