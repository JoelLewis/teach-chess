# Release & Distribution Checklist

Status: **playtest builds only** — local DMG builds are ad-hoc signed and must not
be shipped wide until the signing TODO below is done.

## TODO before shipping wide (blocking)

Local builds use `"signingIdentity": "-"` (ad-hoc). A downloaded DMG signed this
way fails Gatekeeper on other Macs — on Apple Silicon it shows
*"ChessMentor is damaged and can't be opened"*, and right-click → Open does
**not** bypass it.

1. **Create a Developer ID Application certificate** (the cert currently on this
   machine is "Apple Development", which only works locally and cannot be notarized):
   - Xcode → Settings → Accounts → select Apple ID → **Manage Certificates…** →
     **+** → **Developer ID Application**
   - Requires a paid Apple Developer membership; only the **Account Holder** role
     can create Developer ID certificates.
2. **Point the build at it** — in `src-tauri/tauri.conf.json`, set
   `bundle.macOS.signingIdentity` to the cert name, e.g.
   `"Developer ID Application: Joel Lewis (TEAMID)"`.
3. **Notarize** — generate an app-specific password at appleid.apple.com, then
   export before building (Tauri notarizes the DMG automatically when set):
   ```sh
   export APPLE_ID="joel.e.lewis@gmail.com"
   export APPLE_PASSWORD="<app-specific password>"
   export APPLE_TEAM_ID="<TEAMID>"
   npm run tauri build
   ```
4. Verify on a clean machine (or a fresh user account): download the DMG, drag
   to /Applications, double-click — it must open with no Gatekeeper warning.

## Interim workaround (un-notarized playtest builds)

If a tester needs a build before notarization is set up, ship this one-liner
with the DMG (run after dragging the app to /Applications):

```sh
xattr -dr com.apple.quarantine /Applications/ChessMentor.app
```

Transfers that skip quarantine (USB drive, `scp`) avoid the issue entirely;
browser downloads, AirDrop, and most chat apps set the quarantine flag.

## Known limitations

- **Apple Silicon only** — only the `aarch64-apple-darwin` Stockfish sidecar is
  fetched. Intel support needs `./scripts/fetch-stockfish.sh` for
  `x86_64-apple-darwin` plus a universal build.
- **DMG is ~850 MB** — the Gemma model (806 MB) and tokenizer are bundled as
  resources by design (offline coaching).

## Build paths

- **Playtest / direct distribution:** `npm run tauri build` →
  `src-tauri/target/release/bundle/dmg/`. Not sandboxed; entitlements are
  intentionally not applied to this build.
- **Mac App Store:** `./scripts/build-appstore.sh` — re-signs with the App
  Sandbox entitlements in `src-tauri/entitlements/` and builds a `.pkg`.
  Requires "3rd Party Mac Developer" certs (separate from Developer ID).
