# Security Policy

Smart Copy can read selected text after the user performs a selection in a supported app. Accessibility access is sensitive, and contributions should treat clipboard data as private.

## Supported versions

Smart Copy is currently a preview. Only the latest published release receives active fixes.

## Reporting a vulnerability

Please **do not post exploit details, secrets, or private clipboard contents in public issues**.

Use GitHub's **Report a vulnerability** feature on this repository, if enabled. If private reporting is unavailable, open a public issue requesting a secure reporting channel without including sensitive technical details.

Reports involving unintended collection of secure-field contents, unexpected network transmission, installer tampering or permission escalation are especially important.

## Principles

- Process text locally.
- Do not add telemetry of selected text.
- Never log selected clipboard contents.
- Respect macOS permission boundaries.
- Minimize clipboard writes and avoid copying from unsupported or secure controls.
