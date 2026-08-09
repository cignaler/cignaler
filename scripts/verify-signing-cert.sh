#!/usr/bin/env bash
#
# Reproduces exactly what Tauri does with APPLE_CERTIFICATE in CI: base64
# round-trip, import into a throwaway keychain, then look for a codesigning
# identity. Run this before setting the secret, so an unusable .p12 fails here
# in five seconds instead of eleven minutes into a release build.
#
# Usage: scripts/verify-signing-cert.sh path/to/DeveloperID.p12

set -euo pipefail

EXPECTED_IDENTITY="Developer ID Application: JAKUB OSTWALD (JLMPGSCFHU)"

P12="${1:-}"
[ -n "$P12" ] || { echo "usage: $0 path/to/cert.p12" >&2; exit 1; }
[ -f "$P12" ] || { echo "No such file: $P12" >&2; exit 1; }

read -r -s -p "Password for that .p12: " P12_PASS; echo

tmp=$(mktemp -d)
keychain="$tmp/verify.keychain-db"
kc_pass=$(openssl rand -base64 12)
cleanup() { security delete-keychain "$keychain" 2>/dev/null || true; rm -rf "$tmp"; }
trap cleanup EXIT

echo "--- 1. is it actually a PKCS#12 file?"
if openssl pkcs12 -in "$P12" -passin pass:"$P12_PASS" -nokeys -legacy >/dev/null 2>&1 ||
   openssl pkcs12 -in "$P12" -passin pass:"$P12_PASS" -nokeys >/dev/null 2>&1; then
  echo "    yes, and the password opens it"
else
  echo "    NO — not a valid .p12, or the password is wrong." >&2
  echo "    In Keychain Access, select the certificate *and* its private key," >&2
  echo "    right-click -> Export 2 items -> Personal Information Exchange (.p12)." >&2
  echo "    A .cer export contains no private key and cannot sign." >&2
  exit 1
fi

echo "--- 2. does it contain a private key?"
if openssl pkcs12 -in "$P12" -passin pass:"$P12_PASS" -nocerts -legacy 2>/dev/null | grep -q "PRIVATE KEY" ||
   openssl pkcs12 -in "$P12" -passin pass:"$P12_PASS" -nocerts 2>/dev/null | grep -q "PRIVATE KEY"; then
  echo "    yes"
else
  echo "    NO private key in this export — it cannot sign anything." >&2
  exit 1
fi

echo "--- 3. base64 round-trip, the way Tauri does it"
b64=$(base64 -i "$P12")
# Tauri strips ASCII whitespace before decoding, so wrapping is not the issue.
printf '%s' "$b64" | tr -d '[:space:]' | base64 --decode > "$tmp/cert.p12"
if cmp -s "$P12" "$tmp/cert.p12"; then
  echo "    round-trip is byte-identical ($(wc -c < "$tmp/cert.p12" | tr -d ' ') bytes)"
  echo "    secret value will be $(printf '%s' "$b64" | tr -d '[:space:]' | wc -c | tr -d ' ') characters"
else
  echo "    round-trip MISMATCH — encoding is lossy." >&2
  exit 1
fi

echo "--- 4. security import, same arguments as Tauri"
security create-keychain -p "$kc_pass" "$keychain"
security unlock-keychain -p "$kc_pass" "$keychain"
security import "$tmp/cert.p12" -P "$P12_PASS" \
  -T /usr/bin/codesign -T /usr/bin/pkgbuild -T /usr/bin/productbuild \
  -k "$keychain"
echo "    imported"

echo "--- 5. codesigning identity in that keychain"
found=$(security find-identity -v -p codesigning "$keychain" || true)
echo "$found"

if printf '%s' "$found" | grep -qF "$EXPECTED_IDENTITY"; then
  echo
  echo "PASS — this .p12 works, and matches APPLE_SIGNING_IDENTITY:"
  echo "  $EXPECTED_IDENTITY"
else
  echo
  echo "The .p12 imports, but no identity matches APPLE_SIGNING_IDENTITY:" >&2
  echo "  expected: $EXPECTED_IDENTITY" >&2
  echo "Set APPLE_SIGNING_IDENTITY to the exact name shown above." >&2
  exit 1
fi
