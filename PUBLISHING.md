# First npm publication (browser-only)

1. Upload the contents of this source archive to the root of `arifzulfugarov/smartcopy` using GitHub web UI, retaining nested paths.
2. Make sure `.github/workflows/publish.yml` exists and the previous automatically publishing `.github/workflows/release.yml` is removed or disabled.
3. Open GitHub > Actions > Publish Smart Copy to npm > Run workflow.
4. Wait for the build and publish jobs to finish. Do not run `npx` until all three npm packages are visible under your npm account.
5. Verify package ownership, then test `npx --yes @arifzulfugarov/smartcopy status` before `on`.

This test package has not been compiled on macOS in this environment. Do not claim it is working until the workflow and a macOS test pass.

**Release security:** Revoke the temporary npm token after setting up trusted publishing. The current token can publish packages and has wide access.
