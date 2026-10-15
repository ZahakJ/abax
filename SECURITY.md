# Security policy

Abax makes no network requests. It reads one JSON profile from the platform
data directory and a few `ABAX_*` environment variables.

## Supported versions

The latest release.

## Reporting a vulnerability

Use GitHub's private vulnerability reporting ("Security" tab > "Report a
vulnerability"), not a public issue. Include the version, platform and a
reproduction if you have one.

## Release integrity

Every release attaches `SHA256SUMS.txt` covering all assets. The Android
APKs are signed with the project's release key; the desktop binaries are not
code-signed.
