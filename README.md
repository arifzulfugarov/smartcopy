# Smart Copy

Rust-native highlight-to-copy for Chrome on macOS. After one-time Accessibility permission approval, Smart Copy runs in the background without keeping Terminal open.

## End-user commands (after npm publication)

```sh
npx --yes smartcopy-cli on
npx --yes smartcopy-cli status
npx --yes smartcopy-cli off
```

Or install globally: `npm install -g smartcopy-cli`, then `smartcopy on`.

`on` installs `~/Applications/Smart Copy.app` only if it is absent; it does not overwrite or re-sign it on each toggle. Run `smartcopy update` when deliberately replacing it. macOS may ask for Accessibility approval on first installation or after updates; this cannot be bypassed. Chrome accessibility support is requested by the native worker when needed, without requiring a browser extension.

## Development

On macOS: `cargo run -- on` or `cargo install --path . && smartcopy on`.

## Publishing

The npm package uses OS/CPU-specific optional dependencies. Release workflow builds `aarch64-apple-darwin` and `x86_64-apple-darwin` binaries on macOS runners, then publishes both native packages **before** the main `smartcopy-cli` package. Configure an `NPM_TOKEN` repository secret and push a version tag matching Cargo and npm package versions. See `.github/workflows/release.yml`.

**Not published yet.** The package name must be available on npm or renamed consistently in the manifests and launcher. Builds here have not been tested on macOS. Windows is not yet supported.

## Troubleshooting

`smartcopy logs` reports background worker errors. On macOS, approve `Smart Copy.app` in System Settings > Privacy & Security > Accessibility when requested. `smartcopy status` checks the running worker. Chrome page selection may vary by version or accessibility exposure.

## Privacy and efficiency

All selection processing stays local. The worker waits for mouse drag completion rather than scanning the screen. It doesn't copy from other applications in this initial version.
