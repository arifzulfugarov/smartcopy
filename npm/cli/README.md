# Smart Copy CLI

Native Rust copy-on-selection helper for Chrome on macOS.

```sh
npx --yes smartcopy-cli on
npx --yes smartcopy-cli status
npx --yes smartcopy-cli off
```

First run installs `~/Applications/Smart Copy.app`. macOS requires a one-time Accessibility approval. The app then runs as a per-user LaunchAgent, even after Terminal closes. Run `npx --yes smartcopy-cli update` only when deliberately upgrading the installed app. This npm package must be published with its matching architecture packages before these commands work.
