# Contributing to Smart Copy

Thanks for helping make copy-on-selection more useful and reliable.

## Good first contributions

- Reproduce and document Chrome selection edge cases on macOS.
- Test installation on a fresh Apple Silicon or Intel Mac.
- Improve accessibility error messages or permission setup documentation.
- Add unit tests and improve documentation.

## Before opening a pull request

1. Check existing issues and open one for larger behavioral changes.
2. Keep changes focused and explain the motivation.
3. Run `cargo fmt --check`, `cargo test`, and `cargo clippy` on supported development platforms, or explain what you could not run.
4. Avoid introducing telemetry or transmitting selected text. Any change to clipboard handling or accessibility permissions needs a clear privacy explanation.
5. Include the macOS version, Chrome version and steps to reproduce for behavior changes.

## Project layout

- `src/` — Rust CLI, macOS selection handling and background service.
- `npm/` — platform-specific prebuilt binaries and npm launcher packages.
- `.github/workflows/` — CI and release automation.

## Pull request checklist

- [ ] The change is focused and documented.
- [ ] No secrets, credentials, or personal clipboard contents are included.
- [ ] Tests or manual verification steps are included.
- [ ] User-visible behavior is reflected in the README.

Be respectful and constructive in issues and reviews.
