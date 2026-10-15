# macOS signing

The macOS app is ad-hoc signed and not notarized.

## First launch

macOS says "Apple could not verify 'Abax' is free of malware". Then:

- macOS 14 and earlier: Control-click the app in Applications, choose
  **Open**, then **Open** again.
- macOS 15 and later: try to open the app once, then click **Open Anyway**
  for Abax in **System Settings > Privacy & Security**.
- Or clear the quarantine attribute: `xattr -cr /Applications/Abax.app`

## Release build

`packaging/make-app.sh` assembles `Abax.app` (binary, `Info.plist`, icon)
and, as its last step, signs the bundle with
`codesign --force --deep --sign - Abax.app`. Any later change to the bundle
breaks the seal, and macOS then reports the app as "damaged". The release
job builds the `.dmg` from the signed `.app` with `hdiutil` and ad-hoc signs
the `.dmg` too.

`packaging/verify-macos-signature.sh` then checks the `.app` and the copy
inside the `.dmg`. It fails unless `codesign --verify --deep --strict`
passes, `codesign -dv` reports `Signature=adhoc` with sealed resources, and
`Contents/_CodeSignature/CodeResources` exists. It also logs
`spctl -a -vvv -t install`, which says "rejected" for any app without a
Developer ID.

## References

- Apple, "Updates to runtime protection in macOS Sequoia":
  https://developer.apple.com/news/?id=saqachfa
- Apple, "Safely open apps on your Mac":
  https://support.apple.com/en-us/102445
- Apple, macOS Big Sur 11 release notes (arm64 code must be signed):
  https://developer.apple.com/documentation/macos-release-notes/macos-big-sur-11_0_1-universal-apps-release-notes
- `man codesign`, `man spctl`.
