# Release & code signing

The goal: a user runs one command, or drags one icon, and the app opens. No
architecture choice, no Gatekeeper warning, no terminal workarounds.

That needs three things, all wired up in `.github/workflows/`:

1. **Developer ID signing + notarization** — so macOS opens the app without complaint.
2. **A universal binary** — one DMG for Apple silicon and Intel.
3. **A Homebrew cask** — `brew install --cask cignaler/tap/cignaler`, upgrades included.

Steps 1 and 3 need one-time credential setup before they do anything.

## 1. Signing & notarization

### One-time setup

1. **Export the certificate.** In Keychain Access, find
   `Developer ID Application: JAKUB OSTWALD (JLMPGSCFHU)`, right-click →
   Export → `.p12`, set a password.

   Select the certificate **and its private key** — an export of the
   certificate alone cannot sign, and fails in CI with a bare
   `SecKeychainItemImport: One or more parameters ... were not valid`.

   Check it before you set the secret. This reproduces Tauri's import exactly,
   so a bad `.p12` fails in seconds instead of eleven minutes into a release:

   ```sh
   scripts/verify-signing-cert.sh DeveloperID.p12
   ```

   ```sh
   base64 -i DeveloperID.p12 | pbcopy
   ```

2. **Create an App Store Connect API key** at
   <https://appstoreconnect.apple.com/access/integrations/api> with the
   *Developer* role. Download the `.p8` (one chance only) and note the Key ID
   and Issuer ID.

   ```sh
   base64 -i AuthKey_XXXXXXXXXX.p8 | pbcopy
   ```

   Preferred over Apple ID + app-specific password: it does not expire when you
   change your password and is not tied to your personal account.

3. **Add GitHub repository secrets** (Settings → Secrets and variables → Actions):

   | Secret | Value |
   |---|---|
   | `APPLE_CERTIFICATE` | base64 of the `.p12` from step 1 |
   | `APPLE_CERTIFICATE_PASSWORD` | the `.p12` export password |
   | `APPLE_SIGNING_IDENTITY` | `Developer ID Application: JAKUB OSTWALD (JLMPGSCFHU)` |
   | `KEYCHAIN_PASSWORD` | any random string — password for the ephemeral CI keychain |
   | `APPLE_API_KEY` | the Key ID (e.g. `XXXXXXXXXX`) |
   | `APPLE_API_ISSUER` | the Issuer ID (a UUID) |
   | `APPLE_API_KEY_P8_BASE64` | base64 of the `.p8` from step 2 |

Notarization then happens automatically: Tauri submits the `.dmg` and `.app`,
waits for the ticket, and staples it. Stapling is what lets the app open on a
machine that is offline or behind a firewall.

### Verifying a release artifact

```sh
codesign -dv --verbose=4 /Applications/cignaler.app
codesign -dv --verbose=4 /Applications/cignaler.app/Contents/MacOS/cignaler-native-host
xcrun stapler validate /Applications/cignaler.app
spctl -a -vvv -t install /Applications/cignaler.app
```

`spctl` should print `source=Notarized Developer ID`. Test on a machine that has
never built the app — a local keychain will mask problems.

## 2. Universal binary

`release.yml` builds macOS once with `--target universal-apple-darwin`,
producing a single `cignaler_<version>_universal.dmg`.

Tauri lipos only the *main* binary. Sidecars declared in `externalBin` are
resolved by exact filename (`cignaler-native-host-universal-apple-darwin`), so
`scripts/stage-native-host.mjs` builds both arch slices and lipos them itself.
It reads the target from `TAURI_ENV_TARGET_TRIPLE`, so it does the right thing
whether Tauri invokes it as a build hook or you run it by hand.

Building universal locally needs a rustup toolchain with both targets:

```sh
rustup target add aarch64-apple-darwin x86_64-apple-darwin
pnpm tauri build --target universal-apple-darwin
```

The Homebrew `rust` formula only ships std for the host architecture, so a
universal build will fail with it. CI uses `dtolnay/rust-toolchain`, which
installs both.

To sign a local build too:

```sh
export APPLE_SIGNING_IDENTITY="Developer ID Application: JAKUB OSTWALD (JLMPGSCFHU)"
export APPLE_API_KEY=XXXXXXXXXX
export APPLE_API_ISSUER=<issuer-uuid>
export APPLE_API_KEY_PATH=$PWD/AuthKey_XXXXXXXXXX.p8
pnpm tauri build --target universal-apple-darwin
```

## 3. Homebrew tap

### One-time setup

1. **Create the tap repo**: a public GitHub repo named `homebrew-tap` under the
   `cignaler` org. The name matters — Homebrew maps `cignaler/tap` to
   `github.com/cignaler/homebrew-tap`. It can start empty.

2. **Create a fine-grained PAT** with *Contents: read and write* on that repo,
   and add it to *this* repo as the secret `HOMEBREW_TAP_TOKEN`.

`homebrew.yml` then runs on every published release: it downloads the universal
DMG, computes its SHA-256, renders `packaging/homebrew/cignaler.rb`, and commits
the result to `Casks/cignaler.rb` in the tap.

Edit the cask template — description, `zap` paths, minimum macOS — in
`packaging/homebrew/cignaler.rb`. The `version` and `sha256` lines there are
placeholders; the workflow overwrites them.

### Why a tap and not homebrew-cask proper

The main `homebrew-cask` repo enforces notability thresholds (roughly 75 stars /
30 forks / 30 watchers) that a new project will not meet. A tap costs users one
extra path segment and nothing else. Revisit later if the project takes off.

## Release checklist

1. Bump the version in `package.json`, `src-tauri/Cargo.toml`, and
   `src-tauri/tauri.conf.json`.
2. Tag and push: `git tag v0.1.0 && git push --tags`.
3. `release.yml` builds all platforms and opens a **draft** release.
4. Check the draft's artifacts, write release notes, publish.
5. Publishing fires `homebrew.yml`, which updates the tap.

## Not on the Mac App Store

Worth recording so it does not get re-litigated: the store requires App Sandbox,
and a sandboxed build cannot write the Chrome native messaging manifest to
`~/Library/Application Support/Google/Chrome/NativeMessagingHosts/`
(`src-tauri/src/main.rs:302` — under sandbox `dirs::data_dir()` resolves inside
the app container instead). No entitlement grants that access, so the browser
extension would silently stop working. Tauri also has no Mac App Store bundle
target, so packaging would be hand-rolled. Developer ID + Homebrew covers the
same users with less friction.
