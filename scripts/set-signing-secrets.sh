#!/usr/bin/env bash
#
# Uploads the Apple signing + notarization secrets to GitHub Actions.
#
# Prompts for the two files you exported from Keychain Access and App Store
# Connect, base64-encodes them, and sets all seven secrets. Nothing is written
# to disk and nothing is echoed back.
#
# Requires: gh (brew install gh && gh auth login)
#
# Usage: scripts/set-signing-secrets.sh

set -euo pipefail

REPO="cignaler/cignaler"
SIGNING_IDENTITY="Developer ID Application: JAKUB OSTWALD (JLMPGSCFHU)"

command -v gh >/dev/null || { echo "gh not found. Run: brew install gh && gh auth login" >&2; exit 1; }
gh auth status >/dev/null 2>&1 || { echo "Not logged in. Run: gh auth login" >&2; exit 1; }

read -r -p "Path to Developer ID .p12: " P12
[ -f "$P12" ] || { echo "No such file: $P12" >&2; exit 1; }
read -r -s -p "Password for that .p12: " P12_PASS; echo

read -r -p "Path to App Store Connect .p8: " P8
[ -f "$P8" ] || { echo "No such file: $P8" >&2; exit 1; }
read -r -p "Key ID (the XXXXXXXXXX in AuthKey_XXXXXXXXXX.p8): " KEY_ID
read -r -p "Issuer ID (UUID from App Store Connect): " ISSUER_ID

echo "Setting secrets on $REPO ..."

base64 -i "$P12" | gh secret set APPLE_CERTIFICATE --repo "$REPO"
printf '%s' "$P12_PASS"  | gh secret set APPLE_CERTIFICATE_PASSWORD --repo "$REPO"
printf '%s' "$SIGNING_IDENTITY" | gh secret set APPLE_SIGNING_IDENTITY --repo "$REPO"

# Only ever used to unlock the throwaway keychain CI creates for the build.
openssl rand -base64 24 | tr -d '\n' | gh secret set KEYCHAIN_PASSWORD --repo "$REPO"

printf '%s' "$KEY_ID"    | gh secret set APPLE_API_KEY --repo "$REPO"
printf '%s' "$ISSUER_ID" | gh secret set APPLE_API_ISSUER --repo "$REPO"
base64 -i "$P8" | gh secret set APPLE_API_KEY_P8_BASE64 --repo "$REPO"

unset P12_PASS

echo
echo "Done. Secrets now set:"
gh secret list --repo "$REPO"
echo
echo "Next: git push origin v0.0.3"
