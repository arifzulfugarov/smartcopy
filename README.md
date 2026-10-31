<div align="center">

# Smart Copy

### Highlight it. It's copied.

**A tiny, open-source utility that copies selected text in Chrome automatically — no `⌘C` required.**

[![npm version](https://img.shields.io/npm/v/%40arifzulfugarov%2Fsmartcopy?label=npm&color=2563eb)](https://www.npmjs.com/package/@arifzulfugarov/smartcopy)
[![MIT License](https://img.shields.io/badge/license-MIT-111827.svg)](LICENSE)
[![macOS](https://img.shields.io/badge/platform-macOS-111827)](#compatibility)
[![GitHub issues](https://img.shields.io/github/issues/arifzulfugarov/smartcopy)](https://github.com/arifzulfugarov/smartcopy/issues)

[Quick start](#quick-start) · [How it works](#how-it-works) · [Privacy](#privacy) · [Contributing](CONTRIBUTING.md)

</div>

---

## One less shortcut in your day

Highlight a sentence in Google Chrome, release your mouse, then paste anywhere. Smart Copy handles the copy step for you.

```text
Chrome:  Drag to select "Hello, world!"  →  release mouse
Notes:   ⌘V                             →  Hello, world!
```

No browser extension. No constant screen capture. No need to leave Terminal open.

## Quick start

**Requirements:** macOS (Apple Silicon or Intel), Node.js 18+, and Google Chrome. The current preview is **macOS-only**.

```bash
npx --yes @arifzulfugarov/smartcopy on
```

On first launch, macOS may ask you to allow **Smart Copy** in **System Settings → Privacy & Security → Accessibility**. Approve the permission, then run `on` again if necessary. The permission cannot be granted automatically.

Try highlighting a sentence in Chrome and pasting it into another app with `⌘V`.

### Commands

| Action | Command |
| --- | --- |
| Enable background copying | `npx --yes @arifzulfugarov/smartcopy on` |
| Check whether it's active | `npx --yes @arifzulfugarov/smartcopy status` |
| Disable background copying | `npx --yes @arifzulfugarov/smartcopy off` |
| Troubleshoot a startup issue | `npx --yes @arifzulfugarov/smartcopy logs` |

You can also install the CLI globally with `npm install -g @arifzulfugarov/smartcopy` and then use `smartcopy on`, `smartcopy off`, and `smartcopy status`.

> **Work-managed computers:** If your organization routes npm through a private registry, the public package might be unavailable there. Follow your organization's software installation policy.

## How it works

```text
Mouse drag ends in Chrome
          │
          ▼
macOS Accessibility API reads selected text
          │
          ▼
Smart Copy updates the local clipboard
          │
          ▼
Paste normally with ⌘V
```

The Rust worker waits for mouse events and only attempts to retrieve selected text after a drag is completed. It runs in the background using a macOS LaunchAgent.

## Compatibility

| Platform / app | Status |
| --- | --- |
| macOS · Chrome | **Preview — working in initial testing** |
| macOS · other browsers | Not yet supported |
| Windows | Planned |
| Telegram, Slack, VS Code and other apps | Planned |

The current release focuses on **mouse-drag selection inside Chrome**. Some webpage controls may not expose their selected text through accessibility APIs. Keyboard-only selections are not yet part of this preview.

## Privacy

- Selected text is processed on your device and written to the system clipboard.
- Smart Copy does not upload selected text to a service.
- The utility uses native accessibility information, not screenshots or OCR.
- It only attempts to copy from Google Chrome in this release.

**Be aware:** automatic clipboard updates can overwrite previous clipboard contents. Avoid selecting sensitive text while Smart Copy is enabled. macOS Accessibility permission is powerful; you can revoke it at any time in System Settings.

## Performance

Smart Copy is event-driven rather than polling the browser continuously. On one development Mac, a single idle snapshot showed **0.0% CPU and approximately 10 MiB resident memory**. This is an observation, not a battery-life guarantee; broader benchmarking is welcome.

## Development

Rust source is in [`src/`](src), and npm distribution files are in [`npm/`](npm).

```bash
cargo build --release
cargo test
```

To run locally on macOS:

```bash
cargo run -- status
```

For changes and architecture discussions, open an [issue](https://github.com/arifzulfugarov/smartcopy/issues) or read the [contribution guide](CONTRIBUTING.md).

## Roadmap

- [x] Chrome copy-on-highlight on macOS
- [x] Background `on` / `off` / `status` commands
- [x] Public npm package for Apple Silicon and Intel Macs
- [ ] Clean-install and update testing across additional Macs
- [ ] Windows support
- [ ] More browsers and desktop applications
- [ ] App exclusions and optional keyboard-selection support

## Troubleshooting

**Smart Copy says it isn't running:** Check `smartcopy logs` (or the equivalent `npx` command above). Confirm **Smart Copy** has Accessibility permission and try `on` again.

**Chrome text isn't copying:** Test by dragging to select regular webpage text, then releasing the mouse. App-specific content or certain controls may not expose selection through the accessibility tree.

**`npx` returns 404 on a work computer:** Check whether npm is configured to use an internal registry. Don't bypass organizational policy to install software.

## Contributing

Bug reports, platform testing, documentation improvements and pull requests are welcome. Start with [CONTRIBUTING.md](CONTRIBUTING.md). For security concerns, follow [SECURITY.md](SECURITY.md).

## License

Released under the [MIT License](LICENSE).

---

<div align="center">

**Smart Copy** · Built in Rust · Independent open-source project

</div>
