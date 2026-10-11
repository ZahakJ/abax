# Security policy

Abax is an offline desktop and Android application. It makes no network requests, runs no server, and stores one JSON file (the learner's profile) in the platform data directory. The attack surface is therefore small: parsing that profile, parsing command-line environment variables, and the GPU shader pipeline.

## Supported versions

Only the latest release receives fixes.

## Reporting a vulnerability

Please do not open a public issue for a security problem. Use GitHub's private vulnerability reporting on this repository ("Security" tab > "Report a vulnerability"). Include the version, platform and a reproduction if you have one.

You will get an acknowledgement within a week. Fixes ship as a new release; the advisory is published once the fix is available.

## Release integrity

Every release attaches `SHA256SUMS.txt` covering all assets. The Android APKs are signed with the project's release key; the desktop binaries are not code-signed (see the README for the first-launch steps this implies).
