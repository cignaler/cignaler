# Signing setup, start to finish

One-time setup so releases are signed and notarized. Budget about 20 minutes,
most of it waiting on Apple's website.

You need three things, and the first two you create yourself:

| What | Where it comes from | Used for |
|---|---|---|
| `DeveloperID.p12` | your Keychain | proving the app is from you |
| `AuthKey_XXXXXXXXXX.p8` | App Store Connect | asking Apple to notarize it |
| 7 GitHub secrets | you paste them | letting CI use both |

---

## Step 1 — Export the certificate

The certificate and its private key are already in your login keychain. You
need them in one `.p12` file.

1. Open **Keychain Access** (⌘-Space, type "Keychain Access").

2. In the sidebar pick **login**. If the sidebar is hidden, View → Show
   Sidebar.

3. In the category list choose **My Certificates**. This matters — the
   "Certificates" category shows certificates *without* their private keys,
   and a certificate alone cannot sign anything.

4. Find **Developer ID Application: JAKUB OSTWALD (JLMPGSCFHU)**.

   Ignore `Apple Development: JAKUB OSTWALD (ZGHN97V59N)`. That one is for
   running builds locally, not for distributing them.

5. Click the disclosure triangle next to it. A private key named after you
   should appear underneath. **If there is no key, stop** — the certificate
   cannot sign, and you would need to create a new one at
   <https://developer.apple.com/account/resources/certificates>.

6. Select the certificate, then ⌘-click the private key so **both** rows are
   highlighted.

7. Right-click → **Export 2 items…**

   If the menu says "Export 1 item", you only have one row selected. Go back
   to step 6. Exporting only the certificate is the single most common way to
   end up with a `.p12` that fails in CI.

8. Set **File Format** to **Personal Information Exchange (.p12)**. Save it to
   your Desktop as `DeveloperID.p12`.

9. macOS asks for a password **to protect the exported file**. Make one up.
   Write it down — this becomes the `APPLE_CERTIFICATE_PASSWORD` secret.

10. macOS then asks for your **login keychain password**. That is your Mac
    account password, and it is *not* the same as the one you just invented.
    It authorizes the export and is never stored anywhere.

You should now have `~/Desktop/DeveloperID.p12`.

---

## Step 2 — Verify the export before trusting it

```bash
cd ~/work/cignaler
scripts/verify-signing-cert.sh ~/Desktop/DeveloperID.p12
```

It prompts for the password from step 1.9, then reproduces exactly what CI
does: base64 round-trip, `security import` with Tauri's flags, and an identity
lookup.

A good run ends with:

```
PASS — this .p12 works, and matches APPLE_SIGNING_IDENTITY:
  Developer ID Application: JAKUB OSTWALD (JLMPGSCFHU)
```

If it fails instead:

| Message | Meaning |
|---|---|
| `not a valid .p12, or the password is wrong` | wrong password, or you saved a `.cer` |
| `NO private key in this export` | you exported 1 item instead of 2 — redo step 6 |
| `no identity matches APPLE_SIGNING_IDENTITY` | the `.p12` has the wrong certificate in it, most likely the Apple Development one |

Do not continue until this prints `PASS`. Every minute spent here saves ten
in CI.

---

## Step 3 — Create the notarization key

Signing proves who you are. Notarization is Apple scanning the build and
issuing a ticket that stops Gatekeeper complaining. It needs separate
credentials.

1. Go to <https://appstoreconnect.apple.com/access/integrations/api>.

2. Make sure you are on the **Team Keys** tab, not Individual Keys. A team key
   is not tied to your personal login.

3. Click **+** to generate a key.

4. Name it something recognizable, e.g. `cignaler-notarization`.

5. Set **Access** to **Developer**. Anything narrower cannot submit for
   notarization.

6. Click **Generate**.

7. **Download the `.p8` now.** Apple allows exactly one download, ever. If you
   lose it you cannot recover it — you revoke the key and make a new one. Save
   it next to the `.p12`, e.g. `~/Desktop/AuthKey_XXXXXXXXXX.p8`.

8. Copy two identifiers off that page:

   - **Key ID** — 10 characters, in the key's row. Also embedded in the
     filename: `AuthKey_<KEYID>.p8`.
   - **Issuer ID** — a UUID shown above the table. It is the same for every
     key on your team.

---

## Step 4 — Put all seven into GitHub

### The quick way

```bash
brew install gh
gh auth login          # pick GitHub.com, HTTPS, log in via browser
scripts/set-signing-secrets.sh
```

It asks for both file paths, the `.p12` password, the Key ID and the Issuer
ID, then sets all seven and lists them back. It generates
`KEYCHAIN_PASSWORD` itself, since that one is only ever used to unlock the
throwaway keychain CI builds in.

### The manual way

Go to
<https://github.com/cignaler/cignaler/settings/secrets/actions> and click
**New repository secret** seven times. Names must match exactly — they are
case-sensitive, and a typo reads as "not set".

| Secret | Value |
|---|---|
| `APPLE_CERTIFICATE` | run `base64 -i ~/Desktop/DeveloperID.p12 \| pbcopy`, then paste |
| `APPLE_CERTIFICATE_PASSWORD` | the password you invented in step 1.9 |
| `APPLE_SIGNING_IDENTITY` | `Developer ID Application: JAKUB OSTWALD (JLMPGSCFHU)` |
| `KEYCHAIN_PASSWORD` | run `openssl rand -base64 24 \| pbcopy`, then paste |
| `APPLE_API_KEY` | the Key ID from step 3.8 |
| `APPLE_API_ISSUER` | the Issuer ID from step 3.8 |
| `APPLE_API_KEY_P8_BASE64` | run `base64 -i ~/Desktop/AuthKey_*.p8 \| pbcopy`, then paste |

Paste into the box and click Add — don't retype anything. GitHub hides
secrets after saving, so a mistake can only be fixed by overwriting.

When you are done the list should show exactly seven entries.

---

## Step 5 — Release

The tag currently points at a commit that predates these fixes, so replace it:

```bash
cd ~/work/cignaler
git fetch origin
git tag -f -a v0.0.3 -m "Cignaler v0.0.3" origin/main
git push origin :refs/tags/v0.0.3   # remove the old one
git push origin v0.0.3              # push the new one
```

Before that, delete the leftover draft release at
<https://github.com/cignaler/cignaler/releases> — it holds Linux artifacts
from a broken build, and a new run would append to it rather than replace it.

Then watch <https://github.com/cignaler/cignaler/actions>.

- If a secret is missing, the macOS job now fails in seconds and names it.
- Otherwise expect 20–30 minutes: the universal build compiles the Rust tree
  twice, then Apple's notary service takes its own time.
- The Windows job will fail. It has never worked in any release; unrelated.

---

## Step 6 — Check it actually worked

A missing credential fails loudly now, but a *wrong* one can still produce an
unsigned build that uploads happily. Check by hand, once:

```bash
# download the DMG from the draft release, then
hdiutil attach ~/Downloads/cignaler_0.0.3_universal.dmg
spctl -a -vvv -t install /Volumes/cignaler/cignaler.app
```

What you want:

```
source=Notarized Developer ID
```

Other outcomes:

| Output | Meaning |
|---|---|
| `source=Unnotarized Developer ID` | signing worked, notarization did not — check the `.p8` secrets |
| `rejected` | not signed at all |

Also confirm it is genuinely universal:

```bash
lipo -archs /Volumes/cignaler/cignaler.app/Contents/MacOS/cignaler
# expect: x86_64 arm64
hdiutil detach /Volumes/cignaler
```

Then publish the draft. Publishing is what triggers the Homebrew cask update,
which needs the `cignaler/homebrew-tap` repo and a `HOMEBREW_TAP_TOKEN` secret
to exist — see [RELEASE.md](RELEASE.md). If they don't yet, the release still
publishes fine and you can re-run that job later.

---

## Afterwards

Keep `DeveloperID.p12`, its password, and the `.p8` somewhere safe — a password
manager is ideal. The `.p8` in particular cannot be re-downloaded.

Your certificate expires **1 February 2027**:

```bash
security find-certificate -c "Developer ID Application: JAKUB OSTWALD" -p \
  | openssl x509 -noout -enddate
```

Renewing means issuing a new certificate at
<https://developer.apple.com/account/resources/certificates>, then redoing
steps 1, 2 and the `APPLE_CERTIFICATE` secret. Builds signed before expiry keep
working — notarization tickets do not expire with the certificate.
