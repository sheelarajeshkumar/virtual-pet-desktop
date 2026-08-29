# Contributing to Virtual Pet Desktop

Thank you for helping make desktop pets more expressive, accessible, and fun.

## Before you start

- Search existing issues and pull requests before opening a duplicate.
- Open an issue before a large feature or architecture change.
- Keep each pull request focused on one problem.
- Only submit code and assets you are allowed to license and redistribute.
- Follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## Development setup

```bash
git clone git@github.com:sheelarajeshkumar/virtual-pet-desktop.git
cd virtual-pet-desktop
npm install
npm run dev
```

Platform prerequisites are documented by [Tauri](https://v2.tauri.app/start/prerequisites/).

## Recommended workflow

1. Fork the repository.
2. Create a branch from `main`, such as `feat/corgi-pet` or `fix/fullscreen-follow`.
3. Make the smallest complete change that solves the issue.
4. Add or update a focused test for non-trivial behavior.
5. Run `npm run check` and `npm test`.
6. Update user-facing documentation when behavior changes.
7. Open a pull request using the repository template.

## Commit messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```text
feat(pets): add corgi idle animation
fix(macos): follow cursor in fullscreen spaces
docs: explain sprite atlas layout
```

Common types are `feat`, `fix`, `docs`, `test`, `refactor`, `build`, `ci`, and `chore`.

To use the included local commit template:

```bash
git config commit.template .github/COMMIT_TEMPLATE.md
```

## Code expectations

- Prefer platform APIs and existing helpers over new dependencies.
- Keep JavaScript free of build-tool-specific syntax; it runs directly in the Tauri WebView.
- Format Rust with `cargo fmt`.
- Keep platform-specific behavior behind `cfg` gates.
- Preserve click-through behavior, transparent rendering, and low CPU use.
- Avoid unrelated formatting or refactoring in a focused change.

## Tests

```bash
npm run check
npm test
```

For animation changes, also run the app and verify idle, walk, run, direction changes, and the modified action at normal size.

## Assets and generated content

- State the creator, source URL, and license for every third-party asset.
- Do not submit assets with unclear ownership or non-commercial restrictions.
- Prefer original work, CC0, or licenses compatible with redistribution.
- Disclose generative tools used to produce an asset in the pull request.
- Follow [ASSETS.md](ASSETS.md) and [the animation guide](docs/ANIMATION_GUIDE.md).

## Pull request review

Maintainers review correctness, platform behavior, performance, accessibility, licensing, and scope. Approval may require changes, additional platform testing, or clearer provenance for assets.
