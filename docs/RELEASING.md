# Releasing

Only maintainers publish releases. The release workflow creates a **draft** after it has built and cryptographically verified the macOS and Windows installers. It never publishes a public release by itself.

## One-time repository setup

1. Create a protected GitHub Actions environment named `release`. Require maintainer approval before a job can access it.
2. Add these environment secrets. Never commit certificates, private keys, API keys, passwords, or timestamp credentials.

| Name | Value |
| --- | --- |
| `APPLE_CERTIFICATE` | Base64-encoded `.p12` containing the **Developer ID Application** certificate and private key. |
| `APPLE_CERTIFICATE_PASSWORD` | Password used to export that `.p12`. |
| `KEYCHAIN_PASSWORD` | New random password used for the short-lived CI keychain. |
| `APPLE_API_ISSUER` | App Store Connect API issuer ID. |
| `APPLE_API_KEY` | App Store Connect API key ID. |
| `APPLE_API_PRIVATE_KEY` | Contents of the downloaded App Store Connect `.p8` private key. |
| `WINDOWS_CERTIFICATE` | Base64-encoded Windows code-signing `.pfx`. |
| `WINDOWS_CERTIFICATE_PASSWORD` | Password used to export that `.pfx`. |

Add the non-secret environment variable `WINDOWS_TIMESTAMP_URL`, using the RFC 3161 timestamp endpoint supplied by the code-signing certificate issuer. The workflow refuses to run without it.

This workflow uses Tauri's `.pfx` certificate route. Many modern Windows certificates are hardware- or cloud-held and cannot be exported as a `.pfx`; in that case the release gate will correctly remain blocked. Replace the ephemeral configuration with the provider's documented custom sign command or Azure Artifact Signing configuration before attempting a release.

The implementation follows Tauri's current guides for [macOS signing and notarization](https://v2.tauri.app/distribute/sign/macos/), [Windows signing](https://v2.tauri.app/distribute/sign/windows/), and [GitHub release pipelines](https://v2.tauri.app/distribute/pipelines/github/). A free Apple developer account cannot notarize external releases.

## Prepare a version

1. Ensure CI is green on `main`, including the Windows unsigned installer and the Linux compile investigation.
2. Choose a semantic version, for example `0.2.0`.
3. Update the same version in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.
4. Run `npm install --package-lock-only` to refresh `package-lock.json`.
5. Move relevant entries from `Unreleased` in `CHANGELOG.md` into the dated version.
6. Run `npm run check`, `npm test`, and `npm run build` on each supported target platform.
7. Create and push a signed tag:

```bash
git tag -s v0.2.0 -m "Virtual Pet Desktop v0.2.0"
git push origin main --follow-tags
```

Pushing a `v*` tag starts `.github/workflows/release.yml`. The workflow can also be run manually for an existing tag.

## Automated release gate

The protected `release` environment must approve the run and provide every secret above. The preflight job then checks that:

- the tag is semver-shaped and resolves to a commit;
- the tag version matches all three application version files;
- macOS notarization and Windows signing inputs are present; and
- the timestamp URL is a valid HTTP(S) URL.

After that, the workflow:

1. imports the Developer ID certificate into a temporary macOS keychain, builds a DMG, verifies `codesign`, and validates the notarization staple;
2. imports the Windows certificate only on the temporary runner, builds the NSIS installer with an ephemeral Tauri signing config, and validates it with `signtool`; and
3. uploads both signed installers into a GitHub **draft** release.

The macOS job packages the architecture supplied by `macos-latest`. Do not represent it as universal or Intel-compatible until an additional verified architecture job is added.

## Manual release gate

Before publishing the draft:

1. Install the signed Windows artifact on Windows 10 and Windows 11, complete [WINDOWS_TESTING.md](WINDOWS_TESTING.md), and attach the completed report to the release discussion or issue.
2. Install the notarized DMG on a clean macOS account and verify launch, tray controls, settings, audio, fullscreen overlay and uninstall behavior.
3. Check the artifact names, versions, release notes, and checksums. Verify that no unsigned CI artifact is attached.
4. Have a maintainer review the completed platform evidence and publish the GitHub draft.

The workflow has not itself proved physical Windows GUI behavior or signing until a protected run succeeds and the manual checklist is completed.
